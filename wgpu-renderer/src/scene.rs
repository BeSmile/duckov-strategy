use std::collections::HashMap;
use std::sync::Arc;

use cgmath::{Matrix4, Vector3, Vector4};
use wgpu::{Device, Queue, SurfaceConfiguration};

use crate::camera::Camera;
use crate::entity::{Entity, InstanceRaw, Transform, TransformSystem};
use crate::light::{DirectionalLight, LightManager, PointLight};
use crate::materials::{Texture};
use crate::resource::{MaterialId, MeshId, ResourceManager};
use crate::utils::get_background_color;

pub type PipelineId = String;

use crate::unity::{Component, UnityMeshFilter, UnityMeshRenderer, UnityScene};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use wgpu::util::DeviceExt;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
use crate::ray::Ray;

pub struct PipelineManager {
    pipelines: HashMap<PipelineId, wgpu::RenderPipeline>,
    current_pipeline: Option<PipelineId>,
}

impl PipelineManager {
    pub fn new() -> Self {
        Self {
            pipelines: Default::default(),
            current_pipeline: None,
        }
    }
}

pub struct Scene {
    pub light_manager: LightManager,
    pub camera: Camera,

    // 环境光
    pub ambient_light: [f32; 3],
    pub background_color: wgpu::Color,

    pub entities: Vec<Entity>, // 存档所有的实体类key
    entity_display_map: HashMap<Entity, bool>, // true显示 false不显示

    pub scene_bind_group_layout: wgpu::BindGroupLayout,
    scene_uniform_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,

    pub elapsed_time: f32,
    pub pipeline_manager: PipelineManager,

    // 存储所有的transform变换数据
    pub transform_system: TransformSystem,
    // 批量更新系统
    pub render_batches: RenderBatchSystem,

    // 存放所有的transform buffer数据
    // transforms_uniform_buffer: wgpu::Buffer,
    // transform数据的bind_group
    // pub transform_bind_group: wgpu::BindGroup,
    // pub transform_bind_group_layout: wgpu::BindGroupLayout,
    entity_offsets: HashMap<Entity, u32>,
}

// render一次批量
struct RenderBatch {
    pub mesh_id: MeshId,
    pub material_id: MaterialId,
    pub entities: Vec<Entity>,                 // 属于这个批次的entities
    pub instance_buffer: Option<wgpu::Buffer>, // 延迟创建
}

pub struct RenderBatchSystem {
    // 按(mesh_id, material_id)分组的渲染批次
    batches: HashMap<(MeshId, MaterialId), RenderBatch>,
    // 是否需要重建批次(当entity增删或组件变化时)
    dirty: bool,
}

impl Default for RenderBatchSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderBatchSystem {
    pub fn new() -> Self {
        Self {
            batches: Default::default(),
            dirty: false,
        }
    }

