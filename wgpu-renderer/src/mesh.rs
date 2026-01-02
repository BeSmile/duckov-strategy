use cgmath::{Matrix4, Point3, Transform};
use wgpu::{BufferAddress, Device, SurfaceConfiguration};
use wgpu::util::DeviceExt;
use crate::entity::{InstanceRaw, IVertex, Vertex, VertexBufferLayoutOwned, VertexColor, VertexColorFloat3x4U8, VertexColorUVFloat32, VertexColorUVx3Float32, VertexFloat32, VertexTexUvFloat32, VertexUvFloat1632, VertexFloat16x4Float, VertexFloat32x6, VertexColorUv32, VertexColorUv32f};
use crate::materials::{Material, Texture};
use crate::resource::MeshId;
use crate::scene::Scene;
use crate::unity::{Channel, MeshAsset, UnityVertexAttribute, UnityVertexAttributeDescriptor, UnityVertexFormat};

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl AABB {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }
    /// 从 Unity 格式 (Center + Extent) 创建
    /// From Unity format (Center + Extent) with Z-flip for right-handed system
    pub fn from_unity(center: &Point3<f32>, extent: &Point3<f32>) -> Self {
        // Flip Z axis
        let center_z = -center[2];

        Self {
            min: Point3::new(
                center[0] - extent[0],
                center[1] - extent[1],
                center_z - extent[2],  // extent is always positive
            ),
            max: Point3::new(
                center[0] + extent[0],
                center[1] + extent[1],
                center_z + extent[2],
            ),
        }
    }

    /// Transform AABB by a matrix (handles rotation/scale properly)
    pub fn transform(&self, matrix: &Matrix4<f32>) -> Self {
        // Get all 8 corner points of the AABB
        let corners = [
            Point3::new(self.min.x, self.min.y, self.min.z),
            Point3::new(self.max.x, self.min.y, self.min.z),
            Point3::new(self.min.x, self.max.y, self.min.z),
            Point3::new(self.max.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.min.y, self.max.z),
            Point3::new(self.max.x, self.min.y, self.max.z),
            Point3::new(self.min.x, self.max.y, self.max.z),
            Point3::new(self.max.x, self.max.y, self.max.z),
        ];

        // Transform all corners and find new min/max
        let mut new_min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut new_max = Point3::new(f32::MIN, f32::MIN, f32::MIN);

        for corner in &corners {
            let transformed = matrix.transform_point(*corner);

            new_min.x = new_min.x.min(transformed.x);
            new_min.y = new_min.y.min(transformed.y);
            new_min.z = new_min.z.min(transformed.z);

            new_max.x = new_max.x.max(transformed.x);
            new_max.y = new_max.y.max(transformed.y);
            new_max.z = new_max.z.max(transformed.z);
        }

        Self {
            min: new_min,
            max: new_max,
        }
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

    pub aabb: AABB,
}

