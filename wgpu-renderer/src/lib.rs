mod camera;
mod light;
mod materials;
mod resource;
mod scene;
mod unity;
mod utils;
mod entity;

use log::info;
use std::fs;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::resource::load_binary;
use crate::scene::{Scene};
use crate::unity::UnityScene;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::VertexFormat;
use wgpu::util::RenderEncoder;
use winit::window::WindowId;
use crate::entity::{IVertex, Mesh, Model, Vertex};

pub struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub scene: Scene,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,

    // 顶点buffer，可以通过set方法写入数据
    vertex_buffer: wgpu::Buffer,
    // 顶点索引数据
    index_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    // scene: UnityScene
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

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

        let features = adapter.features();
        if !features.contains(wgpu::Features::SHADER_F16) {
            panic!("设备不支持 SHADER_F16 特性");
        }

        info!(
            "Initializing WGPU: max_texture_dimension_2d {:?}",
            adapter.limits().max_texture_dimension_2d
        );
        // 通过适配器获取device以及queue(类似管线队列)
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::SHADER_F16, // 启用 f16 支持,
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits {
                        max_texture_dimension_2d: 4096, // 尝试请求 4096，看 WebGL2 后端是否能支持
                        ..wgpu::Limits::downlevel_webgl2_defaults()  // ..Default::default()
                    }
                } else {
                    wgpu::Limits::default()
                },
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: Default::default(),
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

        let bytes = load_binary("SM_FabricScroll_1.asset").await.map_err(|e| {
            println!("SM_Table_01_2.asset: {:?}", e);
            e
        })?;

        let asset = Mesh::from_asset(&bytes, &device, &queue)?;

        // println!("asset2: {:?}", asset);
    
        let mut scene = Scene::new(&device, &config);
        
        scene.add_model(Model{
            id: 0,
            name: "first".to_string(),
            meshs: vec![
                asset.clone()
            ]
        });
        
        let render_pipeline = Self::create_render_pipeline(&device, &config, &scene);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            vertex_buffer: asset.vertex_buffer,
            index_buffer: asset.index_buffer,
            render_pipeline,
            scene,
        })
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        scene: &Scene,
    ) -> wgpu::RenderPipeline {
        
        let depth_stencil = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,  // 注意这里
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let primitive = wgpu::PrimitiveState {
            // 设置3点成面
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            // cull_mode: Some(wgpu::Face::Back),
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        };

        let multisample = wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                // 相机
                &scene.camera.bind_group_layout,
                // 环境光 & 背景色
                &scene.scene_bind_group_layout,
                // 光照
                &scene.light_manager.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            primitive,
            depth_stencil: None,
            multisample,
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
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
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            _render_pass.set_pipeline(&self.render_pipeline);
            _render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // print!("self.index_buffer: {:?}", self.index_buffer);
            _render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            
            self.scene.render(&mut _render_pass);
            
            // _render_pass.draw_indexed(0..768, 0, 0..1);
            // _render_pass.draw(0..268, 0..1);

        }

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
            _ => {}
        }
    }
}

// 启动，从外部传入数据
pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
