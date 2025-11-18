use serde::{Deserialize, Deserializer, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use wgpu::VertexAttribute;

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    #[serde(rename = "fileID")]
    file_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComponentEntry {
    component: Component,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityGameObject {
    #[serde(rename = "m_Component")]
    m_component: Vec<ComponentEntry>,
    #[serde(rename = "m_Name")]
    m_name: String,
    #[serde(rename = "m_Layer")]
    m_layer: i8,
    #[serde(rename = "m_IsActive")]
    m_is_active: i8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameObject {
    data: UnityGameObject,
    mesh_colliders: Vec<UnityMeshCollider>,
    transforms: Vec<UnityTransform>,
    mesh_filters: Vec<UnityMeshFilter>,
    mesh_renderers: Vec<UnityMeshRenderer>,
    box_colliders: Vec<UnityBoxCollider>,
}

impl GameObject {
    pub fn new(resource: UnityGameObject) -> Self {
        Self {
            data: resource,
            mesh_colliders: vec![],
            transforms: vec![],
            mesh_filters: vec![],
            mesh_renderers: vec![],
            box_colliders: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Position3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityTransform {
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
    #[serde(rename = "m_LocalRotation")]
    m_local_rotation: Position4,
    #[serde(rename = "m_LocalPosition")]
    m_local_position: Position3,
    #[serde(rename = "m_LocalScale")]
    m_local_scale: Position3,
    #[serde(rename = "m_Children")]
    m_children: Vec<Component>,
    #[serde(rename = "m_Father")]
    m_father: Component,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct UnityReference {
    #[serde(rename = "fileID")]
    pub file_id: i64,
    pub guid: String,
    #[serde(rename = "type")]
    pub ref_type: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityMeshRenderer {
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
    #[serde(rename = "m_Materials")]
    m_children: Vec<UnityReference>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UnityMeshFilter {
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
    #[serde(rename = "m_Mesh")]
    m_mesh: UnityReference,
}

impl UnityMeshFilter {
    //
    // pub fn _asset(path: PathBuf) -> Mesh {
    //     Mesh{
    //         name: "",
    //         vertices: ,
    //         index_buffer: (),
    //     }
    // }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityMeshCollider {
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
    #[serde(rename = "m_Mesh")]
    m_mesh: UnityReference,
    #[serde(rename = "m_IsTrigger")]
    m_is_trigger: i8,
    #[serde(rename = "m_Enabled")]
    m_enabled: i8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityBoxCollider {
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
    #[serde(rename = "m_Size")]
    m_size: Position3,
    #[serde(rename = "m_Center")]
    m_center: Position3,
}

// CapsuleCollider
// ParticleSystem
// MonoBehaviour

// 定义一个枚举来表示所有可能的类型
#[derive(Debug, Deserialize)]
#[serde(untagged)] // 尝试匹配第一个成功的变体
enum Asset {
    UnityGameObject(UnityGameObject),
    UnityMeshCollider(UnityMeshCollider),
    UnityTransform(UnityTransform),
    UnityMeshFilter(UnityMeshFilter),
    UnityMeshRenderer(UnityMeshRenderer),
    UnityBoxCollider(UnityBoxCollider),
    // 您可以添加其他可能的类型
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityScene {
    game_objects: Vec<GameObject>,
}

impl UnityScene {
    pub fn new() -> UnityScene {
        Self {
            game_objects: vec![],
        }
    }

    // 从str返回一个Unity场景对象，多个Object，解析.unity
    pub fn from_str(&mut self, file_path: PathBuf) -> anyhow::Result<()> {
        let mut start: bool = false;
        let mut content = String::new();
        let mut old_component_id: i32 = 0;

        let file = File::open(file_path)?;
        let mut mapping_objects: HashMap<i32, GameObject> = HashMap::new();

        let buffer = BufReader::new(file);

        for line_result in buffer.lines() {
            let line = line_result?;
            // --- 开始追加捕获
            if line.starts_with("---") {
                start = true;

                // 为空表示首次数据
                if !content.trim().is_empty() {
                    match serde_yaml::from_str::<Asset>(&content) {
                        Ok(Asset::UnityGameObject(unity_game_object)) => {
                            let game_object = GameObject::new(unity_game_object);
                            mapping_objects.insert(old_component_id, game_object);
                        }
                        Ok(Asset::UnityMeshCollider(mesh_collider)) => {
                            // 改写成宏
                            let game_object = mapping_objects
                                .get_mut(&mesh_collider.m_game_object.file_id)
                                .unwrap();
                            game_object.mesh_colliders.push(mesh_collider);
                        }
                        Ok(Asset::UnityTransform(unity_transform)) => {
                            let game_object = mapping_objects
                                .get_mut(&unity_transform.m_game_object.file_id)
                                .unwrap();
                            game_object.transforms.push(unity_transform);
                        }
                        Ok(Asset::UnityMeshFilter(unity_mesh_filter)) => {
                            let game_object = mapping_objects
                                .get_mut(&unity_mesh_filter.m_game_object.file_id)
                                .unwrap();
                            game_object.mesh_filters.push(unity_mesh_filter);
                        }
                        Ok(Asset::UnityMeshRenderer(unity_mesh_renderer)) => {
                            let game_object = mapping_objects
                                .get_mut(&unity_mesh_renderer.m_game_object.file_id)
                                .unwrap();
                            game_object.mesh_renderers.push(unity_mesh_renderer);
                        }
                        Ok(Asset::UnityBoxCollider(unity_box_collider)) => {
                            let game_object = mapping_objects
                                .get_mut(&unity_box_collider.m_game_object.file_id)
                                .unwrap();
                            game_object.box_colliders.push(unity_box_collider);
                        }
                        _ => {}
                    }
                    content.clear(); // 清空内容
                }
                let tags = line.split(' ');
                // 当前组件的id，一般都是从Object 开始遍历，不用担心顺序的问题
                match tags
                    .collect::<Vec<_>>()
                    .get(2)
                    .unwrap()
                    .replace("&", "")
                    .to_string()
                    .parse::<i32>()
                {
                    Ok(o_id) => old_component_id = o_id,
                    Err(_) => continue,
                }

                continue;
            }
            if !start {
                continue;
            }
            if !line.starts_with(' ') {
                continue;
            }
            content = content + &line;
            content.push('\n');
        }

        let values = mapping_objects.into_values().collect::<Vec<_>>();
        self.game_objects = values;
        // fs::write("./save.json", serde_json::to_string_pretty(self).unwrap())?;

        Ok(())
    }
}

// Unity 顶点属性对应
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnityVertexAttribute {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Color  = 3,
    TexCoord0 = 4,
    TexCoord1 = 5,
    TexCoord2 = 6,
    TexCoord3 = 7,
    BlendWeight = 8,
    BlendIndices = 9,
}

impl UnityVertexAttribute {
    // 反向转换（可选）
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => UnityVertexAttribute::Position,
            1 => UnityVertexAttribute::Normal,
            2 => UnityVertexAttribute::Tangent,
            3 => UnityVertexAttribute::Color,
            4 => UnityVertexAttribute::TexCoord0,
            5 => UnityVertexAttribute::TexCoord1,
            6 => UnityVertexAttribute::TexCoord2,
            7 => UnityVertexAttribute::TexCoord3,
            8 => UnityVertexAttribute::BlendWeight,
            9 => UnityVertexAttribute::BlendIndices,
            _ => UnityVertexAttribute::Position,
        }
    }
}

impl From<UnityVertexAttribute> for u8 {
    
    fn from(attr: UnityVertexAttribute) -> u8 {
        match attr {
            UnityVertexAttribute::Position => 0,
            UnityVertexAttribute::Normal => 1,
            UnityVertexAttribute::Tangent => 2,
            UnityVertexAttribute::Color => 3,
            UnityVertexAttribute::TexCoord0 => 4,
            UnityVertexAttribute::TexCoord1 => 5,
            UnityVertexAttribute::TexCoord2 => 6,
            UnityVertexAttribute::TexCoord3 => 7,
            UnityVertexAttribute::BlendWeight => 8,
            UnityVertexAttribute::BlendIndices => 9,
        }
    }
}

// 顶点的格式，对应asset的format数据
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnityVertexFormat {
    Float32,
    Float16,
    UNorm8,
    SNorm8,
    UNorm16,
    SNorm16,
    UInt8,
    SInt8,
    UInt16,
    SInt16,
    UInt32,
    SInt32,
    Zero
}

impl UnityVertexFormat {
    pub fn from_u8(value: u8) -> Self {
        match value { 
            0 => UnityVertexFormat::Float32,
            1 => UnityVertexFormat::Float16,
            2 => UnityVertexFormat::UNorm8,
            3 => UnityVertexFormat::SNorm8,
            4 => UnityVertexFormat::UNorm16,
            5 => UnityVertexFormat::SNorm16,
            6 => UnityVertexFormat::UInt8,
            7 => UnityVertexFormat::SInt8,
            8 => UnityVertexFormat::UInt16,
            9 => UnityVertexFormat::SInt16,
            10 => UnityVertexFormat::UInt32,
            11 => UnityVertexFormat::SInt32,
            _ => UnityVertexFormat::Zero,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnityVertexAttributeDescriptor {
    // 表示为m_channels的下标 顺序表示
    pub attribute: UnityVertexAttribute,
    pub stream: u8,    // 通常为 0
    pub offset: u8,// 偏移量
    pub format: UnityVertexFormat,
    pub dimension: u8, // 1, 2, 3, 4(52也是4)
}

impl UnityVertexAttributeDescriptor {
    pub fn size_in_bytes(&self) -> u8 {
        // 0 代表不存在
        let code = match self.format {
            UnityVertexFormat::Float32 => 4,
            UnityVertexFormat::Float16 => 2,
            UnityVertexFormat::UNorm8
            | UnityVertexFormat::SNorm8
            | UnityVertexFormat::UInt8
            | UnityVertexFormat::SInt8 => 1,
            UnityVertexFormat::UNorm16
            | UnityVertexFormat::SNorm16
            | UnityVertexFormat::UInt16
            | UnityVertexFormat::SInt16 => 2,
            UnityVertexFormat::UInt32 | UnityVertexFormat::SInt32  => 4,
            UnityVertexFormat::Zero => 0,
        };
        let dimension = if self.dimension == 52 {
            4
        } else {
            self.dimension
        };
        code * dimension
    }

    // 转换至wgpu需要的格式
    pub fn to_wgpu_format(&self) -> Option<wgpu::VertexFormat> {
        // println!("to format: {:?}, diss: {:?}", self.format, self.dimension);
        match (self.format, self.dimension) {
            (UnityVertexFormat::Float32, 1) => Some(wgpu::VertexFormat::Float32),
            (UnityVertexFormat::Float32, 2) => Some(wgpu::VertexFormat::Float32x2),
            (UnityVertexFormat::Float32, 3) => Some(wgpu::VertexFormat::Float32x3),
            (UnityVertexFormat::Float32, 4) => Some(wgpu::VertexFormat::Float32x4),

            (UnityVertexFormat::Float16, 2) => Some(wgpu::VertexFormat::Float16x2),
            (UnityVertexFormat::Float16, 4) => Some(wgpu::VertexFormat::Float16x4),
            (UnityVertexFormat::Float16, 52) => Some(wgpu::VertexFormat::Float16x4), // 52 也是float64

            (UnityVertexFormat::UNorm8, 2) => Some(wgpu::VertexFormat::Unorm8x2),
            (UnityVertexFormat::UNorm8, 4) => Some(wgpu::VertexFormat::Unorm8x4),

            (UnityVertexFormat::SNorm8, 2) => Some(wgpu::VertexFormat::Snorm8x2),
            (UnityVertexFormat::SNorm8, 4) => Some(wgpu::VertexFormat::Snorm8x4),

            (UnityVertexFormat::UInt8, 2) => Some(wgpu::VertexFormat::Uint8x2),
            (UnityVertexFormat::UInt8, 4) => Some(wgpu::VertexFormat::Uint8x4),

            (UnityVertexFormat::SInt8, 2) => Some(wgpu::VertexFormat::Sint8x2),
            (UnityVertexFormat::SInt8, 4) => Some(wgpu::VertexFormat::Sint8x4),

            (UnityVertexFormat::UInt32, 1) => Some(wgpu::VertexFormat::Uint32),
            (UnityVertexFormat::UInt32, 2) => Some(wgpu::VertexFormat::Uint32x2),
            (UnityVertexFormat::UInt32, 3) => Some(wgpu::VertexFormat::Uint32x3),
            (UnityVertexFormat::UInt32, 4) => Some(wgpu::VertexFormat::Uint32x4),

            _ => None,
        }
    }

    // 顺序按照unity
    pub fn shader_location(&self) -> u32 {
        match self.attribute {
            UnityVertexAttribute::Position => 0,
            UnityVertexAttribute::Normal => 1,
            UnityVertexAttribute::Tangent => 2,
            UnityVertexAttribute::Color => 3,
            UnityVertexAttribute::TexCoord0 => 4,
            UnityVertexAttribute::TexCoord1 => 5,
            UnityVertexAttribute::TexCoord2 => 6,
            UnityVertexAttribute::TexCoord3 => 7,
            UnityVertexAttribute::BlendWeight => 8,
            UnityVertexAttribute::BlendIndices => 9,
        }
    }
}
