use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3, Vector4};
use wgpu::{Device, Queue, SurfaceConfiguration};

use crate::camera::Camera;
use crate::entity::{Entity, Mesh, Model, Transform, TransformSystem};
use crate::light::{DirectionalLight, LightManager, PointLight};
use crate::materials::{Material, Texture};
use crate::resource::load_binary;
use crate::utils::get_background_color;

pub type PipelineId = String;

pub struct PipelineManager {
    pipelines: HashMap<PipelineId, wgpu::RenderPipeline>,
    current_pipeline: Option<PipelineId>,
}

impl PipelineManager {
    pub fn new() -> Self {
        Self{
            pipelines: Default::default(),
            current_pipeline: None,
        }
    }
}

#[derive(Debug)]
pub struct ResourceManager {
    materials: HashMap<Entity, Arc<Material>>,
    meshes: HashMap<Entity, Arc<Mesh>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self{
            materials: Default::default(),
            meshes: Default::default(),
        }
    }

    pub async fn load_mesh(&mut self, bytes: &[u8], entity: Entity, device: &Device, scene: &Scene, material: &Material, config: &SurfaceConfiguration) -> anyhow::Result<u32> {
        let mesh = Mesh::from_unity_data(&bytes, device, scene, material, config).await?;// 车轮子
        // let id = self.meshes.len() as u32;
        self.meshes.insert(entity, Arc::new(mesh));

        Ok(entity.id())
    }

    // 加载mat资源材质包，暂时使用实体的Id
    pub async fn load_material(&mut self, entity: Entity, device: &Device, queue: &Queue) -> anyhow::Result<u32> {
        let mat_bytes = load_binary("MAT_ElectricControlBox.mat").await.map_err(|e| {
            println!("Load mat asset error: {:?}", e);
            e
        })?;

        // 后续处理多布局layout的问题, 可能共用mesh, 会有优化部分, 先使用entity_id
        let material = Material::from_unity_bytes(&mat_bytes, &device, &queue).await?;
        self.materials.insert(entity, Arc::new(material));

        Ok(entity.id())
    }

    pub fn get_material(&self, entity: Entity) -> Option<&Arc<Material>> {
        self.materials.get(&entity)
    }

    pub fn get_mesh(&self, entity: Entity) -> Option<&Arc<Mesh>> {
        self.meshes.get(&entity)
    }
}

pub struct Scene {
    pub light_manager: LightManager,
    pub camera: Camera,

    // 环境光
    pub ambient_light: [f32; 3],
    pub background_color: wgpu::Color,

    pub entities: Vec<Entity>,// 存档所有的实体类key

    pub scene_bind_group_layout: wgpu::BindGroupLayout,
    scene_uniform_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,

    pub elapsed_time: f32,
    pub pipeline_manager: PipelineManager,
    pub depth_texture: Texture,

    // 存储所有的transform变换数据
    pub transform_system: TransformSystem,

    // 存放所有的transform buffer数据
    transforms_uniform_buffer: wgpu::Buffer,
    // transform数据的bind_group
    pub transform_bind_group: wgpu::BindGroup,
    pub transform_bind_group_layout: wgpu::BindGroupLayout,

    entity_offsets: HashMap<Entity, u32>,
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

        let ambient_light = [0.9;3];

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

        // ⭐ 一次性更新所有 transform 到 buffer
        let max_entities = 1000;
        let aligned_size = Self::aligned_uniform_size(
            size_of::<Matrix4<f32>>() as u64
        );

