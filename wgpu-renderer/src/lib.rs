mod camera;
mod light;
mod materials;
mod resource;
mod scene;
mod unity;
mod utils;
mod entity;
mod queries;
mod ray;
mod mesh;
mod map;
mod frustum;
mod stat;

use std::cell::RefCell;
use log::{error, info, warn};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};
use winit::event::MouseScrollDelta;

use crate::resource::{ResourceManager};
use crate::scene::{Scene};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::dpi::PhysicalSize;
use winit::window::WindowId;
use crate::entity::{Entity, TransformSystem};
use crate::materials::{Texture};
use crate::ray::Ray;
use crate::unity::UnityScene;

// 全局场景路径存储（用于 wasm 和本地环境）
use std::sync::{OnceLock, Mutex};
use once_cell::sync::Lazy;

static SCENE_PATH: OnceLock<String> = OnceLock::new();


#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
use crate::stat::{set_loading_state, SceneCommand, SceneLoadingState, COMMAND_QUEUE, LOADING_PROGRESS, QUERY_RESULTS};

pub struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub scene: Scene,
    // 资源管理器
    pub resource_manager: ResourceManager,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    pub depth_texture: Option<Texture>,
    mouse_pos: (f32, f32),
    // 当前加载的场景路径
    current_scene_path: String,

    self_ref: Option<Rc<RefCell<State>>>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        set_loading_state(SceneLoadingState::Initializing, 0.0, "Initializing graphics...");

        #[cfg(target_arch = "wasm32")]
        let size = {
            // WASM 环境：确保最小尺寸
            PhysicalSize::new(
                size.width.max(1),
                size.height.max(1)
            )
        };

        // 获得Instance面板
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        set_loading_state(SceneLoadingState::Initializing, 0.1, "Creating surface...");
        let surface = instance.create_surface(window.clone()).unwrap();

        set_loading_state(SceneLoadingState::Initializing, 0.15, "Requesting adapter...");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let features = adapter.features()
            & (wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES);

        if features.contains(wgpu::Features::TIMESTAMP_QUERY) {
            info!("Adapter supports timestamp queries.");
        } else {
            info!("Adapter does not support timestamp queries, aborting.");
        }
        // if !features.contains(wgpu::Features::SHADER_F16) {
        //     panic!("设备不支持 SHADER_F16 特性");
        // }

        let timestamps_inside_passes = features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES);
        if timestamps_inside_passes {
            info!("Adapter supports timestamp queries within passes.");
        } else {
            warn!("Adapter does not support timestamp queries within passes.");
        }

        set_loading_state(SceneLoadingState::Initializing, 0.2, "Creating device...");
        // 通过适配器获取device以及queue(类似管线队列)
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("WGPU Device Adapter"),

                // required_features: wgpu::Features::SHADER_F16, // 启用 f16 支持,
                required_features: features,
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits {
                        max_texture_dimension_2d: 4096, // 尝试请求 4096，看 WebGL2 后端是否能支持
                        ..wgpu::Limits::downlevel_webgl2_defaults()  // ..Default::default()
                    }
                } else {
                    wgpu::Limits::default()
                },
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        // 贴图 渲染格式
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        set_loading_state(SceneLoadingState::Initializing, 0.25, "Configuring surface...");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // 做场景资源转换，所需资源
        // 获取场景路径（支持 wasm 通过 set_scene_path 设置，或从环境变量读取）
        let scene_path = get_scene_path();
        let path = PathBuf::from(&scene_path);

        set_loading_state(SceneLoadingState::LoadingScene, 0.3, "Loading scene file...");

        // 在 wasm 环境下使用 worker，在非 wasm 环境下使用正常方法
        let mut unity_scene = {
            let mut uns = UnityScene::new();
            uns.from_str(path.clone()).await.map_err(|e| {
                error!("Failed to load unity scene: {:?}", e);
                e
            })?
        };

        set_loading_state(SceneLoadingState::LoadingScene, 0.4, "Scene file parsed");

        // 3. 加载资源
        let mut scene = Scene::new(&device, &config, unity_scene.game_object_raw.len() * 2);
        let mut resource_manager = ResourceManager::new(&device, &queue);
        set_loading_state(SceneLoadingState::LoadingScene, 0.45, "Creating scene structure...");

        set_loading_state(SceneLoadingState::LoadingAssets, 0.5, "Loading resource mapping...");

        resource_manager.loading_mapping().await.map_err(| e| {
            error!("Failed to load resource mapping: {:?}", e);
            e
        })?;

        set_loading_state(SceneLoadingState::LoadingAssets, 0.6, "Loading scene assets...");

        Scene::loading_scene(&device, &queue, &mut scene, &mut unity_scene, &mut resource_manager, &config).await.map_err(|e| {
            error!("Failed to load scene scene: {:?}", e);
            e
        })?;

        set_loading_state(SceneLoadingState::Ready, 1.0, "Ready loaded...");

        Ok(Self {
            self_ref: None,
            window,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            scene,
            resource_manager,
            depth_texture: None,
            mouse_pos: (0.0, 0.0),
            current_scene_path: scene_path,
        })
    }

    // 初始化后设置自引用
    pub fn set_self_ref(&mut self, rc: Rc<RefCell<State>>) {
        self.self_ref = Some(rc);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            // 重新创建深度纹理
            self.depth_texture = Some(Texture::create_depth_texture(&self.device, &self.config, "depth_texture"));
        }
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {
                // println!("key pressed: {:?} {:?}", code, is_pressed);
                // 监听Key，保存移动方向
                self.scene.camera.controller.handle_key(code, is_pressed);
            }
        }
    }

    fn on_click(&self, scene: &Scene, window_size: PhysicalSize<u32>) {
        let ray = Ray::from_screen_coords(
            self.mouse_pos,
            (window_size.width, window_size.height),
            self.scene.camera.get_view_matrix(),
            self.scene.camera.get_projection_only(), // 使用纯投影矩阵
        );

        if let Some((entity_id, distance)) = scene.pick_entity(&ray, &self.resource_manager) {
            println!("Clicked entity {} at distance {}", entity_id, distance);
            println!("pick entity: {:?} position: {:?}", ray, self.scene.transform_system.get_local_transform(Entity::new(entity_id)).unwrap());
            
            // Handle click event here
        }
    }

    /// 处理命令队列
    fn process_commands(&mut self) {
        let Ok(progress) = LOADING_PROGRESS.lock() else {
            info!("progress error");
            return;
        };

        if progress.state.is_busy() {
            return;
        }

        // 获取所有待处理的命令
        let commands = if let Ok(mut queue) = COMMAND_QUEUE.lock() {
            queue.drain(..).collect::<Vec<_>>()
        } else {
            return;
        };

        // 处理每个命令
        for command in commands {
            match command {
                SceneCommand::ChangeScene { path } => {
                    self.handle_scene_change(path);
                },
                SceneCommand::SetCameraPosition { x, y, z } => {
                    self.scene.camera.set_eye(cgmath::Point3::new(x, y, z));
                },
                SceneCommand::SetCameraTarget { x, y, z } => {
                    self.scene.camera.set_target(cgmath::Point3::new(x, y, z));
                },
            }
        }
    }

    /// 重新加载场景（动态切换）
    pub async fn reload_scene(&mut self, scene_path: String) -> anyhow::Result<()> {
        info!("Reloading scene: {}", scene_path);

        // ===== 阶段 1: 开始切换 =====
        set_loading_state(SceneLoadingState::Switching, 0.0, "Starting scene switch...");

        // ===== 阶段 2: 清理当前场景 =====
        set_loading_state(SceneLoadingState::DisposingScene, 0.05, "Clearing scene entities...");
        self.scene.entities.clear();

        set_loading_state(SceneLoadingState::DisposingScene, 0.10, "Resetting transform system...");
        self.scene.transform_system = TransformSystem::new();

        set_loading_state(SceneLoadingState::DisposingScene, 0.12, "Clearing display maps...");
       self.scene.clear_entity();

        set_loading_state(SceneLoadingState::DisposingScene, 0.15, "Resetting render batches...");
        self.scene.render_batches = scene::RenderBatchSystem::new();

        // ===== 阶段 3: 解析场景文件 =====
        let path = std::path::PathBuf::from(&scene_path);
        let scene_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("scene");

        set_loading_state(
            SceneLoadingState::LoadingScene,
            0.20,
            &format!("Parsing scene file: {}...", scene_name)
        );

        let mut unity_scene = {
            let mut uns = UnityScene::new();
            uns.from_str(path.clone()).await.map_err(|e| {
                set_loading_state(
                    SceneLoadingState::SceneParseError,
                    0.20,
                    &format!("Failed to parse scene: {}", e)
                );
                error!("Failed to load unity scene: {:?}", e);
                e
            })?
        };

        self.scene.reload();

        // ===== 阶段 4: 加载场景资源 =====
        set_loading_state(
            SceneLoadingState::LoadingAssets,
            0.35,
            "Loading meshes, textures and materials..."
        );

        Scene::loading_scene(
            &self.device,
            &self.queue,
            &mut self.scene,
            &mut unity_scene,
            &mut self.resource_manager,
            &self.config,
        )
            .await
            .map_err(|e| {
                set_loading_state(
                    SceneLoadingState::AssetLoadError,
                    0.35,
                    &format!("Failed to load assets: {}", e)
                );
                e
            })?;

        // ===== 阶段 5: 设置场景 =====
        set_loading_state(
            SceneLoadingState::Setting,
            0.85,
            "Setting up render pipelines..."
        );
        // 不进行初始化灯光， buffer不同
        // self.scene.setup(&self.device, &self.queue);

        // ===== 阶段 6: 完成 =====
        set_loading_state(
            SceneLoadingState::Ready,
            1.0,
            &format!("Scene '{}' loaded successfully", scene_name)
        );
        info!("Scene reloaded successfully: {}", scene_path);

        Ok(())
    }

    /// 处理场景切换命令
    fn handle_scene_change(&mut self, scene_path: String) {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("🎬 Handling scene change: {}", scene_path).into());

        info!("Handling scene change to: {}", scene_path);

        #[cfg(not(target_arch = "wasm32"))]
        {
            // 本地环境：直接同步加载
            match pollster::block_on(self.reload_scene(scene_path.clone())) {
                Ok(_) => {
                    info!("✓ Scene loaded successfully: {}", scene_path);
                    self.current_scene_path = scene_path;
                },
                Err(e) => {
                    error!("✗ Failed to load scene: {:?}", e);
                },
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(state_rc) = self.self_ref.clone() {
                let scene_path_clone = scene_path.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    // 在 async 块内部借用
                    {
                        let mut state = state_rc.borrow_mut();
                        state.reload_scene(scene_path_clone.clone()).await;

                        state.window.request_redraw();
                    }
                });
            }

        }
    }

    /// 更新查询结果
    fn update_query_results(&self) {
        if let Ok(mut results) = QUERY_RESULTS.lock() {
            let eye = self.scene.camera.eye();
            let target = self.scene.camera.target();

            results.camera_position = [eye.x, eye.y, eye.z];
            results.camera_target = [target.x, target.y, target.z];
            results.entity_count = self.scene.entities.len();
            results.visible_count = self.scene.total_show_entities();
            results.current_scene = self.current_scene_path.clone();
        }
    }

    fn update(&mut self) {
        // 处理命令队列
        self.process_commands();

        // 更新场景
        self.scene.update(&self.queue, 1.0, &self.resource_manager);

        // 更新资源管理器帧计数
        self.resource_manager.update_frame();

        // 每60帧（约1秒）清理一次未使用的资源
        if self.resource_manager.current_frame % 60 == 0 {
            let stats = self.resource_manager.get_resource_stats();
            #[cfg(debug_assertions)]
            println!("Resource stats before cleanup - Meshes: {}/{}, Materials: {}/{}, Textures: {}/{}",
                stats.0, stats.1, stats.2, stats.3, stats.4, stats.5);

            self.resource_manager.unload_all_unused_resources();

            let stats_after = self.resource_manager.get_resource_stats();
            #[cfg(debug_assertions)]
            if stats.0 != stats_after.0 || stats.2 != stats_after.2 || stats.4 != stats_after.4 {
                println!("Resource cleanup - Meshes: {}/{}, Materials: {}/{}, Textures: {}/{}",
                    stats_after.0, stats_after.1, stats_after.2, stats_after.3, stats_after.4, stats_after.5);
            }
        }

        // 将结果写入至查询结果中
        self.update_query_results();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // 在第一次渲染时创建深度纹理
        if self.depth_texture.is_none() {
            let width = output.texture.width();
            let height = output.texture.height();

            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!(
                "Creating depth texture: {}x{}", width, height
            ).into());

            self.depth_texture = Some(Texture::create_depth_texture(&self.device, &self.config, "depth_texture"));
        }

        let depth_view = self.depth_texture.as_ref().unwrap();


        // 创建指令
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let used_resources = {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.scene.background_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                //
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment{
                    view: &depth_view.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.scene.render(
                &self.device,
                &mut _render_pass,
                &self.resource_manager
            )
        };

        // 标记本帧使用的资源
        for (mesh_id, material_id) in used_resources {
            self.resource_manager.mark_mesh_used(&mesh_id);
            self.resource_manager.mark_material_used(&material_id);
        }

        // 记录结束时间戳
        // encoder.write_timestamp(&self.query_set, 1);
        //
        // // 解析查询结果到 buffer
        // encoder.resolve_query_set(&self.query_set, 0..2, &self.query_buffer, 0);
        //
        // // 复制到可读取的 staging buffer
        // encoder.copy_buffer_to_buffer(&self.query_buffer, 0, &self.staging_buffer, 0, 16);


        // 提交任务
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    state: Option<Rc<RefCell<State>>>,
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy()); // web需要
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}

