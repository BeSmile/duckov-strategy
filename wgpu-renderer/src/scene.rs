use std::collections::HashMap;
use std::sync::Arc;

use cgmath::{Matrix4, Vector3, Vector4};
use wgpu::{Device, Queue, SurfaceConfiguration};

use crate::camera::Camera;
use crate::entity::{Entity, InstanceRaw, Transform, TransformSystem};
use crate::light::{DirectionalLight, LightManager, PointLight};
use crate::materials::Texture;
use crate::resource::{MaterialId, MeshId, ResourceManager};
use crate::utils::get_background_color;

pub type PipelineId = String;

use log::{error, info};
use wgpu::util::DeviceExt;

use crate::ray::Ray;
use crate::unity::{
    Component, UnityGameObject, UnityMeshFilter, UnityMeshRenderer, UnityScene, UnityTransform,
};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
use crate::stat::{set_loading_state, SceneLoadingState};

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

    pub entities: Vec<Entity>,                 // 存档所有的实体类key
    entity_display_map: HashMap<Entity, bool>, // entity的显示隐藏，true, false 隐藏， 但是隐藏的才会塞入，后续调整
    entity_frustum_culling: HashMap<Entity, bool>,// true显示。false隐藏

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

    // 视锥剔除
    pub frustum: crate::frustum::Frustum,
    pub culling_enabled: bool,
}

