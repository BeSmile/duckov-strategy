use cgmath::{Deg, Matrix4, Quaternion};
use half::f16;
use serde::Deserialize;
use wgpu::{Device, Queue, VertexFormat};
use wgpu::util::DeviceExt;

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
                    shader_location: 3,
                }
            ],
        }
    }
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
    // 后续使用
    // pub num_elements: u32,//
    // pub material: usize,
}

impl Mesh{
    // pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, mesh_name: &str) -> Mesh{
    //     let vertex_buffer = device.create_buffer_init(&wgpu::VertexBufferLayout)
    // }
    
}

#[derive(Debug, Deserialize)]
pub struct Channel{
    pub stream: i8,
    pub offset: i8,
    pub format: i8,
    pub dimension: i8,
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
        println!("Index: sizeof {:?}", bytes);
        // 将字节转换为 u16 索引数组
        let indices: &[u16] = bytemuck::cast_slice(&bytes);

        indices.to_vec()
    }

    pub fn from_asset(buff:  &[u8], device: &Device, queue: &Queue) -> anyhow::Result<Mesh> {
        let content = std::str::from_utf8(buff)?;
        let raw_asset = serde_yaml::from_str::<MeshAsset>(content)?;
        let raw = raw_asset.mesh;
        let Some(sub_mesh) = raw.sub_mesh.get(0) else {
            return Err(anyhow::anyhow!("Mesh does not contain sub mesh"));
        };
        
        let vertices = Mesh::parse_vertex_data(&raw.vertex_data._type_less_data);
        // println!("vertices: {:?}", vertices);
        let indices = Mesh::parse_index_buffer(&raw.index_buffer);

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

        Ok(Mesh{
            name: format!("Mesh: {}", raw.m_name),
            vertex_buffer,
            index_buffer,
            index_count: sub_mesh.index_count,
            vertex_count: sub_mesh.vertex_count,
        })
    }
}

// gameObject包含多个Mesh,等同于Model加载
pub struct Model{
    pub id: usize,
    pub name: String,
    pub meshs: Vec<Mesh>
}

impl Model{
    pub fn render<'a>(&self, render_pass: &mut wgpu::RenderPass<'a>, transform: &Transform) {
        // 多个mesh进行渲染顶点
        for mesh in &self.meshs {
            println!("Mesh: {}", mesh.index_count);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
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
        if !self.visible {
            return;
        }

        // 绑定模型特定的资源并渲染
        model.render(render_pass, &self.transform);
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