impl ApplicationHandler<State> for App {
    // 窗口恢复功能
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        // 创建一个window对象
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            // If we are not on web we can use pollster to
            // await the
            let mut state = pollster::block_on(State::new(window)).unwrap();

            state.scene.setup(&state.device, &state.queue);

            self.state = Some(Rc::new(RefCell::new(state)));
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Run the future asynchronously and use the
            // proxy to send the results to the event loop
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(
                                State::new(window)
                                    .await
                                    .map_err(|e| log::error!("{:?}", e))
                                    .expect("Unable to create canvas!!!")
                            )
                            .is_ok()
                    )
                });
            }
        }
    }

    // 处理用户事件
    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {

        #[cfg(target_arch = "wasm32")]
        {
            // 设置window的宽高
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        // This is where proxy.send_event() ends up
        let state_rc = Rc::new(RefCell::new(event));

        self.state = Some(state_rc);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        // ⚠️ 使用 try_borrow_mut 避免在场景加载时 panic
        let mut state = match &mut self.state {
            Some(s) => match s.try_borrow_mut() {
                Ok(state) => state,
                Err(_) => return, // 场景加载中，跳过设备事件
            },
            None => return,
        };

        // 处理鼠标移动（用于FPS相机控制）
        if let DeviceEvent::MouseMotion { delta } = event {
            state.scene.camera.controller.handle_mouse_move(delta.0 as f32, delta.1 as f32);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // ⚠️ 使用 try_borrow_mut 避免在场景加载时 panic
        // 先检查能否借用，避免借用冲突
        let state_ref = match &self.state {
            Some(s) => s,
            None => return,
        };

        let mut state = match state_ref.try_borrow_mut() {
            Ok(s) => s,
            Err(_) => {
                // 场景正在异步加载中，跳过这一帧
                #[cfg(target_arch = "wasm32")]
                if matches!(event, WindowEvent::RedrawRequested) {
                    info!("Redraw requested Error");
                    // 继续请求重绘，下一帧再试
                    if let Ok(state) = state_ref.try_borrow() {
                        state.window.request_redraw();
                    }
                }
                return;
            }
        };

        state.set_self_ref(Rc::clone(state_ref));

        match event {
            // 退出
            WindowEvent::CloseRequested => event_loop.exit(),
            // window窗口size调整接口
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            // 重绘请求
            WindowEvent::RedrawRequested => {
                // 更新每次的状态
                state.update();

                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        // 报错调整宽高
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            // 处理键盘按键
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            // 处理鼠标滚轮和触控板缩放
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_delta = match delta {
                    // Windows/Linux 鼠标滚轮
                    MouseScrollDelta::LineDelta(_x, y) => y,
                    // macOS 触控板双指手势
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
                };
                state.scene.camera.controller.handle_scroll(scroll_delta);
            }
            WindowEvent::CursorMoved { position, .. } => {
                state.mouse_pos = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state: button_state, button, .. } => {
                match (button, button_state) {
                    // 左键点击：选取实体
                    (MouseButton::Left, ElementState::Pressed) => {
                        let size = state.window.inner_size();
                        state.on_click(&state.scene, size);
                    }
                    // 右键点击：切换FPS模式
                    (MouseButton::Right, ElementState::Pressed) => {
                        state.scene.camera.controller.toggle_mouse_capture();

                        // 如果进入FPS模式，初始化yaw和pitch角度
                        if state.scene.camera.controller.is_mouse_captured() {
                            let eye = state.scene.camera.eye().clone();
                            let target = state.scene.camera.target().clone();
                            state.scene.camera.controller.init_angles_from_target(
                                &eye,
                                &target,
                            );

                            // 锁定并隐藏鼠标
                            state.window.set_cursor_visible(false);
                            let _ = state.window.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                .or_else(|_| state.window.set_cursor_grab(winit::window::CursorGrabMode::Locked));

                            println!("FPS mode enabled - Mouse locked");
                        } else {
                            // 解锁并显示鼠标
                            state.window.set_cursor_visible(true);
                            let _ = state.window.set_cursor_grab(winit::window::CursorGrabMode::None);

                            println!("FPS mode disabled - Mouse unlocked");
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }


}

// 启动，从外部传入数据
pub fn run() -> anyhow::Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        init_logger()
    }
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}


pub fn init_logger(){
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()  // 仍然允许 RUST_LOG 环境变量覆盖
        .init();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();
    Ok(())
}

#[wasm_bindgen]
pub struct Commander{}

// ==================== wasm_bindgen JavaScript API ====================

#[wasm_bindgen]
impl Commander {
    // 切换场景
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn change_scene(path: String) {
        web_sys::console::log_1(&format!("📝 Queuing scene change: {}", path).into());

        if let Ok(mut queue) = COMMAND_QUEUE.lock() {
            queue.push(SceneCommand::ChangeScene { path });
            web_sys::console::log_1(&"✓ Scene change queued".into());
        } else {
            web_sys::console::error_1(&"✗ Failed to queue scene change".into());
        }
    }


    /// 设置场景路径（wasm 环境使用）
    /// 必须在 run_web() 之前调用
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn set_scene_path(path: String) {
        SCENE_PATH.get_or_init(|| path);
    }

    /// 设置相机位置
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn set_camera_position_bridge(x: f32, y: f32, z: f32) {
        if let Ok(mut queue) = COMMAND_QUEUE.lock() {
            queue.push(SceneCommand::SetCameraPosition { x, y, z });
        }
    }

    /// 设置相机目标
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn set_camera_target_bridge(x: f32, y: f32, z: f32) {
        if let Ok(mut queue) = COMMAND_QUEUE.lock() {
            queue.push(SceneCommand::SetCameraTarget { x, y, z });
        }
    }


    /// 获取相机位置
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn get_camera_position_bridge() -> js_sys::Float32Array {
        if let Ok(results) = QUERY_RESULTS.lock() {
            js_sys::Float32Array::from(&results.camera_position[..])
        } else {
            js_sys::Float32Array::new_with_length(3)
        }
    }

    /// 获取相机目标
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn get_camera_target_bridge() -> js_sys::Float32Array {
        if let Ok(results) = QUERY_RESULTS.lock() {
            js_sys::Float32Array::from(&results.camera_target[..])
        } else {
            js_sys::Float32Array::new_with_length(3)
        }
    }

    /// 获取实体总数
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn scene_get_entity_count() -> usize {
        if let Ok(results) = QUERY_RESULTS.lock() {
            results.entity_count
        } else {
            0
        }
    }

    /// 获取可见实体数量
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn scene_get_visible_count() -> usize {
        if let Ok(results) = QUERY_RESULTS.lock() {
            results.visible_count
        } else {
            0
        }
    }

    /// 获取当前场景路径
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen]
    pub fn scene_get_current_scene() -> String {
        if let Ok(results) = QUERY_RESULTS.lock() {
            results.current_scene.clone()
        } else {
            String::new()
        }
    }
}


/// 获取场景路径（内部使用）
fn get_scene_path() -> String {
    // 优先级：
    // 1. localStorage 中的pending_scene_path (wasm 场景切换后)
    // 2. 全局变量（wasm 通过 set_scene_path 设置）
    // 3. 环境变量（本地环境）
    // 4. 默认值

    #[cfg(target_arch = "wasm32")]
    {
        // 检查 localStorage 中是否有待加载的场景
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                web_sys::console::log_1(&"Checking localStorage for pending_scene_path...".into());

                match storage.get_item("pending_scene_path") {
                    Ok(Some(pending_path)) => {
                        web_sys::console::log_1(&format!("✓ Found pending scene in localStorage: {}", pending_path).into());

                        // 清除 localStorage 中的路径（避免重复加载）
                        let _ = storage.remove_item("pending_scene_path");
                        info!("Loading scene from localStorage: {}", pending_path);
                        return pending_path;
                    },
                    Ok(None) => {
                        web_sys::console::log_1(&"No pending scene in localStorage".into());
                    },
                    Err(e) => {
                        web_sys::console::error_1(&format!("Error reading localStorage: {:?}", e).into());
                    }
                }
            } else {
                web_sys::console::error_1(&"localStorage not available".into());
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    dotenv::dotenv().ok();

    let final_path = SCENE_PATH.get()
        .map(|s| s.clone())
        .or_else(|| std::env::var("SCENE_PATH").ok())
        .unwrap_or_else(|| "Scenes/Level_StormZone/Level_StormZone_B4.unity".to_string());

    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("get_scene_path returning: {}", final_path).into());

    final_path
}