// render一次批量
struct RenderBatch {
    pub mesh_id: MeshId,
    pub material_id: MaterialId,
    pub entities: Vec<Entity>,                 // 属于这个批次的entities
    pub instance_buffer: Option<wgpu::Buffer>, // 延迟创建
    pub instance_count: u32,                   // 实际创建的实例数量（用于draw_indexed）
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
        entity_display_map: &HashMap<Entity, bool>,
    ) {
        // 更新instance_buffers，同时应用视锥剔除
        for batch in self.batches.values_mut() {
            let instances: Vec<InstanceRaw> = batch
                .entities
                .iter()
                .filter(|&&entity| {
                    // 只处理可见的实体
                    entity_display_map.get(&entity).copied().unwrap_or(true)
                })
                .filter_map(|&entity| {
                    let transform = transform_system.get_world_matrix(entity);
                    match transform {
                        Some(ts) => Some(InstanceRaw { model: ts.into() }),
                        None => {
                            println!("entity: {:?} not transofm", entity);
                            None
                        }
                    }
                })
                .collect();

            let instance_count = instances.len() as u32;

            if instances.is_empty() {
                batch.instance_buffer = None; // 全部被剔除
                batch.instance_count = 0;
                continue;
            }
            // 创建/更新instance buffer
            batch.instance_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ));
            batch.instance_count = instance_count;
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
                    instance_count: 0,
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
    pub light_direction: [f32; 3],
    pub _padding2: f32,
    pub light_color: [f32; 3],
    pub _padding3: f32,
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
        info!("Real alignment required: {}", alignment); // 在 macOS 上打印看看
        // ⭐ 一次性更新所有 transform 到 buffer
        // let max_entities = 100000;// 数据连续： 优化部分，transform 高频变化数据，直接通过offset进行数据的写入以及更新，利用entity的is_dirty进行管理是否更新
        let aligned_size = Self::aligned_uniform_size(size_of::<Matrix4<f32>>() as u64);

        info!(
            "max_entities: {}, aligned_size: {}",
            max_entities,
            aligned_size * max_entities as u64
        );
        
        // 初始化视锥体（使用当前相机的view_proj矩阵）
        let initial_view_proj = camera.get_projection_matrix();
        let frustum = crate::frustum::Frustum::from_view_proj(&initial_view_proj);

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
            entity_frustum_culling: HashMap::new(),
            render_batches: RenderBatchSystem::default(),
            frustum,
            culling_enabled:Self::get_culling_enabled(),
        }
    }


    /// 重新加载场景，清空所有实体和运行时状态，但保留 GPU 资源
    pub fn reload(&mut self) {
        info!("Reloading scene...");

        // 清空实体列表
        self.entities.clear();

        // 重置时间
        self.elapsed_time = Instant::now().elapsed().as_secs_f32();

        // 重置 pipeline manager
        self.pipeline_manager = PipelineManager::new();

        // 重置 transform 系统
        self.transform_system = TransformSystem::new();

        // 清空映射表
        self.entity_offsets.clear();
        self.entity_display_map.clear();
        self.entity_frustum_culling.clear();

        // 重置渲染批次
        self.render_batches = RenderBatchSystem::default();

        // 重新初始化视锥体
        let view_proj = self.camera.get_projection_matrix();
        self.frustum = crate::frustum::Frustum::from_view_proj(&view_proj);

        // 重新读取 culling 配置
        self.culling_enabled = Self::get_culling_enabled();

        // 重置背景色
        self.background_color = get_background_color();

        // 重置环境光
        self.ambient_light = [0.9; 3];

        info!("Scene reloaded successfully");
    }

    /// 完全重新初始化场景，包括重建 GPU 资源
    pub fn reload_with_device(&mut self, device: &Device, config: &SurfaceConfiguration) {
        info!("Reloading scene with new GPU resources...");

        // 重建 light manager
        self.light_manager = LightManager::new(device);

        // 重建 camera
        self.camera = Camera::new(device, config.width as f32 / config.height as f32);

        // 重建 scene uniform buffer
        // self.scene_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Scene Uniform Buffer"),
        //     size: size_of::<SceneUniforms>() as wgpu::BufferAddress,
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        // 重建 bind group
        // self.scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("Scene Bind Group"),
        //     layout: &self.scene_bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: self.scene_uniform_buffer.as_entire_binding(),
        //     }],
        // });

        // 调用基础 reload 清空运行时状态
        self.reload();

        info!("Scene reloaded with new GPU resources");
    }

    pub fn clear_entity(&mut self) {
        self.entity_display_map.clear();
        self.entity_frustum_culling.clear();
        self.entity_offsets.clear();
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

    fn get_culling_enabled() -> bool {
        // 读取环境变量，默认启用剔除
        #[cfg(not(target_arch = "wasm32"))]
        let culling_enabled = std::env::var("ENABLE_FRUSTUM_CULLING")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true);

        #[cfg(target_arch = "wasm32")]
        let culling_enabled = true;

        culling_enabled
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
        let objects = &unity_scene.game_object_raw;
        // let mut transforms: HashMap<u32, UnityTransform> = HashMap::new();
        let transforms_raw = &unity_scene.transforms_raw;
        // let mesh_renders = &unity_scene.mesh_renderers;
        let mesh_renderers_raw = &unity_scene.mesh_renderers_raw;
        // let mesh_filters = &unity_scene.mesh_filters;
        let mesh_filters_raw = &unity_scene.mesh_filters_raw;
        let test_id = 16188;
        for (entity_id, game_object) in objects {
            let game_object =
                serde_yaml::from_str::<UnityGameObject>(game_object).map_err(|e| {
                    error!("Failed to deserialize game object: {}: {:?}", entity_id, e);
                    panic!("Failed to deserialize game object: {:?}", e);
                    e
                })?;
            // 查看挂载的transform
            let entity = Entity::new(*entity_id);
            if game_object.m_is_active != 1 {
                scene.hidden_entity(entity);
            }

            if *entity_id == test_id {
                info!("entity: {:?}", entity);
            }
            let mut is_light = false;
            let mut unity_mesh_render: Option<UnityMeshRenderer> = None;
            let mut unity_mesh_filter: Option<UnityMeshFilter> = None;

            let mut local_transform = Transform::new();

            for m_component in &game_object.m_component {
                let file_id = m_component.component.file_id.clone();
                let s_type = indexs.get(&file_id);
                match s_type {
                    // transform管理
                    Some(s) if s.as_str() == "Transform" => {
                        let transform_raw = transforms_raw.get(&file_id);
                        let Some(transform_raw) = transform_raw else {
                            continue;
                        };
                        let unity_transform = serde_yaml::from_str::<UnityTransform>(transform_raw)
                            .map_err(|e| {
                                error!(" 解析错误:{:?} : {:?}", entity_id, e);
                                e
                            })?;
                        if *entity_id == test_id {
                            info!(
                                "{}: transform: {:?}, gameobject : {:?}",
                                *entity_id, unity_transform, &game_object
                            );
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
                        if let Some(m) = &unity_transform.m_father {
                            let transform_raw = transforms_raw.get(&m.file_id);

                            if let Some(transform_raw) = transform_raw {
                                match serde_yaml::from_str::<UnityTransform>(transform_raw) {
                                    Ok(transform) => {
                                        scene.transform_system.set_parent(
                                            Entity::new(transform.m_game_object.file_id),
                                            Entity::new(unity_transform.m_game_object.file_id),
                                        );
                                    }
                                    Err(e) => {
                                        error!("解析坐标异常错误:{:?} : {:?}", entity_id, e);
                                        error!("Failed to deserialize transform: {}", e);
                                    }
                                };
                            }
                        }
                        scene.entities.push(entity);
                    }
                    Some(s) if s.as_str() == "MeshRenderer" => {
                        if let Some(content) = mesh_renderers_raw.get(&file_id) {
                            match serde_yaml::from_str::<UnityMeshRenderer>(content) {
                                Ok(mesh_render) => {
                                    unity_mesh_render = Some(mesh_render);
                                }
                                Err(err) => {
                                    info!("Mesh Renderer{:?}", content);
                                    error!("Serde_yaml Failed to parse mesh renderer: {:?}", err);
                                }
                            };
                        }
                    }
                    // 顶点数据
                    Some(s) if s.as_str() == "MeshFilter" => {
                        if let Some(content) = mesh_filters_raw.get(&file_id) {
                            match serde_yaml::from_str::<UnityMeshFilter>(content) {
                                Ok(mesh_filter) => {
                                    unity_mesh_filter = Some(mesh_filter);
                                }
                                Err(e) => {
                                    info!("MeshFilter{:?}", e);
                                    error!("Serde_yaml Failed to parse mesh filter: {:?}", e);
                                    continue;
                                }
                            }
                        }
                    }
                    Some(s) if s.as_str() == "Light"  => { // || s.as_str() == "SodaPointLight" 暂时不用
                        is_light = true;
                    }
                    Some(s) if s.as_str() == "MonoBehaviour" => {
                        // SodaPointLight是挂载在MonoBehaviour，需要特殊处理
                    }
                    _ => {}
                }
                // println!("loading unity 顶点数据: {:?}", unity_mesh_filter);
            }
            if *entity_id == test_id {
                info!("entity: {:?} End Ground", entity);
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

            if *entity_id == test_id {
                info!("entity: {:?}", entity);
            }
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

        set_loading_state(SceneLoadingState::LoadingAssets, 0.7, "Loading Materials");

        scene.transform_system.update(&mut scene.entity_display_map);

        // 根据culling_enable来判断是否开启来决定渲染的map
        let display_map = if scene.culling_enabled {
            &scene.entity_frustum_culling
        } else {
            &scene.entity_display_map
        };

        set_loading_state(SceneLoadingState::Setting, 0.8, "scene updated...");

        scene.render_batches.rebuild_batches(
            &scene.entities,
            display_map,
            resource_manager,
        );
        set_loading_state(SceneLoadingState::Setting, 0.8, &format!("scene rendered all {} entities", scene.entities.len()));

        info!(
            "scene actually rendered all {} entities",
            scene.total_show_entities()
        );

        Ok(())
    }

    pub fn add_entity(&mut self, entity: Entity, transform: Transform) {
        self.entities.push(entity);
        self.transform_system.add_transform(entity, transform);
    }

    /// 游戏逻辑隐藏实体（不会被视锥剔除影响）
    pub fn hidden_entity(&mut self, entity: Entity) {
        self.entity_display_map.insert(entity, false);
    }

    /// 查看实体是否被游戏逻辑隐藏
    pub fn is_display_by_logic(&self, entity: &Entity) -> bool {
        self.entity_display_map.get(entity).copied().unwrap_or(true)
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
    pub fn update(&mut self, queue: &Queue, delta_time: f32, resource_manager: &ResourceManager) {
        // 更新相机
        self.camera.update(queue, 0.0);

        // 更新视锥体
        let view_proj = self.camera.get_projection_matrix();
        self.frustum = crate::frustum::Frustum::from_view_proj(&view_proj);

        // 更新光照
        self.light_manager.update_buffers(queue);

        // 更新场景数据 比如环境光, fog颜色
        // 更新场景数据 比如环境光, fog颜色
        let dir_light = self.light_manager.directional_lights.first();
        let (light_dir, light_col) = if let Some(l) = dir_light {
             // 归一化光照方向
             let d = l.direction;
             let len = (d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt();
             if len > 0.0 {
                 ([d[0]/len, d[1]/len, d[2]/len], l.color)
             } else {
                 (l.direction, l.color)
             }
        } else {
            ([-0.3, -1.0, -0.5], [1.0, 1.0, 0.95])
        };

        let scene_uniforms = SceneUniforms {
            ambient_light: self.ambient_light,
            ambient_intensity: 0.2,
            fog_color: [0.5, 0.6, 0.7],
            fog_density: 0.0,
            light_direction: light_dir,
            _padding2: 0.0,
            light_color: light_col,
            _padding3: 0.0,
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

        // 视锥剔除：更新entity_display_map
        // entity_frustum_culling 根据视锥 & 游戏实体显示推导的状态
        // entity_display_map 存储游戏内实体显示状态
        if self.culling_enabled {
            let mut visible_count = 0;
            let mut culled_count = 0;

            for &entity in &self.entities {
                // 获取游戏逻辑的隐藏状态（独立于视锥剔除）
                let display_by_logic = self.is_display_by_logic(&entity);

                // 计算视锥剔除结果
                let frustum_visible = if let Some(mesh) = resource_manager.get_mesh(&entity) {
                    if let Some(world_matrix) = self.transform_system.get_world_matrix(entity) {
                        // 将AABB变换到世界坐标
                        let world_aabb = mesh.aabb.transform(&world_matrix);

                        // 测试是否在视锥内
                        self.frustum.is_visible(&world_aabb)
                    } else {
                        false // 没有transform，默认通过视锥测试
                    }
                } else {
                    false // 没有mesh，默认通过视锥测试
                };

                // 最终可见性 = 未被游戏逻辑隐藏 AND 在视锥内
                let final_visible = display_by_logic && frustum_visible;
                // 插入至实际渲染条件
                self.entity_frustum_culling.insert(entity, final_visible);

                // 统计（仅统计未被游戏逻辑隐藏的实体的剔除情况）
                if display_by_logic {
                    if frustum_visible {
                        visible_count += 1;
                    } else {
                        culled_count += 1;
                    }
                }
            }

            // 打印剔除统计（可用于调试）
            #[cfg(debug_assertions)]
            {
                if self.entities.len() > 0 && (visible_count + culled_count) > 0 {
                    let total = visible_count + culled_count;
                    let cull_percentage = (culled_count as f32 / total as f32) * 100.0;
                    info!(
                        "Frustum Culling: {}/{} visible ({:.1}% culled)",
                        visible_count, total, cull_percentage
                    );
                }
                // 打印相机信息
                let camera_pos = *self.camera.eye();
                info!("Camera position: ({:.2}, {:.2}, {:.2})", camera_pos.x, camera_pos.y, camera_pos.z);
            }
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

    pub fn total_show_entities(&self) -> usize {
        // println!("total_show_entities: {:?}", &self.entity_display_map);
        self.entities.len()
            - self
                .entity_display_map
                .iter()
                .filter(|&(_, display)| *display == false)
                .count()
    }

    pub fn render<'a>(
        &'a mut self,
        device: &Device,
        render_pass: &mut wgpu::RenderPass<'a>,
        resource_manager: &'a ResourceManager,
    ) -> Vec<(MeshId, MaterialId)> {
        let display_map = if self.culling_enabled {
            &self.entity_frustum_culling
        } else {
            &self.entity_display_map
        };
        // 更新实例缓冲（应用视锥剔除过滤）
        self.render_batches
            .update_instance_buffers(device, &self.transform_system, display_map);

        // 收集本帧使用的资源ID（用于标记使用）
        let mut used_resources = Vec::new();

        for batch in self.render_batches.batches.values() {
            let Some(instance_buffer) = &batch.instance_buffer else {
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
            // info!("index_count: {:?}, 实例数: {:?}", mesh.index_count, batch.instance_count);
            // 创建pipeline 布局等等，设置buffer之类
            // 使用 instance_count 而不是 entities.len()，因为视锥剔除后实际实例数可能更少
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..batch.instance_count);

            // 记录本帧使用的资源
            used_resources.push((batch.mesh_id.clone(), batch.material_id.clone()));
        }

        used_resources
    }
}
