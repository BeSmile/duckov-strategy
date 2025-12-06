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

use log::{error, info, warn};
use std::path::PathBuf;
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
use crate::entity::Entity;
use crate::materials::{Texture};
use crate::ray::Ray;
use crate::unity::UnityScene;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

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
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();


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

        info!("Initializing WGPU: {:?}", size.width);

        let surface = instance.create_surface(window.clone()).unwrap();


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

        info!(
            "Initializing WGPU: max_texture_dimension_2d {:?}",
            adapter.limits().max_texture_dimension_2d
        );
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
        let path = PathBuf::from("Scenes/Level_JLab/Level_JLab_2.unity");

        let time = Instant::now();
        info!("Start parse Scene, times: {}", time.elapsed().as_millis());

        // 在 wasm 环境下使用 worker，在非 wasm 环境下使用正常方法
        let mut unity_scene = {
            let mut uns = UnityScene::new();
            uns.from_str(path.clone()).await.map_err(|e| {
                error!("Failed to load unity scene: {:?}", e);
                e
            })?
        };
        info!("parse Scene done, current time: {}", time.elapsed().as_millis());

        let mut scene = Scene::new(&device, &config, unity_scene.game_object_raw.len() * 2);
        let mut resource_manager = ResourceManager::new(&device, &queue);
        resource_manager.loading_mapping().await.map_err(| e| {
            error!("Failed to load resource mapping: {:?}", e);
            e
        })?;

        info!("loading_scene, current time: {}", time.elapsed().as_millis());

        Scene::loading_scene(&device, &queue, &mut scene, &mut unity_scene, &mut resource_manager, &config).await.map_err(|e| {
            error!("Failed to load scene scene: {:?}", e);
            e
        })?;

        info!("loading_scene done, current time: {}", time.elapsed().as_millis());


        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            // vertex_buffer: asset.vertex_buffer,
            // index_buffer: asset.index_buffer,
            // render_pipeline,
            scene,
            resource_manager,
            depth_texture: None,
            mouse_pos: (0.0, 0.0),
        })
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

    fn update(&mut self) {
        self.scene.update(&self.queue, 1.0)
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

        {
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
            );
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
    state: Option<State>,
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

            self.state = Some(state);
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
        // This is where proxy.send_event() ends up
        #[cfg(target_arch = "wasm32")]
        {
            // 设置window的宽高
            event.window.request_redraw();
            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height,
            );
        }
        self.state = Some(event);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let state = match &mut self.state {
            Some(s) => s,
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
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

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
