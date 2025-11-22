use std::collections::HashMap;
use bytemuck::Pod;
use cgmath::{Matrix4, One, Quaternion, SquareMatrix, Vector3, Zero};
use half::f16;
use wgpu::{BufferAddress, Device, Queue, SurfaceConfiguration};
use wgpu::util::DeviceExt;
use crate::materials::{Material, Texture};
use crate::resource::MeshId;
use crate::scene::Scene;
use crate::unity::{Channel, MeshAsset, UnityVertexAttribute, UnityVertexAttributeDescriptor, UnityVertexFormat};

// pub trait IVertex{
//     fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
// }

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f16; 4], // 法线
    // 切线
    tangent: [f16; 4],
    tex_coords: [f16; 2],// uv坐标
}

// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct VertexTexFloat32 {
//     position: [f32; 3],// 4 * 3
//     normal: [f16; 4], // 法线 2*4
//     // 切线
//     tangent: [f16; 4],//2*4
//     tex_coords: [f16; 2],// uv坐标2*2
// }

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexFloat32 {    // sizeof 48
    position: [f32; 3],
    normal: [f32; 3], // 法线
    // 切线
    tangent: [f32; 4],
    tex_coords: [f32; 2],// uv坐标
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexColorUVx3Float32 {    // sizeof 80
    position: [f32; 3],
    normal: [f32; 3], // 法线
    // 切线
    tangent: [f32; 4],
    color: [f32; 4],
    tex_coords: [f32; 2],// uv0坐标
    uv_coords: [f32; 2],// uv1坐标
    uv1_coords: [f32; 2],// uv2坐标
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexColorUVFloat32 {    // sizeof 64
    position: [f32; 3],
    normal: [f32; 3], // 法线
    // 切线
    tangent: [f32; 4],
    color: [f32; 4],
    tex_coords: [f32; 2],// uv0坐标
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexTexUvFloat32 { // size_of: 56
    position: [f32; 3],
    normal: [f32; 3], // 法线
    // 切线
    tangent: [f32; 4],
    tex_coords: [f32; 2],// uv0坐标
    uv_coords: [f32; 2],// uv1坐标
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexColor {    // size_of: 36
    position: [f32; 3],
    normal: [f16; 4], // 法线
    // 切线
    tangent: [f16; 4],
    color: [u8; 4], // 颜色
    tex_coords: [f16; 2],// uv0坐标
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexUvFloat1632 { // size_of: 40
    position: [f32; 3],
    normal: [f16; 4], // 法线
    // 切线
    tangent: [f16; 4],
    tex_coords: [f16; 2],// uv0坐标
    uv_coords: [f32; 2],// uv1坐标
}

trait IVertex {
    fn flip_z_axis(&mut self);
}

impl IVertex for Vertex {
    fn flip_z_axis(&mut self) {
        // 翻转Z轴（位置）
        self.position[2] = -self.position[2];
        // 翻转法线Z
        self.normal[2] = f16::from_f32(-self.normal[2].to_f32());

        // 翻转切线Z和手性
        self.tangent[2] = f16::from_f32(-self.tangent[2].to_f32());
        self.tangent[3] = f16::from_f32(-self.tangent[3].to_f32());

        let u = self.tex_coords[0].to_f32();
        let v = self.tex_coords[1].to_f32();

        // 使用fract()获取小数部分，映射到[0,1]
        self.tex_coords[0] = f16::from_f32(u.fract().abs());
        self.tex_coords[1] = f16::from_f32(v.fract().abs());
    }
}
impl IVertex for VertexColor {
    fn flip_z_axis(&mut self) {
        // 翻转Z轴（位置）
        self.position[2] = -self.position[2];
        // 翻转法线Z
        self.normal[2] = f16::from_f32(-self.normal[2].to_f32());

        // 翻转切线Z和手性
        self.tangent[2] = f16::from_f32(-self.tangent[2].to_f32());
        self.tangent[3] = f16::from_f32(-self.tangent[3].to_f32());

        let u = self.tex_coords[0].to_f32();
        let v = self.tex_coords[1].to_f32();

        // 使用fract()获取小数部分，映射到[0,1]
        self.tex_coords[0] = f16::from_f32(u.fract().abs());
        self.tex_coords[1] = f16::from_f32(v.fract().abs());
    }
}

impl IVertex for VertexColorUVx3Float32 {
    fn flip_z_axis(&mut self) {
        // 翻转Z轴（位置）
        self.position[2] = -self.position[2];
        // 翻转法线Z
        self.normal[2] = -self.normal[2];

        // 翻转切线Z和手性
        self.tangent[2] = -self.tangent[2];
        self.tangent[3] = -self.tangent[3];

        let u = self.tex_coords[0];
        let v = self.tex_coords[1];

        // 使用fract()获取小数部分，映射到[0,1]
        self.tex_coords[0] = u.fract().abs();
        self.tex_coords[1] = v.fract().abs();

        let u1 = self.tex_coords[0];
        let v1 = self.tex_coords[1];

        // 使用fract()获取小数部分，映射到[0,1]
        self.uv_coords[0] = u1.fract().abs();
        self.uv_coords[1] = v1.fract().abs();

        let u2 = self.tex_coords[0];
        let v2 = self.tex_coords[1];

        // 使用fract()获取小数部分，映射到[0,1]
        self.uv1_coords[0] = u2.fract().abs();
        self.uv1_coords[1] = v2.fract().abs();
    }
}

impl IVertex for VertexColorUVFloat32 {
    fn flip_z_axis(&mut self) {
        // 翻转Z轴（位置）
        self.position[2] = -self.position[2];
        // 翻转法线Z
        self.normal[2] = -self.normal[2];

        // 翻转切线Z和手性
        self.tangent[2] = -self.tangent[2];
        self.tangent[3] = -self.tangent[3];

        let u = self.tex_coords[0];
        let v = self.tex_coords[1];

        // 使用fract()获取小数部分，映射到[0,1]
        self.tex_coords[0] = u.fract().abs();
        self.tex_coords[1] = v.fract().abs();
    }
}

impl IVertex for VertexFloat32 {
    fn flip_z_axis(&mut self) {
        // 翻转Z轴（位置）
        self.position[2] = -self.position[2];
        // 翻转法线Z
        self.normal[2] = -self.normal[2];

        // 翻转切线Z和手性
        self.tangent[2] = -self.tangent[2];
        self.tangent[3] = -self.tangent[3];

        let u = self.tex_coords[0];
        let v = self.tex_coords[1];

        // 使用fract()获取小数部分，映射到[0,1]
        self.tex_coords[0] = u.fract().abs();
        self.tex_coords[1] = v.fract().abs();
    }
}

impl IVertex for VertexTexUvFloat32 {
    fn flip_z_axis(&mut self) {
        // 翻转Z轴（位置）
        self.position[2] = -self.position[2];
        // 翻转法线Z
        self.normal[2] = -self.normal[2];

        // 翻转切线Z和手性
        self.tangent[2] = -self.tangent[2];
        self.tangent[3] = -self.tangent[3];

        let u = self.tex_coords[0];
        let v = self.tex_coords[1];

        // 使用fract()获取小数部分，映射到[0,1]
        self.tex_coords[0] = u.fract().abs();
        self.tex_coords[1] = v.fract().abs();

        let u1 = self.uv_coords[0];
        let v1 = self.uv_coords[1];

        // 使用fract()获取小数部分，映射到[0,1]
        self.uv_coords[0] = u1.fract().abs();
        self.uv_coords[1] = v1.fract().abs();
    }
}
impl IVertex for VertexUvFloat1632 {
    fn flip_z_axis(&mut self) {
        // 翻转Z轴（位置）
        self.position[2] = -self.position[2];
        // 翻转法线Z
        self.normal[2] = -self.normal[2];

        // 翻转切线Z和手性
        self.tangent[2] = -self.tangent[2];
        self.tangent[3] = -self.tangent[3];

        let u = self.tex_coords[0].to_f32();
        let v = self.tex_coords[1].to_f32();

        // 使用fract()获取小数部分，映射到[0,1]
        self.tex_coords[0] = f16::from_f32(u.fract().abs());
        self.tex_coords[1] = f16::from_f32(v.fract().abs());

        let u1 = self.uv_coords[0];
        let v1 = self.uv_coords[1];

        // 使用fract()获取小数部分，映射到[0,1]
        self.uv_coords[0] = u1.fract().abs();
        self.uv_coords[1] = v1.fract().abs();
    }
}

impl Vertex {
    pub fn analyze_uv_pattern_by_normal(vertices: &[Vertex], indices: &[u16]) {
        let mut x_faces_uvs = Vec::new();
        let mut y_faces_uvs = Vec::new();
        let mut z_faces_uvs = Vec::new();

        for chunk in indices.chunks(3) {
            // 计算三角形的平均法线
            let v0 = &vertices[chunk[0] as usize];
            let v1 = &vertices[chunk[1] as usize];
            let v2 = &vertices[chunk[2] as usize];

            let avg_normal = [
                (v0.normal[0].to_f32() + v1.normal[0].to_f32() + v2.normal[0].to_f32()) / 3.0,
                (v0.normal[1].to_f32() + v1.normal[1].to_f32() + v2.normal[1].to_f32()) / 3.0,
                (v0.normal[2].to_f32() + v1.normal[2].to_f32() + v2.normal[2].to_f32()) / 3.0,
            ];

            let abs_normal = [
                avg_normal[0].abs(),
                avg_normal[1].abs(),
                avg_normal[2].abs(),
            ];

            // 收集UV数据
            let uvs: Vec<(f32, f32)> = chunk.iter().map(|&i| {
                let v = &vertices[i as usize];
                (v.tex_coords[0].to_f32(), v.tex_coords[1].to_f32())
            }).collect();

            // 根据主导法线方向分类
            if abs_normal[0] > abs_normal[1] && abs_normal[0] > abs_normal[2] {
                x_faces_uvs.extend(uvs);
            } else if abs_normal[1] > abs_normal[2] {
                y_faces_uvs.extend(uvs);
            } else {
                z_faces_uvs.extend(uvs);
            }
        }

        // 分析每个方向的UV特征
        println!("X方向面 UV范围: {:?}", Self::calculate_uv_range(&x_faces_uvs));
        println!("Y方向面 UV范围: {:?}", Self::calculate_uv_range(&y_faces_uvs));
        println!("Z方向面 UV范围: {:?}", Self::calculate_uv_range(&z_faces_uvs));
    }

    fn calculate_uv_range(uvs: &[(f32, f32)]) -> (f32, f32, f32, f32) {
        if uvs.is_empty() { return (0.0, 0.0, 0.0, 0.0); }

        let min_u = uvs.iter().map(|uv| uv.0).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_u = uvs.iter().map(|uv| uv.0).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let min_v = uvs.iter().map(|uv| uv.1).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_v = uvs.iter().map(|uv| uv.1).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        (min_u, max_u, min_v, max_v)
    }

    pub fn detect_uv_mapping_type(vertices: &[Vertex]) -> String {
        let mut uv_matches_position = true;
        let mut uv_in_standard_range = true;
        let mut uv_variance = 0.0f32;
        for vertex in vertices {
            let pos = vertex.position;
            let uv = (vertex.tex_coords[0].to_f32(), vertex.tex_coords[1].to_f32());

            // 检查UV是否超出[0,1]范围
            if uv.0 < 0.0 || uv.0 > 1.0 || uv.1 < 0.0 || uv.1 > 1.0 {
                uv_in_standard_range = false;
            }

            // 检查UV是否与位置坐标相关
            // Triplanar通常UV会与世界坐标有关
            let pos_based_uv_x = (pos[0] * 0.1).fract(); // 缩放因子可调
            let pos_based_uv_y = (pos[1] * 0.1).fract();
            let pos_based_uv_z = (pos[2] * 0.1).fract();

            // 检查UV是否匹配某个坐标轴投影
            let matches_xy = (uv.0 - pos_based_uv_x).abs() < 0.1 &&
                (uv.1 - pos_based_uv_y).abs() < 0.1;
            let matches_xz = (uv.0 - pos_based_uv_x).abs() < 0.1 &&
                (uv.1 - pos_based_uv_z).abs() < 0.1;
            let matches_yz = (uv.0 - pos_based_uv_y).abs() < 0.1 &&
                (uv.1 - pos_based_uv_z).abs() < 0.1;

            if !matches_xy && !matches_xz && !matches_yz {
                uv_matches_position = false;
            }

            uv_variance += uv.0.abs() + uv.1.abs();
        }

        uv_variance /= vertices.len() as f32;

        // 判断映射类型
        if !uv_in_standard_range && uv_variance > 1.0 {
            return "可能是Box/Triplanar映射（UV超出标准范围）".to_string();
        }

        if uv_matches_position {
            return "很可能是Triplanar映射（UV与位置相关）".to_string();
        }
        "标准UV映射".to_string()
    }
}

// 每个mesh都有自己的desc
#[derive(Debug, Clone)]
pub struct Mesh{
    pub id: MeshId,
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    // 顶点数量
    pub vertex_count: u32,
    pub index_count: u32,

    // unity顶点描述
    pub vertex_descriptors: Vec<UnityVertexAttributeDescriptor>,
    // pub pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
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

    fn parse_vertex_buffer(hex_string: &str, size_of: &BufferAddress, vertex_count: usize) -> Vec<u8> {
        // 创建顶点数组
        // let mut vertices: Vec<T> = Vec::with_capacity(vertex_count);
        // // 清理数据
        // let hex_clean: String = hex_string.chars().filter(|c| !c.is_whitespace()).collect();
        //
        // let bytes_per_vertex = 36;
        // let stride = bytes_per_vertex * 2; // 每字节 2 个十六进制字符, unity的数据是16进制
        //
        // for i in 0..vertex_count {
        //     let start = i * stride;
        //     if start + stride > hex_clean.len() {
        //         break;
        //     }
        //
        //     let vertex_hex = &hex_clean[start..start + stride];
        //
        //     // 解析位置 (前12字节 = 24个十六进制字符)
        //     let pos_x = parse_f32_le(&vertex_hex[0..8]);
        //     let pos_y = parse_f32_le(&vertex_hex[8..16]);
        //     let pos_z = parse_f32_le(&vertex_hex[16..24]);
        //
        //     // 解析法向量 (12-24字节)
        //     let norm_x = parse_f32_le(&vertex_hex[24..32]);// offset 12
        //     let norm_y = parse_f32_le(&vertex_hex[32..40]);
        //     let norm_z = parse_f32_le(&vertex_hex[40..48]);
        //
        //     // 解析dimension:4
        //     // let color_r = parse_f32_le(&vertex_hex[48..56]);
        //     // let color_g = parse_f32_le(&vertex_hex[56..64]);
        //     // let color_b = parse_f32_le(&vertex_hex[64..72]);
        //     // let color_a = parse_f32_le(&vertex_hex[72..80]);
        //
        //     // 解析 UV (28-36字节)
        //     let uv_x = parse_f32_le(&vertex_hex[56..64]);
        //     let uv_y = parse_f32_le(&vertex_hex[64..72]);
        //
        // }

        // 移除可能的空格和换行
        let cleaned = hex_string.replace([' ', '\n', '\r'], "");

        // 解码十六进制字符串为字节
        let bytes = hex::decode(cleaned).expect("Invalid hex string");

        println!("Vertex sizeof count: {}", size_of);

        match size_of {
            32 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<Vertex>(), 0);
                let vertices: &[Vertex] = bytemuck::cast_slice(&bytes);

                let mut vertices = vertices.to_vec();

                vertices.iter_mut().for_each(|v| v.flip_z_axis());

                // println!("detect_uv_mapping_32_type(&vertices) :{}", Vertex::detect_uv_mapping_type(&vertices));
                println!("Vertex sizeof: {} count: {}", size_of, vertices.len());
                bytemuck::cast_slice(&vertices).to_vec()
            }
            56 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<VertexTexUvFloat32>(), 0);
                let vertices: &[VertexTexUvFloat32] = bytemuck::cast_slice(&bytes);

                let mut vertices = vertices.to_vec();

                vertices.iter_mut().for_each(|v| v.flip_z_axis());

                // println!("detect_uv_mapping_32_type(&vertices) :{}", Vertex::detect_uv_mapping_type(&vertices));
                println!("Vertex sizeof: {} count: {}", size_of, vertices.len());
                bytemuck::cast_slice(&vertices).to_vec()
            }
            36 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<VertexColor>(), 0);
                let vertices: &[VertexColor] = bytemuck::cast_slice(&bytes);

                let mut vertices = vertices.to_vec();

                vertices.iter_mut().for_each(|v| v.flip_z_axis());

                // println!("detect_uv_mapping_32_type(&vertices) :{}", Vertex::detect_uv_mapping_type(&vertices));
                println!("Vertex sizeof: {} count: {}", size_of, vertices.len());
                bytemuck::cast_slice(&vertices).to_vec()
            }
            40 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<VertexUvFloat1632>(), 0);
                let vertices: &[VertexUvFloat1632] = bytemuck::cast_slice(&bytes);

                let mut vertices = vertices.to_vec();

                vertices.iter_mut().for_each(|v| v.flip_z_axis());

                // println!("detect_uv_mapping_32_type(&vertices) :{}", Vertex::detect_uv_mapping_type(&vertices));
                println!("Vertex sizeof: {} count: {}", size_of, vertices.len());
                bytemuck::cast_slice(&vertices).to_vec()
            }
            48 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<VertexFloat32>(), 0);
                let vertices: &[VertexFloat32] = bytemuck::cast_slice(&bytes);

                let mut vertices = vertices.to_vec();

                vertices.iter_mut().for_each(|v| v.flip_z_axis());

                // println!("detect_uv_mapping_32_type(&vertices) :{}", Vertex::detect_uv_mapping_type(&vertices));
                println!("Vertex sizeof: {} count: {}", size_of, vertices.len());
                bytemuck::cast_slice(&vertices).to_vec()
            }
            80 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<VertexColorUVx3Float32>(), 0);
                let vertices: &[VertexColorUVx3Float32] = bytemuck::cast_slice(&bytes);

                let mut vertices = vertices.to_vec();

                vertices.iter_mut().for_each(|v| v.flip_z_axis());

                // println!("detect_uv_mapping_32_type(&vertices) :{}", Vertex::detect_uv_mapping_type(&vertices));
                println!("Vertex sizeof: {} count: {}", size_of, vertices.len());
                bytemuck::cast_slice(&vertices).to_vec()
            }
            64 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<VertexColorUVFloat32>(), 0);
                let vertices: &[VertexColorUVFloat32] = bytemuck::cast_slice(&bytes);

                let mut vertices = vertices.to_vec();

                vertices.iter_mut().for_each(|v| v.flip_z_axis());

                // println!("detect_uv_mapping_32_type(&vertices) :{}", Vertex::detect_uv_mapping_type(&vertices));
                println!("Vertex sizeof: {} count: {}", size_of, vertices.len());
                bytemuck::cast_slice(&vertices).to_vec()
            }
            _ => {
                println!("Vertex _____ sizeof count: {}", size_of);
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<Vertex>(), 0);
                let vertices: &[Vertex] = bytemuck::cast_slice(&bytes);

                let mut vertices = vertices.to_vec();

                vertices.iter_mut().for_each(|v| v.flip_z_axis());

                println!("__ anther Vertex sizeof: {} count: {}", size_of, vertices.len());

                bytemuck::cast_slice(&vertices).to_vec()
            }
        }

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
    pub async fn from_unity_data(buff: &[u8], id: &MeshId, device: &Device, scene: &Scene, material: &Material, config: &SurfaceConfiguration) -> anyhow::Result<Mesh> {
        let content = std::str::from_utf8(buff)?;
        // 获取mesh文件
        let raw_asset = serde_yaml::from_str::<MeshAsset>(content).map_err(|e| {
            println!("Failed to parse mesh asset: {:?}", e);
            e
        })?;
        let raw = raw_asset.mesh;
        let Some(sub_mesh) = raw.sub_mesh.get(0) else {
            return Err(anyhow::anyhow!("Mesh does not contain sub mesh"));
        };

        let vertex_descriptors = Self::render_descriptors(raw.vertex_data.m_channels);
        print!("{:?},", Self::get_vertex_stride(&vertex_descriptors));

        // let vertices = match Self::get_vertex_stride(&vertex_descriptors) {
        //      32 => {
        //          let bytes = Mesh::parse_vertex_data::<Vertex>(&raw.vertex_data._type_less_data);
        //          let vertices: &[Vertex] = load_vertices::<Vertex>(&bytes);
        //      }
        //     _ => {
        //
        //     }
        // };
        // 处理材质数据
        let size_of = Self::get_vertex_stride(&vertex_descriptors);
        let vertices = Mesh::parse_vertex_buffer(&raw.vertex_data._type_less_data, &size_of, raw.vertex_data.vertex_count);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some(&format!("Mesh_Vertice: {}", raw.m_name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // println!("vertices: {:?}", vertices);
        let indices = Mesh::parse_index_buffer(&raw.index_buffer);
        // println!("analyze_uv_pattern_by_normal(&vertices) :{:?}", Vertex::analyze_uv_pattern_by_normal(&vertices, &indices));
        println!("indices: length {:?},", indices.len());

        // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
        //     label: Some(&format!("Mesh_Vertice: {}", raw.m_name)),
        //     contents: bytemuck::cast_slice(&vertices),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some(&format!("Mesh_Index: {}", raw.m_name)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let render_pipeline = Self::create_render_pipeline(device, scene, config, material, &vertex_descriptors, &raw.m_name);

        Ok(Mesh{
            id: id.clone(),
            name: format!("Mesh: {}", raw.m_name),
            vertex_buffer,
            index_buffer,
            index_count: sub_mesh.index_count,
            vertex_count: sub_mesh.vertex_count,
            vertex_descriptors,
            render_pipeline,
        })
    }

    fn create_render_pipeline(device: &Device, scene: &Scene, config: &SurfaceConfiguration, material: &Material, vertex_descriptors: &Vec<UnityVertexAttributeDescriptor>, label: &String) -> wgpu::RenderPipeline {

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some(&format!("Mesh_PipelineLayout: {}", label)),
            bind_group_layouts: &[
                // 相机
                &scene.camera.bind_group_layout,
                // 环境光 & 背景色
                &scene.scene_bind_group_layout,
                // 光照
                // &scene.light_manager.bind_group_layout,
                // transforms座标系
                // &scene.transform_bind_group_layout,
                
                &material.bind_group_layout,
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
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            // cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        };

        let multisample = wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: Some(&format!("Mesh_Pipeline: {}", label)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState{
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    buffer_layout.as_ref(),
                    InstanceRaw::desc(),
                ],
            },
            primitive,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,  // 近的物体遮挡远的
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
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

        // println!(" Self::attributes: {:?}",  attributes);
        // println!(" Self::get_vertex_stride(vertex_descriptors): {:?}",  Self::get_vertex_stride(vertex_descriptors));
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

// 可能共享, 管理pipe_line
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
#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>, // quaternion
    pub scale: Vector3<f32>,
    // 局部变换矩阵（缓存）
    local_matrix: Matrix4<f32>,
    // 世界变换矩阵（缓存）
    world_matrix: Matrix4<f32>,

    pub is_dirty: bool,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 8,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 9,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 10,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 11,
                },
            ],
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::one(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            local_matrix: Matrix4::identity(),
            world_matrix: Matrix4::identity(),
            is_dirty: true,
        }
    }

    // 计算局部变换矩阵
    pub fn compute_local_matrix(&mut self) {
        if self.is_dirty {
            let translation = Matrix4::from_translation(self.position);
            let rotation = Matrix4::from(self.rotation);
            let scale = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);

            // 变换顺序：缩放 -> 旋转 -> 平移
            self.local_matrix = translation * rotation * scale;
            self.is_dirty = false;
        }
    }

    pub fn set_position(&mut self, pos: &Vector3<f32>) {
        self.position = Vector3::new(pos.x, pos.y, -pos.z);
        self.is_dirty = true;
    }

    pub fn set_rotation(&mut self, rot: Quaternion<f32>) {
        self.rotation = rot;
        self.is_dirty = true;
    }
    pub fn set_scale(&mut self, scale: Vector3<f32>) {
        self.scale = scale;
        self.is_dirty = true;
    }
}

