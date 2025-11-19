use std::collections::HashMap;
use image::DynamicImage;
use serde::{Deserialize, Deserializer, Serialize};
use wgpu::{AddressMode, Device, FilterMode, Queue, TextureView};
use wgpu::naga::compact::KeepUnused::No;
use crate::resource::load_binary;
use crate::unity::{Color, UnityReference};

// 由 texture view 采样 组成
#[derive(Debug, Clone)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d { // 2.
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }
    
    // 创建一个白色的材质
    fn create_dummy_white(device: &Device, queue: &Queue) -> Texture {
        use wgpu::util::DeviceExt;

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Dummy White"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,// 设置输出格式srgb
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[
                    // Self::DEPTH_FORMAT,
                ],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &[255, 255, 255, 255],  // 数据在创建时直接写入
        );

        Texture {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            texture,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor::default()),
        }
    }

    pub fn  from_bytes(device: &Device, queue: &Queue, bys: Vec<u8>, label:&str) -> anyhow::Result<Self>{
        let img = image::load_from_memory(&bys)?;
        Self::from_image(device, queue, &img, label)
    }

    pub fn from_image(device: &Device, queue: &Queue, img: &image::DynamicImage, label: &str) -> anyhow::Result<Self> {
        let rgba = img.to_rgba8();// 4通道数据
        let dimensions = rgba.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor{
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,// 设置输出格式srgb
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,// COPY_DST表示临时数据复制

            view_formats: &[
                // Self::DEPTH_FORMAT,
            ],
        });

        // 将cpu数据复制到gpu中
        queue.write_texture(
            wgpu::TexelCopyTextureInfo{
                texture: &texture, // 目标纹理
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout{
                offset: 0,
                bytes_per_row: Some(dimensions.0 * 4),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w:  wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self{
            texture,
            view,
            sampler
        })
    }
}

pub struct MaterialLayoutBuilder {
    entries: Vec<wgpu::BindGroupLayoutEntry>,
    next_binding: u32,
    bind_group_entries: Vec<wgpu::BindGroupEntry<'static>>,
}

impl MaterialLayoutBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_binding: 0,
            bind_group_entries: Vec::new(),
        }
    }

    // pub fn add_texture_view_entry(&mut self, view: TextureView) {
    //     self.bind_group_entries.push(wgpu::BindGroupEntry{
    //         binding: self.next_binding,
    //         resource: wgpu::BindingResource::TextureView(&view),
    //     });
    // }

    pub fn add_texture(&mut self) {

        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.next_binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });
        self.next_binding += 1;
    }

    pub fn add_sampler(&mut self){

        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.next_binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
        self.next_binding += 1;
    }

    // pub fn add_uniform_buffer(mut self) -> Self {
    //
    //     self.entries.push(wgpu::BindGroupLayoutEntry {
    //         binding: self.next_binding,
    //         visibility: wgpu::ShaderStages::FRAGMENT,
    //         ty: wgpu::BindingType::Buffer {
    //             ty: wgpu::BufferBindingType::Uniform,
    //             has_dynamic_offset: false,
    //             min_binding_size: None,
    //         },
    //         count: None,
    //     });
    //     self.next_binding += 1;
    //
    //     self
    // }

    pub fn build(self, device: &Device, label: &str) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(label),
            entries: &self.entries,
        })
    }
}

// 材质 一次管线渲染只用一个材质
#[derive(Debug)]
pub struct Material{
    pub id: usize,
    pub name: String,
    pub albedo_texture: Option<Texture>,      // _MainTex
    pub normal_texture: Option<Texture>,      // _BumpMap
    pub metallic_texture: Option<Texture>,    // _MetallicGlossMap
    pub ao_texture: Option<Texture>,          // _OcclusionMap

    metallic: f32,                // _Metallic
    roughness: f32,               // 1.0 - _Glossiness
    base_color: [f32; 4],         // _Color
    normal_scale: f32,            // _BumpScale

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,// 定义的bing_group数据

}

