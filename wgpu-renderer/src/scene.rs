use std::time::Instant;
use wgpu::{Device};
use crate::camera::Camera;
use crate::entity::{Entity, Model};
use crate::light::{DirectionalLight, LightManager, PointLight};


pub struct Scene {
    pub light_manager: LightManager,
    pub camera: Camera,

    pub models: Vec<Model>,
    pub entities: Vec<Entity>,

    // 环境光
    pub ambient_light: [f32; 3],
    pub background_color: wgpu::Color,

    pub scene_bind_group_layout: wgpu::BindGroupLayout,
    scene_uniform_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,

    pub elapsed_time: f32,

}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SceneUniforms { // 场景的uniforms 与gpu 通信
    pub ambient_light: [f32; 3],
    pub ambient_intensity: f32,  // 替代 _padding1
    pub fog_color: [f32; 3],
    pub fog_density: f32,
}

impl Scene {
    pub fn new(device: &Device, config: &wgpu::SurfaceConfiguration) -> Scene {
        let light_manager = LightManager::new(device);
        let camera = Camera::new(device, config.width as f32 / config.height as f32);

        let ambient_light = [0.1;3];

        let scene_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Scene Uniform Buffer"),
            size: std::mem::size_of::<SceneUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let scene_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some("Scene Bind Group"),
            layout: &scene_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: scene_uniform_buffer.as_entire_binding(),
                }
            ],
        });

        Scene{
            light_manager,
            camera,
            ambient_light,
            background_color: wgpu::Color{
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            scene_uniform_buffer,
            scene_bind_group,
            scene_bind_group_layout,

            models:Vec::new(),
            entities:Vec::new(),
            elapsed_time: Instant::now().elapsed().as_secs_f32(),
        }
    }
    
    pub fn add_model(&mut self, model: Model) {
        self.entities.push(Entity::new(model.id));
        self.models.push(model);
    }

    // 初始化设置环境光等
    pub fn setup(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.light_manager.add_point_light(PointLight{
            position: [5.0, 5.0, 5.0],
            _padding1: 0.0,
            color: [1.0, 0.8, 0.6],
            intensity: 2.0,
            radius: 15.0,
            _padding2: [0.0; 3],
        });
        self.light_manager.add_directional_light(DirectionalLight{
            direction: [-0.3, -1.0, -0.5],
            _padding1: 0.0,
            color: [1.0, 1.0, 0.95],
            intensity: 1.0,
        })
    }

    // 更新-> 通过queue写入gpu buffer
    pub fn update(&mut self, queue: &wgpu::Queue, delta_time: f32) {
        // 更新相机
        self.camera.update(queue, 0.0);

        // 更新光照
        self.light_manager.update_buffers(queue);

        // 更新场景数据 比如环境光, fog颜色
        let scene_uniforms = SceneUniforms{
            ambient_light: self.ambient_light,
            ambient_intensity: 0.5,
            fog_color: [0.5, 0.6, 0.7],
            fog_density: 0.0,
        };
        
        // scene_uniform_buffer 理解是一个管道buffer
        queue.write_buffer(&self.scene_uniform_buffer, 0, bytemuck::bytes_of(&scene_uniforms));
        
        // 渲染所有的models数据
        for entity in &mut self.entities {
            entity.update(delta_time);
        }
        
        self.elapsed_time += delta_time;
    }

    // fn update_dynamic_lighting(&mut self, delta_time: f32) {
    //     // 昼夜循环：24 小时 = 24 秒（加速）
    //     let time_of_day = (self.elapsed_time % 24.0) / 24.0;
    // 
    //     // 白天：暖色调，强度高
    //     // 夜晚：冷色调，强度低
    //     if let Some(sun) = self.light_manager.directional_lights.get_mut(0) {
    //         let intensity = ((time_of_day * std::f32::consts::TAU).cos() + 1.0) / 2.0;
    //         sun.intensity = intensity * 0.8 + 0.2; // 0.2 到 1.0
    // 
    //         // 调整颜色
    //         if time_of_day < 0.25 || time_of_day > 0.75 {
    //             // 夜晚：蓝色调
    //             sun.color = [0.6, 0.7, 1.0];
    //         } else {
    //             // 白天：暖色调
    //             sun.color = [1.0, 0.95, 0.8];
    //         }
    //     }
    // }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // 设置渲染管线- 动态管线
        // bind_group全局资源
        render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
        render_pass.set_bind_group(1, &self.scene_bind_group, &[]);
        render_pass.set_bind_group(2, &self.light_manager.bind_group, &[]);

        // 渲染实体
        for entity in &self.entities {
            if let Some(model) = self.models.get(entity.model_id) {
                entity.render(render_pass, model);
            }
        }
    }
}