// 每个实体都有一个model， model在scene中管理, 有多个子mesh，暂时处理单个mesh的情况
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Entity(u32);

impl Entity {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }
}

// 保管Transform层级
pub struct TransformSystem {
    // 局部变换
    local_transforms: HashMap<Entity, Transform>,
    // 世界变换（缓存）
    world_matrices: HashMap<Entity, Matrix4<f32>>,
    // 父子关系
    parents: HashMap<Entity, Entity>,
    children: HashMap<Entity, Vec<Entity>>,
}

impl Default for TransformSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformSystem {
    pub fn new() -> Self {
        Self {
            local_transforms: HashMap::new(),
            world_matrices: HashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }
    
    pub fn add_transform(&mut self, entity: Entity, transform: Transform) {
        self.local_transforms.insert(entity, transform);
    }

    pub fn set_parent(&mut self, parent: Entity, child: Entity, ) {
        // println!("Setting parent {:?} to {:?}", parent, child);
        self.parents.insert(child, parent);
        self.children.entry(parent)
            .or_insert_with(Vec::new)
            .push(child);
    }

    // 更新所有Transform
    pub fn update(&mut self) {
        // 找出所有根节点
        let roots: Vec<Entity> = self.local_transforms.keys()
            .filter(|e| !self.parents.contains_key(e))
            .copied()
            .collect();

        // 从根节点开始更新, 更新所有的矩阵
        for root in roots {
            self.update_hierarchy(root, Matrix4::identity());
        }
    }

    fn update_hierarchy(&mut self, entity: Entity, parent_world: Matrix4<f32>) {
        // 获取局部变换
        if let Some(local_transform) = self.local_transforms.get_mut(&entity) {
            local_transform.compute_local_matrix();
            let local_matrix = local_transform.local_matrix;

            // 计算世界变换
            let world_matrix = parent_world * local_matrix;
            self.world_matrices.insert(entity, world_matrix);

            // 递归更新子节点
            if let Some(children) = self.children.get(&entity) {
                let children_vec: Vec<Entity> = children.iter().copied().collect();

                for &child in &children_vec {

                    self.update_hierarchy(child, world_matrix);
                }
            }
        }
    }

    // 获取局部 Transform
    pub fn get_local_transform(&self, entity: Entity) -> Option<&Transform> {
        self.local_transforms.get(&entity)
    }

    // 获取可变的局部 Transform（用于修改）
    pub fn get_local_transform_mut(&mut self, entity: Entity) -> Option<&mut Transform> {
        self.local_transforms.get_mut(&entity)
    }

    // 获取世界变换矩阵
    pub fn get_world_matrix(&self, entity: Entity) -> Option<Matrix4<f32>> {
        self.world_matrices.get(&entity).copied()
    }

    // 获取世界变换矩阵的引用
    pub fn get_world_matrix_ref(&self, entity: Entity) -> Option<&Matrix4<f32>> {
        self.world_matrices.get(&entity)
    }

    // 获取父实体
    pub fn get_parent(&self, entity: Entity) -> Option<Entity> {
        self.parents.get(&entity).copied()
    }

    // 获取所有子实体
    pub fn get_children(&self, entity: Entity) -> Option<&Vec<Entity>> {
        self.children.get(&entity)
    }

    // 检查实体是否存在
    pub fn has_entity(&self, entity: Entity) -> bool {
        self.local_transforms.contains_key(&entity)
    }

    // 移除实体（包括其所有子节点）
    pub fn remove_entity(&mut self, entity: Entity) {
        // 递归移除所有子节点
        if let Some(children) = self.children.remove(&entity) {
            for child in children {
                self.remove_entity(child);
            }
        }

        // 从父节点的子列表中移除
        if let Some(parent) = self.parents.remove(&entity) {
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|&e| e != entity);
            }
        }

        // 移除自身数据
        self.local_transforms.remove(&entity);
        self.world_matrices.remove(&entity);
    }

    // 移除父子关系
    pub fn remove_parent(&mut self, child: Entity) {
        if let Some(parent) = self.parents.remove(&child) {
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|&e| e != child);
            }
        }
    }

    // 获取所有实体
    pub fn get_all_entities(&self) -> Vec<Entity> {
        self.local_transforms.keys().copied().collect()
    }

    // 获取根实体（没有父节点的实体）
    pub fn get_root_entities(&self) -> Vec<Entity> {
        self.local_transforms.keys()
            .filter(|e| !self.parents.contains_key(e))
            .copied()
            .collect()
    }
}