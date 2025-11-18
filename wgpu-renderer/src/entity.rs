use cgmath::{Deg, Matrix4, Quaternion};
use cgmath::num_traits::ToPrimitive;
use half::f16;
use serde::Deserialize;
use wgpu::{Device, Queue, VertexFormat};
use wgpu::util::DeviceExt;
use crate::scene::Scene;
use crate::unity::{UnityVertexAttribute, UnityVertexAttributeDescriptor, UnityVertexFormat};

pub trait IVertex{
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f16; 4], // 法线
    // 切线
    tangent: [f16; 4],
    tex_coords: [f16; 2],// uv坐标
}

impl Vertex {
    pub fn flip_z_axis(&mut self) {
        self.position[2] = -self.position[2];
        self.normal[2] = f16::from_f32(-self.normal[2].to_f32());
        self.tangent[2] = f16::from_f32(-self.tangent[2].to_f32());
    }
}

impl IVertex for Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        println!("Vertex desc sizeof: {}", std::mem::size_of::<Self>());
        wgpu::VertexBufferLayout{
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: Default::default(),
            attributes: &[
                wgpu::VertexAttribute{
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute{
                    format: VertexFormat::Float16x4,
                    offset:  std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute{
                    format: VertexFormat::Float16x4,
                    offset:  std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute{
                    format: VertexFormat::Float16x2,
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 4,
                }
            ],
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Channel{
    pub stream: u8,
    pub offset: u8,
    pub format: u8,
    pub dimension: u8,
}

#[derive(Debug, Deserialize)]
pub struct VertexDataRaw{
    #[serde(rename(deserialize = "serializedVersion"))]
    serialized_version: i8,
    #[serde(rename(deserialize = "m_VertexCount"))]
    pub vertex_count: String,
    #[serde(rename(deserialize = "m_Channels"))]
    pub m_channels: Vec<Channel>,
    #[serde(rename(deserialize = "m_DataSize"))]
    pub data_size: f32,
    #[serde(rename(deserialize = "_typelessdata"))]
    pub _type_less_data: String,
}

#[derive(Debug, Deserialize)]
pub struct MeshRaw{
    #[serde(rename(deserialize = "m_Name"))]
    pub m_name: String,
    #[serde(rename(deserialize = "m_IndexBuffer"))]
    pub index_buffer: String,
    #[serde(rename(deserialize = "m_VertexData"))]
    pub vertex_data: VertexDataRaw,
    #[serde(rename(deserialize = "m_SubMeshes"))]
    pub sub_mesh: Vec<SubMesh>,
}

#[derive(Debug, Deserialize)]
pub struct SubMesh {
    // #[serde(rename(deserialize = "serializedVersion"))]
    // pub serialized_version: u32,
    #[serde(rename(deserialize = "firstByte"))]
    pub first_byte: u32,
    #[serde(rename(deserialize = "indexCount"))]
    pub index_count: u32,
    pub topology: u32,
    #[serde(rename(deserialize = "baseVertex"))]
    pub base_vertex: u32,
    #[serde(rename(deserialize = "firstVertex"))]
    pub first_vertex: u32,
    #[serde(rename(deserialize = "vertexCount"))]
    pub vertex_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct MeshAsset{
    #[serde(rename(deserialize = "Mesh"))]
    pub mesh: MeshRaw
}

// 每个mesh都有自己的desc
#[derive(Debug, Clone)]
pub struct Mesh{
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    // 顶点数量
    pub vertex_count: u32,
    pub index_count: u32,

    // unity的描述
    pub vertex_descriptors: Vec<UnityVertexAttributeDescriptor>,
    // pub pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
    // 后续使用
    // pub num_elements: u32,//
    // pub material: usize,
}

impl Mesh{
    // 转换并反转缠绕顺序
    // pub fn parse_index_buffer(hex_string: &str) -> Vec<u32> {
    //     let mut indices = parse_unity_index_buffer(hex_string);
    //
    //     // 反转每个三角形的缠绕顺序
    //     for chunk in indices.chunks_exact_mut(3) {
    //         chunk.swap(0, 2);
    //     }
    //     // println!("{:?}", indices);
    //
    //     indices
    // }

    fn parse_vertex_data(hex_string: &str) -> Vec<Vertex> {
        //     let mut vertices: Vec<Vertex> = Vec::with_capacity(vertex_count);
        //     // 清理数据
        //     let hex_clean: String = hex_string.chars().filter(|c| !c.is_whitespace()).collect();
        //
        //     let bytes_per_vertex = 36;
        //     let stride = bytes_per_vertex * 2; // 每字节 2 个十六进制字符, unity的数据是16进制
        //
        //     for i in 0..vertex_count {
        //         let start = i * stride;
        //         if start + stride > hex_clean.len() {
        //             break;
        //         }
        //
        //         let vertex_hex = &hex_clean[start..start + stride];
        //
        //         // 解析位置 (前12字节 = 24个十六进制字符)
        //         let pos_x = parse_f32_le(&vertex_hex[0..8]);
        //         let pos_y = parse_f32_le(&vertex_hex[8..16]);
        //         let pos_z = parse_f32_le(&vertex_hex[16..24]);
        //
        //         // 解析法向量 (12-24字节)
        //         let norm_x = parse_f32_le(&vertex_hex[24..32]);// offset 12
        //         let norm_y = parse_f32_le(&vertex_hex[32..40]);
        //         let norm_z = parse_f32_le(&vertex_hex[40..48]);
        //
        //         // 解析dimension:4
        //         // let color_r = parse_f32_le(&vertex_hex[48..56]);
        //         // let color_g = parse_f32_le(&vertex_hex[56..64]);
        //         // let color_b = parse_f32_le(&vertex_hex[64..72]);
        //         // let color_a = parse_f32_le(&vertex_hex[72..80]);
        //
        //         // 解析 UV (28-36字节)
        //         let uv_x = parse_f32_le(&vertex_hex[56..64]);
        //         let uv_y = parse_f32_le(&vertex_hex[64..72]);
        //
        //     }
        //
        //     vertices
        // 移除可能的空格和换行
        let cleaned = hex_string.replace([' ', '\n', '\r'], "");

        // 解码十六进制字符串为字节
        let bytes = hex::decode(cleaned).expect("Invalid hex string");

        println!("Vertex: sizeof {:?}", std::mem::size_of::<Vertex>());
        // 检查字节数是否是顶点大小的整数倍
        assert_eq!(bytes.len() % std::mem::size_of::<Vertex>(), 0);

        // 转换为顶点数组
        let vertices: &[Vertex] = bytemuck::cast_slice(&bytes);

        let mut vertices = vertices.to_vec(); // 克隆到 Vec<Vertex>

        // let _ = vertices.iter_mut().map(|v| {
        //     v.flip_z_axis();
        //     v
        // });

        vertices.to_vec()
    }

    fn parse_index_buffer(hex_string: &str) -> Vec<u16> {
        // 移除空格和换行
        let cleaned = hex_string.replace([' ', '\n', '\r'], "");

        // 解码十六进制字符串为字节
        let bytes = hex::decode(cleaned).expect("Invalid hex string");
        // println!("Index: sizeof {:?}", bytes);
        // 将字节转换为 u16 索引数组
        let indices: &[u16] = bytemuck::cast_slice(&bytes);

        indices.to_vec()
    }

    // 初始化pipeline 以及各类的布局
    pub fn from_unity_data(buff:  &[u8], device: &Device, scene: &Scene, config: &wgpu::SurfaceConfiguration) -> anyhow::Result<Mesh> {
        let content = std::str::from_utf8(buff)?;
        let raw_asset = serde_yaml::from_str::<MeshAsset>(content)?;
        let raw = raw_asset.mesh;
        let Some(sub_mesh) = raw.sub_mesh.get(0) else {
            return Err(anyhow::anyhow!("Mesh does not contain sub mesh"));
        };

        let vertices = Mesh::parse_vertex_data(&raw.vertex_data._type_less_data);
        println!("vertices: {:?}", vertices);
        let indices = Mesh::parse_index_buffer(&raw.index_buffer);

        let vertex_descriptors = Self::render_descriptors(raw.vertex_data.m_channels);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some(&format!("Mesh_Vertice: {}", raw.m_name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some(&format!("Mesh_Index: {}", raw.m_name)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some(&format!("Mesh_PipelineLayout: {}", raw.m_name)),
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        
        let buffer_layout = Self::get_vertex_buffer_layout(&vertex_descriptors);

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
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: Some(&format!("Mesh_Pipeline: {}", raw.m_name)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState{
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    buffer_layout.as_ref()
                ],
            },
            primitive,
            depth_stencil: None,
            multisample,
            fragment: Some(wgpu::FragmentState{
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        Ok(Mesh{
            name: format!("Mesh: {}", raw.m_name),
            vertex_buffer,
            index_buffer,
            index_count: sub_mesh.index_count,
            vertex_count: sub_mesh.vertex_count,
            vertex_descriptors,
            render_pipeline
        })
    }

    pub fn get_vertex_stride(vertex_descriptors: &Vec<UnityVertexAttributeDescriptor>) -> wgpu::BufferAddress {
        vertex_descriptors
            .iter()
            .map(|desc| {
                desc.size_in_bytes() as wgpu::BufferAddress
            })
            .sum()
    }

    pub fn get_vertex_buffer_layout(vertex_descriptors: &Vec<UnityVertexAttributeDescriptor>) -> VertexBufferLayoutOwned {
        let attributes: Vec<wgpu::VertexAttribute> = vertex_descriptors
            .iter()
            .filter_map(|desc| {
                if let Some(format) = desc.to_wgpu_format(){
                    let attr = wgpu::VertexAttribute {
                        offset: desc.offset as wgpu::BufferAddress,
                        shader_location: desc.shader_location(),
                        format,
                    };
                    Some(attr)
                } else {
                    None
                }
            }).collect();

        println!(" Self::attributes: {:?}",  attributes);
        println!(" Self::get_vertex_stride(vertex_descriptors): {:?}",  Self::get_vertex_stride(vertex_descriptors));
        VertexBufferLayoutOwned {
            array_stride: Self::get_vertex_stride(vertex_descriptors),
            step_mode: Default::default(),
            attributes,
        }
    }

    pub fn render_descriptors(m_channels: Vec<Channel>) -> Vec<UnityVertexAttributeDescriptor> {
        // 根据channel 渲染
        let mut vertex_descriptors: Vec<UnityVertexAttributeDescriptor> = Vec::new();
        for (i, channel)     in m_channels.iter().enumerate() {
            vertex_descriptors.push(UnityVertexAttributeDescriptor{
                attribute: UnityVertexAttribute::from_u8(i as u8),
                format: UnityVertexFormat::from_u8(channel.format),
                dimension: channel.dimension,
                stream: channel.stream,
                offset: channel.offset,
            })
        }

        vertex_descriptors
    }

    
}

// gameObject包含多个Mesh,等同于Model加载, 管理pipe_line
pub struct Model{
    pub id: usize,
    pub name: String,
    pub meshs: Vec<Mesh>
}

impl Model{
    // pub fn render<'a>(&self, render_pass: &mut wgpu::RenderPass<'a>, transform: &Transform) {
    //     // 多个mesh进行渲染顶点
    //     for mesh in &self.meshs {
    //         render_pass.set_pipeline(&mesh.render_pipeline);
    //         render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
    //         render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    //         // 创建pipeline 布局等等，设置buffer之类
    //         render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
    //     }
    // }
}

#[derive(Debug, Clone)]
pub struct VertexBufferLayoutOwned {
    pub array_stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}

impl VertexBufferLayoutOwned {
    pub fn as_ref(&self) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: self.step_mode,
            attributes: &self.attributes,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Transform {
    pub position: [f32; 3],
    pub _padding1: f32,
    pub rotation: [f32; 4], // quaternion
    pub scale: [f32; 3],
    pub _padding2: f32,
}

// 每个实体都有一个model， model在scene中管理, 有多个子mesh，暂时处理单个mesh的情况
pub struct Entity {
    pub model_id: usize,
    pub transform: Transform,
    // pub material_override: Option<Material>,
    pub visible: bool,
}

impl Entity {
    pub fn new(model_id: usize) -> Self {
        Self {
            model_id,
            transform: Transform {
                position: [0.0, 0.0, 0.0],
                _padding1: 0.0,
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
                _padding2: 0.0,
            },
            // material_override: None,
            visible: true,
        }
    }

    pub fn set_position(&mut self, position: [f32; 3]) {
        self.transform.position = position;
    }

    pub fn set_scale(&mut self, scale: [f32; 3]) {
        self.transform.scale = scale;
    }

    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        model: &'a Model,
    ) {
        
    }

    // 更新函数，主要会用作自旋转等操作
    pub fn update(&self, delta_time: f32) {
        // 后续更新
    }

    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        // 计算空间矩阵
        // 平移矩阵
        let translation = Matrix4::from_translation(self.transform.position.into());
        // 旋转矩阵
        let rotation = Matrix4::from(Quaternion::new(
            self.transform.rotation[3], // w (实部)
            self.transform.rotation[0], // x
            self.transform.rotation[1], // y
            self.transform.rotation[2], // z
        ));
        let scale = Matrix4::from_nonuniform_scale(
            self.transform.scale[0],
            self.transform.scale[1],
            self.transform.scale[2],
        );

        translation * rotation * scale
    }
}