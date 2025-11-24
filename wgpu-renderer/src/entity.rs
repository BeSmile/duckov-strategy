use std::collections::HashMap;
use cgmath::{Matrix4, One, Point3, Quaternion, SquareMatrix, Transform as CgmathTransform, Vector3, Zero};
use half::f16;
use wgpu::{BufferAddress, Device, Queue, SurfaceConfiguration};
use crate::mesh::Mesh;
use crate::unity::{UnityVertexAttribute, UnityVertexAttributeDescriptor, UnityVertexFormat};

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
pub struct VertexFloat32x6 {    // sizeof 88
    position: [f32; 3],
    normal: [f32; 3], // 法线
    // 切线
    tangent: [f32; 4],
    color: [f32; 4],
    tex_coords: [f32; 4],// uv0坐标
    uv_coords: [f32; 4],// uv1坐标
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

// SM_House05_Roof1 44
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexFloat16x4Float {
    position: [f32; 3],
    normal: [f16; 4], // 法线
    // 切线
    tangent: [f16; 4],
    tex_coords: [f16; 2],// uv坐标
    uv_coords: [f32; 2],// uv1坐标
    uv1_coords: [f16; 2],// uv2坐标
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
pub struct VertexColorFloat3x4U8 {    // size_of: 52
    position: [f32; 3],
    normal: [f32; 3], // 法线
    // 切线
    tangent: [f32; 4],
    color: [u8; 4], // 颜色
    tex_coords: [f32; 2],// uv0坐标
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

pub trait IVertex {
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
impl IVertex for VertexFloat16x4Float {
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

        let u0 = self.uv_coords[0];
        let v0 = self.uv_coords[1];

        // 使用fract()获取小数部分，映射到[0,1]
        self.uv_coords[0] = u0.fract().abs();
        self.uv_coords[1] = v0.fract().abs();

        let u1 = self.uv1_coords[0].to_f32();
        let v1 = self.uv1_coords[1].to_f32();

        // 使用fract()获取小数部分，映射到[0,1]
        self.uv1_coords[0] = f16::from_f32(u1.fract().abs());
        self.uv1_coords[1] = f16::from_f32(v1.fract().abs());

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

impl IVertex for VertexColorFloat3x4U8 {
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
impl IVertex for VertexFloat32x6 {
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
        
        // todo uv 4个如何处理？
        let u1 = self.tex_coords[0];
        let v1 = self.tex_coords[1];

        // 使用fract()获取小数部分，映射到[0,1]
        self.uv_coords[0] = u1.fract().abs();
        self.uv_coords[1] = v1.fract().abs();
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


// 可能共享, 管理pipe_line
pub struct Model{
    pub id: usize,
    pub name: String,
    pub meshs: Vec<Mesh>
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
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
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
    pub fn update(&mut self, entity_display_map: &mut HashMap<Entity, bool>) {
        // 找出所有根节点
        let roots: Vec<Entity> = self.local_transforms.keys()
            .filter(|e| !self.parents.contains_key(e))
            .copied()
            .collect();

        println!("all roots: {:?}", roots);
        // 从根节点开始更新, 更新所有的矩阵
        for root in roots {
            let display = entity_display_map.get(&root).copied().unwrap_or(true);
            self.update_hierarchy(root, Matrix4::identity(), entity_display_map, display);
        }
    }

    fn update_hierarchy(&mut self, entity: Entity, parent_world: Matrix4<f32>, entity_display_map: &mut HashMap<Entity, bool>, show: bool) {
        // 递归如果父组件隐藏 或当前组件隐藏，则直接隐藏
        let hidden = !show || !entity_display_map.get(&entity).copied().unwrap_or(true);
        if hidden {
            entity_display_map.insert(entity, false);
        }
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

                    self.update_hierarchy(child, world_matrix, entity_display_map, !hidden);
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