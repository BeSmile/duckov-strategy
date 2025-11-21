
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLight {
    // 内存对齐
    pub position: [f32; 3],
    pub _padding1: f32,
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,// what's mean?
    pub _padding2: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLight {
    pub direction: [f32; 3],
    pub _padding1: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpotLight {
    pub position: [f32; 3],
    pub _padding1: f32,
    pub direction: [f32; 3],
    pub cutoff: f32,
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
    pub outer_cutoff: f32,
    pub _padding2: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightCounts {
    point_light_count: u32,
    directional_light_count: u32,
    spot_light_count: u32,
    _padding: u32,
}

const MAX_POINT_LIGHTS: usize = 16;
const MAX_DIRECTIONAL_LIGHTS: usize = 4;
const MAX_SPOT_LIGHTS: usize = 8;

pub struct LightManager{
    pub point_lights: Vec<PointLight>,
    pub directional_lights: Vec<DirectionalLight>,
    pub spot_lights: Vec<SpotLight>,

    // GPU buffers
    point_light_buffer: wgpu::Buffer,
    directional_light_buffer: wgpu::Buffer,
    spot_light_buffer: wgpu::Buffer,
    light_count_buffer: wgpu::Buffer,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}


impl LightManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let point_light_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("point_lights"),
            size: std::mem::size_of::<PointLight>() as u64,
            #[cfg(target_arch = "wasm32")]
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,  // UNIFORM wasm中不支持STORAGE
            #[cfg(not(target_arch = "wasm32"))]
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let directional_light_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("directional_lights"),
            size: std::mem::size_of::<DirectionalLight>() as u64,
            #[cfg(target_arch = "wasm32")]
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,  // UNIFORM wasm中不支持STORAGE
            #[cfg(not(target_arch = "wasm32"))]
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let spot_light_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("spot_lights"),
            size: std::mem::size_of::<SpotLight>() as u64,
            #[cfg(target_arch = "wasm32")]
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,  // UNIFORM wasm中不支持STORAGE
            #[cfg(not(target_arch = "wasm32"))]
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_count_buffer = device.create_buffer(&wgpu::BufferDescriptor{
            label: Some("light_counts"),
            size: std::mem::size_of::<LightCounts>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout = Self::create_bind_group_layout(&device);


        let bind_group = Self::create_bind_group(&device, &bind_group_layout, &point_light_buffer, &directional_light_buffer, &spot_light_buffer, &light_count_buffer);

        Self {
            point_lights: Vec::new(),
            directional_lights: Vec::new(),
            spot_lights: Vec::new(),
            point_light_buffer,
            directional_light_buffer,
            spot_light_buffer,
            light_count_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn add_point_light(&mut self, light: PointLight) {
        if self.point_lights.len() < MAX_POINT_LIGHTS {
            self.point_lights.push(light);
        }
    }

    pub fn add_directional_light(&mut self, light: DirectionalLight) {
        if self.directional_lights.len() < MAX_DIRECTIONAL_LIGHTS {
            self.directional_lights.push(light);
        }
    }

    pub fn add_spot_light(&mut self, light: SpotLight) {
        if self.spot_lights.len() < MAX_SPOT_LIGHTS {
            self.spot_lights.push(light);
        }
    }

    // 
    pub fn update_buffers(&self, queue: &wgpu::Queue) {
        if !self.point_lights.is_empty() {
            queue.write_buffer(&self.point_light_buffer, 0, bytemuck::cast_slice(&self.point_lights));
        }

        // update directional_lights
        if !self.directional_lights.is_empty() {
            queue.write_buffer(&self.directional_light_buffer, 0, bytemuck::cast_slice(&self.directional_lights));
        }
        if !self.spot_lights.is_empty() {
            queue.write_buffer(&self.spot_light_buffer, 0, bytemuck::cast_slice(&self.spot_lights));
        }

        let counts = LightCounts{
            point_light_count: self.point_lights.len() as u32,
            directional_light_count: self.directional_lights.len() as u32,
            spot_light_count: self.spot_lights.len() as u32,
            _padding: 0,
        };

        queue.write_buffer(&self.light_count_buffer, 0, bytemuck::bytes_of(&counts));
    }

    fn create_bind_group(device: &wgpu::Device,bind_group_layout: &wgpu::BindGroupLayout ,point_light_buffer: &wgpu::Buffer, directional_light_buffer: &wgpu::Buffer, spot_light_buffer: &wgpu::Buffer, light_count_buffer: &wgpu::Buffer) -> wgpu::BindGroup {
       device.create_bind_group( &wgpu::BindGroupDescriptor{
           label: Some("bind_group"),
           layout: bind_group_layout,
           entries: &[
               wgpu::BindGroupEntry{
                   binding: 0,
                   resource: point_light_buffer.as_entire_binding(),
               },
               wgpu::BindGroupEntry{
                   binding: 1,
                   resource: directional_light_buffer.as_entire_binding(),
               },
               wgpu::BindGroupEntry{
                   binding: 2,
                   resource: spot_light_buffer.as_entire_binding(),
               },
               wgpu::BindGroupEntry{
                   binding: 3,
                   resource: light_count_buffer.as_entire_binding(),
               },
           ],
       })
    }

    // 说明bind_group的传参格式
    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("Light Bind Group Layout"),
            entries: &[
                // point
                wgpu::BindGroupLayoutEntry{
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        #[cfg(target_arch = "wasm32")]
                        ty: wgpu::BufferBindingType::Uniform, // wasm中不支持Storage
                        #[cfg(not(target_arch = "wasm32"))]
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        #[cfg(target_arch = "wasm32")]
                        ty: wgpu::BufferBindingType::Uniform, // wasm中不支持Storage
                        #[cfg(not(target_arch = "wasm32"))]
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        #[cfg(target_arch = "wasm32")]
                        ty: wgpu::BufferBindingType::Uniform, // wasm中不支持Storage
                        #[cfg(not(target_arch = "wasm32"))]
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry{
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        })
    }
}