        let transforms_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("Transforms Buffer"),
            size: aligned_size * max_entities,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let transform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("Transform Bind Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,// 动态偏移
                        min_binding_size: Some(
                            std::num::NonZeroU64::new(
                                std::mem::size_of::<Matrix4<f32>>() as u64
                            ).unwrap()
                        ),
                    },
                    count: None,
                }
            ],
        });

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some("Transform Bind Group"),
            layout: &transform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding{
                        buffer: &transforms_uniform_buffer,
                        offset: 0,
                        size: Some(
                            std::num::NonZeroU64::new(
                                std::mem::size_of::<Matrix4<f32>>() as u64
                            ).unwrap()
                        ),
                    }),
                }
            ],
        });


        Scene{
            light_manager,
            camera,
            ambient_light,
            background_color: get_background_color(),
            scene_uniform_buffer,
            scene_bind_group,
            scene_bind_group_layout,

            entities: Vec::new(),
            elapsed_time: Instant::now().elapsed().as_secs_f32(),
            pipeline_manager: PipelineManager::new(),
            // materials: HashMap::new(),
            depth_texture: Texture::create_depth_texture(device,config, "depth_texture"),
            transform_system: TransformSystem::new(),
            transform_bind_group,
            transform_bind_group_layout,
            transforms_uniform_buffer,
            entity_offsets: HashMap::new(),
        }
    }

    // 计算对齐后的 uniform 大小（必须是 256 的倍数）
    fn aligned_uniform_size(size: u64) -> u64 {
        let alignment = 256; // wgpu 要求
        (size + alignment - 1) & !(alignment - 1)
    }

    pub async fn loading_scene(device: &Device, queue: &Queue, scene: &mut Scene, resource_manager: &mut ResourceManager, config: &wgpu::SurfaceConfiguration) -> anyhow::Result<()> {
        // 加载车身体
        let bytes = load_binary("Car_15.asset").await.map_err(|e| {
            println!("SM_Table_01_2.asset: {:?}", e);
            e
        })?;
        let wheel_bytes_1 = load_binary("Car_15_Wheel_1.asset").await.map_err(|e| {
            println!("SM_Table_01_2.asset: {:?}", e);
            e
        })?;
        let wheel_bytes_2 = load_binary("Car_15_Wheel_2.asset").await.map_err(|e| {
            println!("SM_Table_01_2.asset: {:?}", e);
            e
        })?;
        let wheel_bytes_3 = load_binary("Car_15_Wheel_3.asset").await.map_err(|e| {
            println!("SM_Table_01_2.asset: {:?}", e);
            e
        })?;
        // Mat_Car_06_1 车轮胎材质球
        // Car_15_Wheel_1,2,3.asset mesh

        let entity = Entity::new(122);
        let entity_wheel_1 = Entity::new(123);
        let entity_wheel_2 = Entity::new(124);
        let entity_wheel_3 = Entity::new(125);

        // 实体类和模型的区分， 一个实体类可以是同一个车模型，但是贴图不一致，以及transform的变换
        // let asset = Mesh::from_unity_data(&bytes, entity , device, queue, scene, config).await?;// 车主体资源
        let m_id = resource_manager.load_material(entity, device, queue).await?;

        let m_id = resource_manager.load_material(entity_wheel_1, device, queue).await?;
        let m_id = resource_manager.load_material(entity_wheel_2, device, queue).await?;
        let m_id = resource_manager.load_material(entity_wheel_3, device, queue).await?;

        // let mut resource_manager = &scene.resource_manager;
        let material = {
            println!("M_Table_01_2.asset: {:?}", m_id);
            Arc::clone(resource_manager.get_material(entity).unwrap())
        };

        resource_manager.load_mesh(&bytes, entity, device, scene, &material, config).await?;
        resource_manager.load_mesh(&wheel_bytes_1, entity_wheel_1, device, scene, &*material, config).await?;// 车轮子
        resource_manager.load_mesh(&wheel_bytes_2, entity_wheel_2, device, scene, &*material, config).await?;// 车轮子
        resource_manager.load_mesh(&wheel_bytes_3, entity_wheel_3, device, scene, &*material, config).await?;// 车轮子

        let car_bu = 0;

        let mut car_transform = Transform::new();
        // car_transform.set_position(Vector3::from([-86.82, 169.88, -0.48]));
        car_transform.set_position(Vector3::from([1.0, 1.0, 0.0]));

        // 创建车轮子entity
        let mut entity_transform_wheel_1 = Transform::new();
        entity_transform_wheel_1.set_position(Vector3::from([0.0, 0.5279994, -1.712251]));

        let mut entity_transform_wheel_2 = Transform::new();
        entity_transform_wheel_2.set_position(Vector3::from([0.0, 0.5279994, 2.337084]));

        let mut entity_transform_wheel_3 = Transform::new();
        entity_transform_wheel_3.set_position(Vector3::from([0.0, 0.5279994, 3.448621]));

        scene.transform_system.set_parent(entity, entity_wheel_1);
        scene.transform_system.set_parent(entity, entity_wheel_2);
        scene.transform_system.set_parent(entity, entity_wheel_3);
        
        scene.add_entity(entity, car_transform);
        scene.add_entity(entity_wheel_1, entity_transform_wheel_1);
        scene.add_entity(entity_wheel_2, entity_transform_wheel_2);
        scene.add_entity(entity_wheel_3, entity_transform_wheel_3);

        scene.transform_system.update();

        Ok(())
    }

    pub fn add_entity(&mut self, entity: Entity, transform: Transform) {
        self.entities.push(entity);
        self.transform_system.add_transform(entity, transform);
    }

    pub fn add_pipelines(&mut self, pipeline_id: PipelineId, pipeline: wgpu::RenderPipeline) {
        self.pipeline_manager.pipelines.insert(pipeline_id, pipeline);
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
            ambient_intensity: 0.2,
            fog_color: [0.5, 0.6, 0.7],
            fog_density: 0.0,
        };

        // scene_uniform_buffer 理解是一个管道buffer
        queue.write_buffer(&self.scene_uniform_buffer, 0, bytemuck::bytes_of(&scene_uniforms));

        // ⭐ 一次性更新所有 transform 到 buffer
        let aligned_size = Self::aligned_uniform_size(
            std::mem::size_of::<Matrix4<f32>>() as u64
        ) as u32;

        for (index, &entity) in self.entities.iter().enumerate() {
            if let Some(matrix) = self.transform_system.get_world_matrix(entity) {
                let matrix_array: [[f32; 4]; 4] = matrix.into();
                // println!("entity {:?}", matrix_array);
                let offset = index as u32 * aligned_size;
                // 固定的偏移量
                self.entity_offsets.insert(entity, offset);

                queue.write_buffer(
                    &self.transforms_uniform_buffer,
                    offset as u64,
                    bytemuck::cast_slice(&[matrix_array])
                );
            }
        }

        // 渲染所有的transform
        // for entity in &mut self.entities {
        //     entity.update(delta_time);
        // }

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

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, resource_manager: &'a ResourceManager) {
        // println!("resource_manager: {:#?}", &resource_manager);
        // 渲染实体
        for entity in &self.entities {
            // 从资源管理器获取 mesh
            let Some(mesh) = resource_manager.get_mesh(*entity) else {
                println!("Entity does not exist for {:?}", entity);
                continue; // 没有 mesh 就跳过
            };


            // 从资源管理器获取 material
            let Some(material) = resource_manager.get_material(*entity) else {
                println!("Material does not exist for {:?}", entity);
                continue;
            };

            // let Some(world_matrix) = self.transform_system.get_world_matrix(*entity) else {
            //     println!("World matrix does not exist for {:?}", entity);
            //     continue;
            // };
            let Some(&offset) = self.entity_offsets.get(&entity) else {
                continue
            };

            // 绑定模型特定的资源并渲染
            render_pass.set_pipeline(&mesh.render_pipeline);
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // 设置渲染管线- 动态管线
            // bind_group全局资源
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(1, &self.scene_bind_group, &[]);
            // render_pass.set_bind_group(2, &self.light_manager.bind_group, &[]);
            render_pass.set_bind_group(2, &self.transform_bind_group, &[offset]);

            render_pass.set_bind_group(3, &material.bind_group, &[]);

            // 创建pipeline 布局等等，设置buffer之类
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}