// todo 处理mesh多个材质 HiddenWarehouse场景下 SM_House_03_Roof1 物体
// -[ ] 处理材质颜色的问题
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
            52 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<VertexColorFloat3x4U8>(), 0);
                let vertices: &[VertexColorFloat3x4U8] = bytemuck::cast_slice(&bytes);

                let mut vertices = vertices.to_vec();

                vertices.iter_mut().for_each(|v| v.flip_z_axis());

                // println!("detect_uv_mapping_32_type(&vertices) :{}", Vertex::detect_uv_mapping_type(&vertices));
                println!("Vertex sizeof: {} count: {}", size_of, vertices.len());
                bytemuck::cast_slice(&vertices).to_vec()
            }
            44 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<VertexFloat16x4Float>(), 0);
                let vertices: &[VertexFloat16x4Float] = bytemuck::cast_slice(&bytes);

                let mut vertices = vertices.to_vec();

                vertices.iter_mut().for_each(|v| v.flip_z_axis());

                // println!("detect_uv_mapping_32_type(&vertices) :{}", Vertex::detect_uv_mapping_type(&vertices));
                bytemuck::cast_slice(&vertices).to_vec()
            }
            76 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<VertexColorUv32>(), 0);
                let vertices: &[VertexColorUv32] = bytemuck::cast_slice(&bytes);

                let mut vertices = vertices.to_vec();

                vertices.iter_mut().for_each(|v| v.flip_z_axis());

                // println!("detect_uv_mapping_32_type(&vertices) :{}", Vertex::detect_uv_mapping_type(&vertices));
                println!("Vertex sizeof: {} count: {}", size_of, vertices.len());
                bytemuck::cast_slice(&vertices).to_vec()
            }
            72 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<VertexColorUv32f>(), 0);
                let vertices: &[VertexColorUv32f] = bytemuck::cast_slice(&bytes);

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
            88 => {
                // 检查字节数是否是顶点大小的整数倍
                assert_eq!(bytes.len() % std::mem::size_of::<VertexFloat32x6>(), 0);
                let vertices: &[VertexFloat32x6] = bytemuck::cast_slice(&bytes);

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

    /// 创建默认的 Cube mesh（1x1x1 立方体，中心在原点）
    pub fn create_default_cube(
        id: &MeshId,
        device: &Device,
        scene: &Scene,
        material: &Material,
        config: &SurfaceConfiguration,
    ) -> Mesh {
        // Cube 有 24 个顶点（每个面 4 个顶点，共 6 个面）
        // 每个面需要独立的顶点以确保法线正确
        let vertices: Vec<f32> = vec![
            // 前面 (Z+) - 法线指向 (0, 0, 1)
            -0.5, -0.5,  0.5,    0.0,  0.0,  1.0,    0.0, 0.0,  // 0
            0.5, -0.5,  0.5,    0.0,  0.0,  1.0,    1.0, 0.0,  // 1
            0.5,  0.5,  0.5,    0.0,  0.0,  1.0,    1.0, 1.0,  // 2
            -0.5,  0.5,  0.5,    0.0,  0.0,  1.0,    0.0, 1.0,  // 3

            // 后面 (Z-) - 法线指向 (0, 0, -1)
            0.5, -0.5, -0.5,    0.0,  0.0, -1.0,    0.0, 0.0,  // 4
            -0.5, -0.5, -0.5,    0.0,  0.0, -1.0,    1.0, 0.0,  // 5
            -0.5,  0.5, -0.5,    0.0,  0.0, -1.0,    1.0, 1.0,  // 6
            0.5,  0.5, -0.5,    0.0,  0.0, -1.0,    0.0, 1.0,  // 7

            // 上面 (Y+) - 法线指向 (0, 1, 0)
            -0.5,  0.5,  0.5,    0.0,  1.0,  0.0,    0.0, 0.0,  // 8
            0.5,  0.5,  0.5,    0.0,  1.0,  0.0,    1.0, 0.0,  // 9
            0.5,  0.5, -0.5,    0.0,  1.0,  0.0,    1.0, 1.0,  // 10
            -0.5,  0.5, -0.5,    0.0,  1.0,  0.0,    0.0, 1.0,  // 11

            // 下面 (Y-) - 法线指向 (0, -1, 0)
            -0.5, -0.5, -0.5,    0.0, -1.0,  0.0,    0.0, 0.0,  // 12
            0.5, -0.5, -0.5,    0.0, -1.0,  0.0,    1.0, 0.0,  // 13
            0.5, -0.5,  0.5,    0.0, -1.0,  0.0,    1.0, 1.0,  // 14
            -0.5, -0.5,  0.5,    0.0, -1.0,  0.0,    0.0, 1.0,  // 15

            // 右面 (X+) - 法线指向 (1, 0, 0)
            0.5, -0.5,  0.5,    1.0,  0.0,  0.0,    0.0, 0.0,  // 16
            0.5, -0.5, -0.5,    1.0,  0.0,  0.0,    1.0, 0.0,  // 17
            0.5,  0.5, -0.5,    1.0,  0.0,  0.0,    1.0, 1.0,  // 18
            0.5,  0.5,  0.5,    1.0,  0.0,  0.0,    0.0, 1.0,  // 19

            // 左面 (X-) - 法线指向 (-1, 0, 0)
            -0.5, -0.5, -0.5,   -1.0,  0.0,  0.0,    0.0, 0.0,  // 20
            -0.5, -0.5,  0.5,   -1.0,  0.0,  0.0,    1.0, 0.0,  // 21
            -0.5,  0.5,  0.5,   -1.0,  0.0,  0.0,    1.0, 1.0,  // 22
            -0.5,  0.5, -0.5,   -1.0,  0.0,  0.0,    0.0, 1.0,  // 23
        ];

        // 12 个三角形（6 个面 × 2 个三角形），36 个索引
        let indices: Vec<u16> = vec![
            // 前面
            0, 1, 2,    0, 2, 3,
            // 后面
            4, 5, 6,    4, 6, 7,
            // 上面
            8, 9, 10,   8, 10, 11,
            // 下面
            12, 13, 14, 12, 14, 15,
            // 右面
            16, 17, 18, 16, 18, 19,
            // 左面
            20, 21, 22, 20, 22, 23,
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Default_Cube_Vertex"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Default_Cube_Index"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let vertex_descriptors = vec![
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::Position,
                stream: 0,
                offset: 0,
                format: UnityVertexFormat::Float32,
                dimension: 3,
            },
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::Normal,
                stream: 0,
                offset: 12,
                format: UnityVertexFormat::Float16,
                dimension: 4,
            },
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::Tangent,
                stream: 0,
                offset: 20,
                format: UnityVertexFormat::Float16,
                dimension: 4,
            },
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::TexCoord0,
                stream: 0,
                offset: 28,
                format: UnityVertexFormat::Float16,
                dimension: 2,
            },
        ];

        let render_pipeline = Self::create_render_pipeline(
            device,
            scene,
            config,
            material,
            &vertex_descriptors,
            &"Default_Cube".to_string(),
        );

        Mesh {
            id: id.clone(),
            name: "Default_Cube".to_string(),
            vertex_buffer,
            index_buffer,
            index_count: 36,
            vertex_count: 24,
            vertex_descriptors,
            render_pipeline,
            aabb: AABB::new(
                Point3::new(0.0, 0.0, 0.0),  // center
                Point3::new(0.5, 0.5, 0.5),  // half extents
            ),
        }
    }

    /// 创建默认的 Quad mesh（1x1 平面，中心在原点）
    pub fn create_default_quad(
        id: &MeshId,
        device: &Device,
        scene: &Scene,
        material: &Material,
        config: &SurfaceConfiguration,
    ) -> Mesh {
        // Quad 的 4 个顶点（位置 + 法线 + UV）
        // 假设你的 Vertex 结构是 32 bytes: position(12) + normal(12) + uv(8)
        let vertices: Vec<f32> = vec![
            // position (x, y, z)    normal (x, y, z)       uv (u, v)
            -0.5, -0.5, 0.0,         0.0, 0.0, -1.0,        0.0, 0.0,  // 左下
            0.5, -0.5, 0.0,         0.0, 0.0, -1.0,        1.0, 0.0,  // 右下
            -0.5, 0.5, 0.0,         0.0, 0.0, -1.0,        0.0, 1.0,  // 左上
            0.5,  0.5, 0.0,         0.0, 0.0, -1.0,        1.0, 1.0,  // 右上
        ];

        // 2 个三角形，6 个索引（顺时针或逆时针取决于你的 cull mode）
        let indices: Vec<u16> = vec![
            0, 2, 1,  // 第一个三角形
            1, 2, 3,  // 第二个三角形
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Default_Quad_Vertex"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Default_Quad_Index"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // 默认的顶点描述符（position + normal + uv）
        let vertex_descriptors = vec![
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::Position,
                stream: 0,
                offset: 0,
                format: UnityVertexFormat::Float32,
                dimension: 3,
            },
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::Normal,
                stream: 0,
                offset: 12,
                format: UnityVertexFormat::Float16,
                dimension: 4,
            },
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::Tangent,
                stream: 0,
                offset: 20,
                format: UnityVertexFormat::Float16,
                dimension: 4,
            },
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::TexCoord0,
                stream: 0,
                offset: 28,
                format: UnityVertexFormat::Float16,
                dimension: 2,
            },
        ];

        let render_pipeline = Self::create_render_pipeline(
            device,
            scene,
            config,
            material,
            &vertex_descriptors,
            &"Default_Quad".to_string(),
        );

        Mesh {
            id: id.clone(),
            name: "Default_Quad".to_string(),
            vertex_buffer,
            index_buffer,
            index_count: 6,
            vertex_count: 4,
            vertex_descriptors,
            render_pipeline,
            aabb: AABB::new(
                Point3::new(0.0, 0.0, 0.0),  // center
                Point3::new(0.5, 0.5, 0.0),  // half extents
            ),
        }
    }

    /// Create default Plane mesh (10x10 units, 10x10 segments)
    pub fn create_default_plane(
        id: &MeshId,
        device: &Device,
        scene: &Scene,
        material: &Material,
        config: &SurfaceConfiguration,
    ) -> Mesh {
        let width = 10.0;
        let height = 10.0;
        let segments_x = 10;
        let segments_y = 10;
        let width_half = width / 2.0;
        let height_half = height / 2.0;
        let segment_width = width / segments_x as f32;
        let segment_height = height / segments_y as f32;

        let num_vertices = (segments_x + 1) * (segments_y + 1);
        let mut vertices: Vec<f32> = Vec::with_capacity(num_vertices as usize * 12);
        let mut indices: Vec<u16> = Vec::new();

        for y in 0..=segments_y {
            for x in 0..=segments_x {
                let x_pos = (x as f32 * segment_width) - width_half;
                let z_pos = (y as f32 * segment_height) - height_half;
                
                // Position
                vertices.push(x_pos);
                vertices.push(0.0);
                vertices.push(z_pos);

                // Normal (Up)
                vertices.push(0.0);
                vertices.push(1.0);
                vertices.push(0.0);

                // Tangent (Right)
                vertices.push(1.0);
                vertices.push(0.0);
                vertices.push(0.0);
                vertices.push(1.0);

                // UV
                vertices.push(x as f32 / segments_x as f32);
                vertices.push(1.0 - (y as f32 / segments_y as f32)); // Flip V to match Unity? Unity Plane V grows same direction as Z?
                // Unity Plane: (0,0) at min, (1,1) at max.
                // Our Z grows.
            }
        }

        for y in 0..segments_y {
            for x in 0..segments_x {
                let a = (segments_x + 1) * y + x;
                let b = (segments_x + 1) * (y + 1) + x;
                let c = (segments_x + 1) * (y + 1) + (x + 1);
                let d = (segments_x + 1) * y + (x + 1);

                indices.push(a as u16);
                indices.push(d as u16);
                indices.push(b as u16);

                indices.push(b as u16);
                indices.push(d as u16);
                indices.push(c as u16);
            }
        }

        Self::create_mesh_from_data(id, "Default_Plane", device, scene, material, config, vertices, indices, AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(5.0, 0.0, 5.0)))
    }

   /// Create default Sphere mesh
    pub fn create_default_sphere(
        id: &MeshId,
        device: &Device,
        scene: &Scene,
        material: &Material,
        config: &SurfaceConfiguration,
    ) -> Mesh {
        let radius = 0.5;
        let lat_segments = 24;
        let long_segments = 24;

        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        for lat in 0..=lat_segments {
            let theta = lat as f32 * std::f32::consts::PI / lat_segments as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for lon in 0..=long_segments {
                let phi = lon as f32 * 2.0 * std::f32::consts::PI / long_segments as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = cos_phi * sin_theta;
                let y = cos_theta;
                let z = sin_phi * sin_theta;

                let u = 1.0 - (lon as f32 / long_segments as f32);
                let v = lat as f32 / lat_segments as f32;

                // Position
                vertices.push(x * radius);
                vertices.push(y * radius);
                vertices.push(z * radius);

                // Normal
                vertices.push(x);
                vertices.push(y);
                vertices.push(z);

                // Tangent
                // Tangent is along longitude (derivative wrt phi)
                // dx/dphi = -sin_phi * sin_theta
                // dy/dphi = 0
                // dz/dphi = cos_phi * sin_theta
                vertices.push(-sin_phi);
                vertices.push(0.0);
                vertices.push(cos_phi);
                vertices.push(1.0);

                // UV
                vertices.push(u);
                vertices.push(v);
            }
        }

        for lat in 0..lat_segments {
            for lon in 0..long_segments {
                let first = (lat * (long_segments + 1)) + lon;
                let second = first + long_segments + 1;

                indices.push(first as u16);
                indices.push(second as u16);
                indices.push((first + 1) as u16);

                indices.push(second as u16);
                indices.push((second + 1) as u16);
                indices.push((first + 1) as u16);
            }
        }

        Self::create_mesh_from_data(id, "Default_Sphere", device, scene, material, config, vertices, indices, AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.5, 0.5, 0.5)))
    }

    /// Create default Capsule mesh
    pub fn create_default_capsule(
        id: &MeshId,
        device: &Device,
        scene: &Scene,
        material: &Material,
        config: &SurfaceConfiguration,
    ) -> Mesh {
        let radius = 0.5;
        let height = 2.0;
        let segments = 24;
        let rings_cap = 8;
        let rings_body = 1; 

        // Helper to add vertex
        fn add_vertex(vertices: &mut Vec<f32>, x: f32, y: f32, z: f32, nx: f32, ny: f32, nz: f32, u: f32, v: f32) {
             // Position
             vertices.push(x);
             vertices.push(y);
             vertices.push(z);
             
             // Normal
             vertices.push(nx);
             vertices.push(ny);
             vertices.push(nz);

             // Tangent (approximate, planar projection on XZ)
             let len = (nx * nx + nz * nz).sqrt();
             if len > 0.001 {
                vertices.push(-nz / len);
                vertices.push(0.0);
                vertices.push(nx / len);
             } else {
                 vertices.push(1.0);
                 vertices.push(0.0);
                 vertices.push(0.0);
             }
             vertices.push(1.0);

             // UV
             vertices.push(u);
             vertices.push(v);
        }

        let cylinder_height = height - 2.0 * radius;
        let sub_height = cylinder_height;
        
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u16> = Vec::new(); // indices unused in generation logic? No, used at end.

        let rings_total = rings_cap * 2 + 1; // +1 region for body (2 extra rings of vertices? No, standard sphere logic stretches)
        
         for r in 0..=rings_total {
             let v_ratio = r as f32 / rings_total as f32;
             
             // Determine phi and y_offset based on r
             let (phi, y_offset) = if r <= rings_cap {
                 // Top hemisphere (phi 0 to pi/2)
                  let p = r as f32 * std::f32::consts::PI / (2.0 * rings_cap as f32);
                  (p, sub_height / 2.0)
             } else {
                 // Bottom hemisphere
                 // Map (rings_cap + 1) -> PI/2
                 // Map (rings_total) -> PI
                 // But wait, the "cylinder" body needs to be represented.
                 // A simple way is to split the loop.
                 // But using continuous loop:
                 // r = rings_cap -> phi = PI/2, y = +half
                 // r = rings_cap + 1 -> phi = PI/2, y = -half  <-- This creates the cylinder side!
                 // BUT phi is same, so normals are same.
                 
                 let r_local = r - 1; // shift back
                 let p = r_local as f32 * std::f32::consts::PI / (2.0 * rings_cap as f32);
                 (p, -sub_height / 2.0)
             };
             
              for s in 0..=segments {
                 let u_ratio = 1.0 - (s as f32 / segments as f32);
                 let theta = s as f32 * 2.0 * std::f32::consts::PI / segments as f32;
                 
                 let sin_phi = phi.sin();
                 let cos_phi = phi.cos();
                 let sin_theta = theta.sin();
                 let cos_theta = theta.cos();
                 
                 let nx = cos_theta * sin_phi;
                 let ny = cos_phi;
                 let nz = sin_theta * sin_phi;
                 
                 let x = nx * radius;
                 let y = ny * radius + y_offset;
                 let z = nz * radius;
                 
                add_vertex(&mut vertices, x, y, z, nx, ny, nz, u_ratio, v_ratio);
             }
         }
         
         // Generate indices
          for r in 0..rings_total {
            for s in 0..segments {
                let first = (r * (segments + 1)) + s;
                let second = first + segments + 1;

                indices.push(first as u16);
                indices.push(second as u16);
                indices.push((first + 1) as u16);

                indices.push(second as u16);
                indices.push((second + 1) as u16);
                indices.push((first + 1) as u16);
            }
        }
        
        Self::create_mesh_from_data(id, "Default_Capsule", device, scene, material, config, vertices, indices, AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.5, 1.0, 0.5)))
    }
    
    /// Create default Cylinder mesh
    pub fn create_default_cylinder(
        id: &MeshId,
        device: &Device,
        scene: &Scene,
        material: &Material,
        config: &SurfaceConfiguration,
    ) -> Mesh {
        let radius = 0.5;
        let height = 2.0;
        let segments = 20;
        
        let half_height = height / 2.0;
        
        let mut vertices: Vec<f32> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        fn add_vertex(vertices: &mut Vec<f32>, x: f32, y: f32, z: f32, nx: f32, ny: f32, nz: f32, u: f32, v: f32) {
             // Position
             vertices.push(x);
             vertices.push(y);
             vertices.push(z);
             vertices.push(nx);
             vertices.push(ny);
             vertices.push(nz);
             // Tangent
             let len = (nx * nx + nz * nz).sqrt();
             if len > 0.001 {
                vertices.push(-nz / len);
                vertices.push(0.0);
                vertices.push(nx / len);
             } else {
                 vertices.push(1.0);
                 vertices.push(0.0);
                 vertices.push(0.0);
             }
             vertices.push(1.0);
             // UV
             vertices.push(u);
             vertices.push(v);
        }
        
        // Side
        for s in 0..=segments {
            let u = 1.0 - (s as f32 / segments as f32);
            let theta = s as f32 * 2.0 * std::f32::consts::PI / segments as f32;
            let nx = theta.cos();
            let nz = theta.sin();
            let x = nx * radius;
            let z = nz * radius;
            
            // Top edge
            add_vertex(&mut vertices, x, half_height, z, nx, 0.0, nz, u, 0.0);
            // Bottom edge
            add_vertex(&mut vertices, x, -half_height, z, nx, 0.0, nz, u, 1.0);
        }
        
        for s in 0..segments {
            let top1 = s * 2;
            let bot1 = s * 2 + 1;
            let top2 = (s + 1) * 2;
            let bot2 = (s + 1) * 2 + 1;
            
            indices.push(top1 as u16);
            indices.push(bot2 as u16);
            indices.push(bot1 as u16);

            indices.push(top1 as u16);
            indices.push(top2 as u16);
            indices.push(bot2 as u16);
        }
        
        let offset_cap_top = vertices.len() as u16 / 12;
        // Top Cap
        // Center
        add_vertex(&mut vertices, 0.0, half_height, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5);
        for s in 0..=segments {
             let theta = s as f32 * 2.0 * std::f32::consts::PI / segments as f32;
             let x = theta.cos() * radius;
             let z = theta.sin() * radius;
             let u = (theta.cos() + 1.0) * 0.5; // Planar mapping
             let v = (theta.sin() + 1.0) * 0.5;
             add_vertex(&mut vertices, x, half_height, z, 0.0, 1.0, 0.0, u, v);
        }
        
        for s in 0..segments {
            indices.push(offset_cap_top); // Center
            indices.push(offset_cap_top + 1 + s as u16);
            indices.push(offset_cap_top + 1 + (s + 1) as u16);
        }
        
        let offset_cap_bot = vertices.len() as u16 / 12;
        // Bottom Cap
        add_vertex(&mut vertices, 0.0, -half_height, 0.0, 0.0, -1.0, 0.0, 0.5, 0.5);
         for s in 0..=segments {
             let theta = s as f32 * 2.0 * std::f32::consts::PI / segments as f32;
             let x = theta.cos() * radius;
             let z = theta.sin() * radius;
               let u = (theta.cos() + 1.0) * 0.5; 
             let v = (theta.sin() + 1.0) * 0.5;
             add_vertex(&mut vertices, x, -half_height, z, 0.0, -1.0, 0.0, u, v);
        }
        
        for s in 0..segments {
            indices.push(offset_cap_bot); // Center
            indices.push(offset_cap_bot + 1 + (s + 1) as u16);
            indices.push(offset_cap_bot + 1 + s as u16);
        }

         Self::create_mesh_from_data(id, "Default_Cylinder", device, scene, material, config, vertices, indices, AABB::new(Point3::new(0.0, 0.0, 0.0), Point3::new(0.5, 1.0, 0.5)))
    }
    
    // Internal helper for these default meshes
    fn create_mesh_from_data(
        id: &MeshId,
        name: &str,
        device: &Device,
        scene: &Scene,
        material: &Material,
        config: &SurfaceConfiguration,
        vertices: Vec<f32>,
        indices: Vec<u16>,
        aabb: AABB
    ) -> Mesh {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_Vertex", name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_Index", name)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let vertex_descriptors = vec![
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::Position,
                stream: 0,
                offset: 0,
                format: UnityVertexFormat::Float32,
                dimension: 3,
            },
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::Normal,
                stream: 0,
                offset: 12,
                format: UnityVertexFormat::Float32,
                dimension: 3,
            },
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::Tangent,
                stream: 0,
                offset: 24,
                format: UnityVertexFormat::Float32,
                dimension: 4,
            },
            UnityVertexAttributeDescriptor {
                attribute: UnityVertexAttribute::TexCoord0,
                stream: 0,
                offset: 40,
                format: UnityVertexFormat::Float32,
                dimension: 2,
            },
        ];

        let render_pipeline = Self::create_render_pipeline(
            device,
            scene,
            config,
            material,
            &vertex_descriptors,
            &name.to_string(),
        );

        Mesh {
            id: id.clone(),
            name: name.to_string(),
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            vertex_count: (vertices.len() / 12) as u32,
            vertex_descriptors,
            render_pipeline,
            aabb,
        }
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
            aabb: AABB::from_unity(&raw.m_local_aabb.m_center, &raw.m_local_aabb.m_extent),
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