    // 标记为需要重建
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn update_instance_buffers(
        &mut self,
        device: &Device,
        transform_system: &TransformSystem,
        // camera: &Camera,
        // enable_frustum_culling: bool,
    ) {
        // 更新instance_buffers

        for batch in self.batches.values_mut() {
            let instances: Vec<InstanceRaw> = batch.entities.iter().filter_map(|&entity| {
                let transform =  transform_system.get_world_matrix(entity);
                match transform {
                    Some(ts) => Some(InstanceRaw{
                        model: ts.into()
                    }),
                    None => {
                        println!("entity: {:?} not transofm", entity);
                        None
                    }
                }

            }).collect();

            if instances.is_empty() {
                batch.instance_buffer = None; // 全部被剔除
                continue;
            }
            // 创建/更新instance buffer
            batch.instance_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ));
        }
    }

    pub fn rebuild_batches(
        &mut self,
        entities: &[Entity],
        entity_display_map: &HashMap<Entity, bool>,
        resource_manager: &ResourceManager,
    ) {
        self.batches.clear();

        for entity in entities {
            if !entity_display_map.get(&entity).unwrap_or(&true) {
                continue;
            }
            let Some(mesh) = resource_manager.get_mesh(entity) else {
                continue;
            };
            let Some(material) = resource_manager.get_material(entity) else {
                continue;
            };
            // let id = mesh.id.clone();
            let key = (mesh.id.clone(), material.id.clone());
            self.batches
                .entry(key)
                .or_insert_with(|| RenderBatch {
                    mesh_id: mesh.id.clone(),
                    material_id: material.id.clone(),
                    entities: Vec::new(),
                    instance_buffer: None,
                })
                .entities
                .push(entity.clone());
        }
        self.dirty = true;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SceneUniforms {
    // 场景的uniforms 与gpu 通信
    pub ambient_light: [f32; 3],
    pub ambient_intensity: f32, // 替代 _padding1
    pub fog_color: [f32; 3],
    pub fog_density: f32,
}

impl Scene {
    pub fn new(device: &Device, config: &SurfaceConfiguration, max_entities: usize) -> Scene {
        let light_manager = LightManager::new(device);
        let camera = Camera::new(device, config.width as f32 / config.height as f32);

        let ambient_light = [0.9; 3];

        let scene_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene Uniform Buffer"),
            size: size_of::<SceneUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Scene Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene Bind Group"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniform_buffer.as_entire_binding(),
            }],
        });

        let alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        println!("Real alignment required: {}", alignment); // 在 macOS 上打印看看
        // ⭐ 一次性更新所有 transform 到 buffer
        // let max_entities = 100000;// 数据连续： 优化部分，transform 高频变化数据，直接通过offset进行数据的写入以及更新，利用entity的is_dirty进行管理是否更新
        let aligned_size = Self::aligned_uniform_size(size_of::<Matrix4<f32>>() as u64);

        println!(
            "max_entities: {}, aligned_size: {}",
            max_entities,
            aligned_size * max_entities as u64
        );
        // let transforms_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Transforms Buffer"),
        //     size: aligned_size * max_entities as u64,
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        // let transform_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         label: Some("Transform Bind Layout"),
        //         entries: &[wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::VERTEX,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: true, // 动态偏移
        //                 min_binding_size: Some(
        //                     std::num::NonZeroU64::new(std::mem::size_of::<Matrix4<f32>>() as u64)
        //                         .unwrap(),
        //                 ),
        //             },
        //             count: None,
        //         }],
        //     });

        // let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("Transform Bind Group"),
        //     layout: &transform_bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        //             buffer: &transforms_uniform_buffer,
        //             offset: 0,
        //             size: Some(
        //                 std::num::NonZeroU64::new(std::mem::size_of::<Matrix4<f32>>() as u64)
        //                     .unwrap(),
        //             ),
        //         }),
        //     }],
        // });

        Scene {
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
            transform_system: TransformSystem::new(),
            // transform_bind_group,
            // transform_bind_group_layout,
            // transforms_uniform_buffer,
            entity_offsets: HashMap::new(),
            entity_display_map: HashMap::new(),
            render_batches: RenderBatchSystem::default(),
        }
    }

    pub fn pick_entity(&self, ray: &Ray, resource_manager: &ResourceManager) -> Option<(u32, f32)> {
        let mut closest: Option<(u32, f32)> = None;
        for entity in &self.entities {
            let Some(mesh) = resource_manager.get_mesh(entity) else {
                continue;
            };
            let Some(transform) = self.transform_system.get_world_matrix(*entity) else {
                continue;
            };
            // println!("entity: {} mesh: {:?}", entity.id(), mesh.aabb);
            // Transform AABB to world space (simplified: just offset by position)
            let world_aabb = mesh.aabb.transform(&transform);


            if let Some(distance) = ray.intersect_aabb(world_aabb.min, world_aabb.max) {
                match closest {
                    None => closest = Some((entity.id(), distance)),
                    Some((_, d)) if distance < d => closest = Some((entity.id(), distance)),
                    _ => {}
                }
            }
        }

        closest
    }

    // 计算对齐后的 uniform 大小（必须是 256 的倍数）
    fn aligned_uniform_size(size: u64) -> u64 {
        let alignment = 256; // wgpu 要求
        (size + alignment - 1) & !(alignment - 1)
    }

    pub async fn loading_scene(
        device: &Device,
        queue: &Queue,
        scene: &mut Scene,
        unity_scene: &mut UnityScene,
        resource_manager: &mut ResourceManager,
        config: &SurfaceConfiguration,
    ) -> anyhow::Result<()> {
        let indexs = &unity_scene.index; // 查看类型
        let objects = &unity_scene.game_object;
        let transforms = &unity_scene.transforms;
        let mesh_renders = &unity_scene.mesh_renderers;
        let mesh_filters = &unity_scene.mesh_filters;
        let test_id = 19022;
        for (entity_id, game_object) in objects {
            // 查看挂载的transform
            let entity = Entity::new(*entity_id);
            if game_object.m_is_active != 1 {
                scene.hidden_entity(entity);
            }

            if *entity_id == test_id {
                println!("entity: {:?}", entity);
            }
            let mut is_light = false;
            let mut unity_mesh_render: Option<&UnityMeshRenderer> = None;
            let mut unity_mesh_filter: Option<&UnityMeshFilter> = None;

            let mut local_transform = Transform::new();

            for m_component in &game_object.m_component {
                let file_id = m_component.component.file_id.clone();
                let s_type = indexs.get(&file_id);
                match s_type {
                    // transform管理
                    Some(s) if s.as_str() == "UnityTransform" => {
                        let transform = transforms.get(&file_id);
                        let op = transform.as_deref();

                        let Some(unity_transform) = op else {
                            continue;
                        };
                        if *entity_id == test_id {
                            println!("{}: transform: {:?}, gameobject : {:?}", *entity_id, op, &game_object);
                        }
                        // 通过transform查找children上的transform数据，transform对应;
                        local_transform.set_position(&unity_transform.m_local_position);
                        let unity_rot = &unity_transform.m_local_rotation;
                        local_transform.set_rotation(cgmath::Quaternion::new(
                            -unity_rot.w, // w 取负
                            unity_rot.x,  // x 保持
                            unity_rot.y,  // y 保持
                            -unity_rot.z, // z 取负
                        ));
                        let unity_scale = &unity_transform.m_local_scale;
                        local_transform.set_scale(Vector3::new(
                            unity_scale.x,
                            unity_scale.y,
                            unity_scale.z,
                        ));

                        // 根据children设置父子关系
                        // for m_child in &unity_transform.m_children {
                        //     scene.transform_system.set_parent(Entity::new(file_id), Entity::new(m_child.file_id));
                        // }
                        if let Some(m) = &unity_transform.m_father {
                            let parent_transform = transforms.get(&m.file_id);

                            if let Some(transform) = parent_transform {
                                scene.transform_system.set_parent(
                                    Entity::new(transform.m_game_object.file_id),
                                    Entity::new(unity_transform.m_game_object.file_id),
                                );
                            }
                        }
                        scene.entities.push(entity);
                    }
                    Some(s) if s.as_str() == "UnityMeshRenderer" => {
                        unity_mesh_render = mesh_renders.get(&file_id);
                    }
                    // 顶点数据
                    Some(s) if s.as_str() == "UnityMeshFilter" => {
                        unity_mesh_filter = mesh_filters.get(&file_id);
                    }
                    Some(s)
                        if s.as_str() == "UnityLight" || s.as_str() == "UnitySodaPointLight" =>
                    {
                        is_light = true;
                    }
                    _ => {}
                }
                // println!("loading unity 顶点数据: {:?}", unity_mesh_filter);
            }
            if *entity_id == test_id {
                println!("entity: {:?} End Ground", entity);
            }
            // 光照暂时不渲染
            if is_light {
                continue;
            }

            scene.add_entity(entity, local_transform);
            
            let Some(mesh_filter) = unity_mesh_filter else {
                continue;
            };
            let Some(mesh_mesh_reference) = unity_mesh_render else {
                continue;
            };
            // mesh隐藏不加载，地图有隐藏提前加载的物品是unity场景优化部分,存在顶部mesh不渲染，子gameobject渲染，但是实际同一个材质
            // if mesh_mesh_reference.m_enabled == 0u8 {
            //     scene.hidden_entity(entity);
            //     println!("entity mesh_renderer {} 隐藏: {:?}", game_object.m_name, entity);
            //     continue;
            // }
            // 材质球
            let Some(mesh_render) = mesh_mesh_reference.m_children.get(0) else {
                continue;
            };

            resource_manager
                .load_material(entity, &mesh_render.guid, device, queue)
                .await?;

            let material = { Arc::clone(resource_manager.get_material(&entity).unwrap()) };

            resource_manager
                .load_mesh(
                    &mesh_filter.m_mesh,
                    entity,
                    device,
                    scene,
                    &material,
                    config,
                )
                .await?;
        }

        scene.transform_system.update(&mut scene.entity_display_map);
        scene.render_batches.rebuild_batches(
            &scene.entities,
            &scene.entity_display_map,
            resource_manager,
        );
        println!("scene rendered all {} entities", scene.entities.len());
        println!("scene actually rendered all {} entities", scene.total_show_entities());

        Ok(())
    }

    pub fn add_entity(&mut self, entity: Entity, transform: Transform) {
        self.entities.push(entity);
        self.transform_system.add_transform(entity, transform);
    }

    pub fn hidden_entity(&mut self, entity: Entity) {
        self.entity_display_map.insert(entity, false);
    }
    
    // 查看是否显示
    pub fn is_display(&self, entity: &Entity) -> &bool {
        self.entity_display_map.get(entity).unwrap_or(&true)
    }

    pub fn add_pipelines(&mut self, pipeline_id: PipelineId, pipeline: wgpu::RenderPipeline) {
        self.pipeline_manager
            .pipelines
            .insert(pipeline_id, pipeline);
    }

    // 初始化设置环境光等
    pub fn setup(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.light_manager.add_point_light(PointLight {
            position: [5.0, 5.0, 5.0],
            _padding1: 0.0,
            color: [1.0, 0.8, 0.6],
            intensity: 2.0,
            radius: 15.0,
            _padding2: [0.0; 3],
        });
        self.light_manager.add_directional_light(DirectionalLight {
            direction: [-0.3, -1.0, -0.5],
            _padding1: 0.0,
            color: [1.0, 1.0, 0.95],
            intensity: 1.0,
        })
    }

    // 更新-> 通过queue写入gpu buffer
    pub fn update(&mut self, queue: &Queue, delta_time: f32) {
        // 更新相机
        self.camera.update(queue, 0.0);

        // 更新光照
        self.light_manager.update_buffers(queue);

        // 更新场景数据 比如环境光, fog颜色
        let scene_uniforms = SceneUniforms {
            ambient_light: self.ambient_light,
            ambient_intensity: 0.2,
            fog_color: [0.5, 0.6, 0.7],
            fog_density: 0.0,
        };

        // scene_uniform_buffer 理解是一个管道buffer
        queue.write_buffer(
            &self.scene_uniform_buffer,
            0,
            bytemuck::bytes_of(&scene_uniforms),
        );

        // 一次性更新所有 transform 到 buffer
        // let aligned_size =
        //     Self::aligned_uniform_size(std::mem::size_of::<Matrix4<f32>>() as u64) as u32;

        // 根据索引检查entity的is_dirty数据进行写入
        // for (index, &entity) in self.entities.iter().enumerate() {
        //     if let Some(matrix) = self.transform_system.get_world_matrix(entity) {
        //         let matrix_array: [[f32; 4]; 4] = matrix.into();
        //         // println!("entity {:?}", matrix_array);
        //         let offset = index as u32 * aligned_size;
        //         // 固定的偏移量
        //         self.entity_offsets.insert(entity, offset);
        //
        //         queue.write_buffer(
        //             &self.transforms_uniform_buffer,
        //             offset as u64,
        //             bytemuck::cast_slice(&[matrix_array]),
        //         );
        //     }
        // }

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

    fn total_show_entities(&self) -> usize {
        // println!("total_show_entities: {:?}", &self.entity_display_map);
        self.entities.len() - self.entity_display_map
            .iter()
            .filter(|&(_, display)| *display == false)
        .count()
    }

    pub fn render<'a>(
        &'a mut self,
        device: &Device,
        render_pass: &mut wgpu::RenderPass<'a>,
        resource_manager: &'a ResourceManager,
    ) {
        
        self.render_batches.update_instance_buffers(device, &self.transform_system);

        for batch in self.render_batches.batches.values() {
            let Some(instance_buffer) = &batch.instance_buffer  else {
                continue;
            };
            // 从资源管理器获取 mesh
            let Some(mesh) = resource_manager.has_mesh(&batch.mesh_id) else {
                // println!("Mesh does not exist for {:?}", entity);
                continue; // 没有 mesh 就跳过
            };

            // 从资源管理器获取 mesh
            let Some(material) = resource_manager.has_material(&batch.material_id) else {
                // println!("Mesh does not exist for {:?}", entity);
                continue; // 没有 mesh 就跳过
            };

            render_pass.set_pipeline(&mesh.render_pipeline);
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // 设置渲染管线- 动态管线
            // bind_group全局资源
            render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
            render_pass.set_bind_group(1, &self.scene_bind_group, &[]);
            // render_pass.set_bind_group(2, &self.light_manager.bind_group, &[]);
            // render_pass.set_bind_group(2, &self.transform_bind_group, &[offset]);

            // print!("pipeline {},", batch.entities.len());

            render_pass.set_bind_group(2, &material.bind_group, &[]);
            // 创建pipeline 布局等等，设置buffer之类
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..batch.entities.len() as u32,);
        }


        // println!("resource_manager: {:#?}", &resource_manager);
        // 渲染实体
        // for entity in &self.entities {
        //     let hidden = self.entity_display_map.get(entity).unwrap_or(&false);
        //     // 隐藏属性不展示
        //     if *hidden == true {
        //         continue;
        //     }
        //
        //     // 从资源管理器获取 mesh
        //     let Some(mesh) = resource_manager.get_mesh(entity) else {
        //         // println!("Mesh does not exist for {:?}", entity);
        //         continue; // 没有 mesh 就跳过
        //     };
        //
        //     // 从资源管理器获取 material
        //     let Some(material) = resource_manager.get_material(entity) else {
        //         println!("Material does not exist for {:?}", entity);
        //         continue;
        //     };
        //
        //     // let Some(world_matrix) = self.transform_system.get_world_matrix(*entity) else {
        //     //     println!("World matrix does not exist for {:?}", entity);
        //     //     continue;
        //     // };
        //     let Some(&offset) = self.entity_offsets.get(&entity) else {
        //         continue;
        //     };
        //
        //     // 绑定模型特定的资源并渲染
        //     render_pass.set_pipeline(&mesh.render_pipeline);
        //     render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        //     render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        //
        //     // 设置渲染管线- 动态管线
        //     // bind_group全局资源
        //     render_pass.set_bind_group(0, &self.camera.bind_group, &[]);
        //     render_pass.set_bind_group(1, &self.scene_bind_group, &[]);
        //     // render_pass.set_bind_group(2, &self.light_manager.bind_group, &[]);
        //     // render_pass.set_bind_group(2, &self.transform_bind_group, &[offset]);
        //
        //     render_pass.set_bind_group(3, &material.bind_group, &[]);
        //
        //     // 创建pipeline 布局等等，设置buffer之类
        //     render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        // }
    }
}