impl Material{
    pub async fn from_unity_bytes(bytes: &[u8], device: &Device, queue: &Queue) -> anyhow::Result<Self>{
        let content = std::str::from_utf8(bytes)?;
        let mat = serde_yaml::from_str::<MatYaml>(content)?;
        let unity_material = mat.material;
        let tex_envs = unity_material.saved_properties.tex_envs;
        let mut albedo_texture = None;
        let mut normal_texture = None;
        let mut metallic_texture = None;
        let mut ao_texture = None;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            // compare: Some(wgpu::CompareFunction::LessEqual), // 5.
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            
            ..Default::default()
        });

        let white_texture = Texture::create_dummy_white(device, queue);

        let mut builder = MaterialLayoutBuilder::new();

        let mut entries = Vec::new();

        if let Some(main_tex) = tex_envs.main_tex {
            println!("main_tex: {:?}", main_tex);
            // 临时设置主贴图
            let texture_bytes = load_binary("T_ElectricControlBox_C.png").await?;
            let texture = Texture::from_bytes(device, queue, texture_bytes, "T_ElectricControlBox_C.png")?;

            albedo_texture = Some(texture);
        } else {
            albedo_texture = Some(white_texture.clone());
        }

        entries.push(wgpu::BindGroupEntry{
            binding: builder.next_binding,
            resource: wgpu::BindingResource::TextureView(&albedo_texture.as_ref().unwrap().view),
        });
        builder.add_texture();

        if let Some(normal_map) = tex_envs.normal_map {
            println!("normal_map: {:?}", normal_map);
            // 临时设置主贴图
            let texture_bytes = load_binary("T_ElectricControlBox_MMOR.png").await?;

            let texture = Texture::from_bytes(device, queue, texture_bytes, "T_ElectricControlBox_MMOR.png")?;
            normal_texture = Some(texture);

        } else {
            normal_texture = Some(white_texture.clone());
        }

        entries.push(wgpu::BindGroupEntry{
            binding: builder.next_binding,
            resource: wgpu::BindingResource::TextureView(&normal_texture.as_ref().unwrap().view),
        });
        builder.add_texture();

        // 暂时创建白色的
        if let Some(metallic_smoothness) = tex_envs.metallic_smoothness {
            let texture = Texture::create_dummy_white(&device, &queue);
            metallic_texture = Some(texture);
        } else {
            metallic_texture = Some(white_texture.clone());
        }
        entries.push(wgpu::BindGroupEntry{
            binding: builder.next_binding,
            resource: wgpu::BindingResource::TextureView(&metallic_texture.as_ref().unwrap().view),
        });
        builder.add_texture();

        if let Some(m_texture) = tex_envs.m_texture {
            let texture = Texture::create_dummy_white(&device, &queue);
            ao_texture = Some(texture);
        } else {
            ao_texture = Some(white_texture.clone());
        }

        entries.push(wgpu::BindGroupEntry{
            binding: builder.next_binding,
            resource: wgpu::BindingResource::TextureView(&ao_texture.as_ref().unwrap().view),
        });
        builder.add_texture();

        entries.push(wgpu::BindGroupEntry{
            binding: builder.next_binding,
            resource: wgpu::BindingResource::Sampler(&sampler),
        });
        builder.add_sampler();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some(&format!("Material bind_group_layout : {}", unity_material.name)),
            entries: &builder.entries,
        });

        println!("bind_group_layout:{:?}", &builder.entries);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some(&format!("Material bind_group : {}", unity_material.name)),
            layout: &bind_group_layout,
            entries: &entries,
        });

        Ok(Self{
            id: 1,
            name: unity_material.name,
            albedo_texture,
            normal_texture,
            metallic_texture,
            ao_texture,
            metallic: 0.0,
            roughness: 0.0,
            base_color: [1.0,1.0,1.0,1.0],
            normal_scale: 0.0,
            bind_group_layout,
            bind_group,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextureProperty {
    #[serde(rename = "m_Texture")]
    pub texture: UnityReference,
    #[serde(rename = "m_Scale")]
    pub scale: cgmath::Vector2<u8>,
    #[serde(rename = "m_Offset")]
    pub offset: cgmath::Vector2<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TexEnvs {
    #[serde(rename = "_EmissionMap")]
    pub m_texture: Option<TextureProperty>,
    #[serde(rename = "_MainTex")]
    pub main_tex: Option<TextureProperty>,
    #[serde(rename = "_MetallicSmoothness")]
    pub metallic_smoothness: Option<TextureProperty>,
    #[serde(rename = "_NormalMap")]
    pub normal_map: Option<TextureProperty>,
}

#[derive(Debug, Deserialize)]
pub struct Colors{
    #[serde(rename = "_MaskTint")]
    pub mask_int: Color,
}

#[derive(Debug, Deserialize)]
pub struct SavedProperties {
    #[serde(rename = "serializedVersion")]
    pub serialized_version: i32,
    #[serde(rename = "m_TexEnvs")]
    pub tex_envs: TexEnvs,
    #[serde(rename = "m_Ints")]
    pub ints: HashMap<String, i32>,
    // #[serde(rename = "m_Floats")]
    // pub floats: MFloats,
    #[serde(rename = "m_Colors")]
    pub colors: Colors,
    // #[serde(rename = "m_BuildTextureStacks")]
    // pub build_texture_stacks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MaterialYaml {
    #[serde(rename = "m_Name")]
    pub name: String,
    #[serde(rename = "m_SavedProperties")]
    pub saved_properties: SavedProperties,
}

#[derive(Debug, Deserialize)]
struct MatYaml{
    #[serde(rename = "Material")]
    pub material: MaterialYaml,
}