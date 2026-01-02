use serde::{de, Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::{fs};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use cgmath::{Point3, Vector3};
use log::info;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    #[serde(rename = "fileID")]
    pub file_id: u32,
}

pub trait UnityAsset {
    // fn set_file_id(&mut self, file_id: u32);
    fn name(&self) -> &'static str;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentEntry {
    pub component: Component,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityGameObject {
    // #[serde(skip_deserializing)]
    // file_id: u32,
    // 表示当前挂载的组件，包括mesh,transform,脚本组件等
    #[serde(rename = "m_Component")]
    pub m_component: Vec<ComponentEntry>,
    #[serde(rename = "m_Name")]
    pub m_name: String,
    #[serde(rename = "m_Layer")]
    m_layer: i8,
    #[serde(rename = "m_IsActive")]
    pub m_is_active: i8,
}

impl UnityAsset for UnityGameObject {
    // fn set_file_id(&mut self, file_id: u32) {
    //     self.file_id = file_id;
    // }
    fn name(&self) -> &'static str {
        "UnityGameObject"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityScene {
    // 归类各个类中，提前命中数据
    // pub game_object: HashMap<u32, UnityGameObject>,
    pub game_object_raw: HashMap<u32, String>,// 保存原始的string
    // pub mesh_colliders: HashMap<u32, UnityMeshCollider>,
    pub mesh_colliders_raw: HashMap<u32, String>,// 保存原始的string

    // pub transforms: HashMap<u32, UnityTransform>,
    pub transforms_raw: HashMap<u32, String>,// 保存原始的string

    // pub mesh_filters: HashMap<u32, UnityMeshFilter>,
    pub mesh_filters_raw: HashMap<u32, String>,// 保存原始的string

    // pub mesh_renderers: HashMap<u32, UnityMeshRenderer>,
    pub mesh_renderers_raw: HashMap<u32, String>,// 保存原始的string
    // pub lights: HashMap<u32, UnityLight>,
    pub lights_raw: HashMap<u32, String>,// 保存原始的string
    pub soda_lights: HashMap<u32, UnitySodaPointLight>,
    pub box_colliders: HashMap<u32, UnityBoxCollider>,// 不太需要
    pub index: HashMap<u32, String>,// 只保留索引
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
    #[serde(skip_deserializing)]
    file_id: u32,
    #[serde(rename = "m_GameObject")]
    pub m_game_object: Component,
    #[serde(rename = "m_LocalRotation")]
    pub m_local_rotation: Position4,
    #[serde(rename = "m_LocalPosition")]
    pub m_local_position: Vector3<f32>,
    #[serde(rename = "m_LocalScale")]
    pub m_local_scale: Position3,
    #[serde(rename = "m_Children")]
    pub m_children: Vec<Component>,
    #[serde(rename = "m_Father")]
    pub m_father: Option<Component>,
}

impl UnityAsset for UnityTransform {
    // fn set_file_id(&mut self, file_id: u32) {
    //     self.file_id = file_id;
    // }
    fn name(&self) -> &'static str {
        "UnityTransform"
    }
}


#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct UnityReference {
    #[serde(rename = "fileID")]
    pub file_id: i64,
    // serde的bug， 需要看源码，提mr
    // #[serde(deserialize_with = "deserialize_guid", default)]
    pub guid: String,
    #[serde(rename = "type")]
    pub ref_type: i32,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct TextureReference {
    #[serde(rename = "fileID")]
    pub file_id: i64,
    pub guid: Option<String>,
    #[serde(rename = "type")]
    pub ref_type: Option<i32>,
}

// 颜色数据
#[derive(Debug, Deserialize, Serialize)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityMeshRenderer {
    // #[serde(skip_deserializing)]
    // file_id: u32,
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
    #[serde(rename = "m_Enabled")]
    pub m_enabled: u8,
    #[serde(rename = "m_Materials")]
    pub m_children: Vec<UnityReference>,
}

impl UnityAsset for UnityMeshRenderer {
    // fn set_file_id(&mut self, file_id: u32) {
    //     self.file_id = file_id;
    // }
    fn name(&self) -> &'static str {
        "UnityMeshRenderer"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityMeshFilter {
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
    #[serde(rename = "m_Mesh")]
    pub m_mesh: UnityReference,
}

impl UnityAsset for UnityMeshFilter {
    // fn set_file_id(&mut self, file_id: u32) {
    //     self.file_id = file_id;
    // }
    fn name(&self) -> &'static str {
        "UnityMeshFilter"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Local_AABB{
    #[serde(rename = "m_Center")]
    pub m_center: Point3<f32>,
    #[serde(rename = "m_Extent")]
    pub m_extent: Point3<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityMeshCollider {
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,// 表示挂载的game_object的file_id
    #[serde(rename = "m_Mesh")]
    m_mesh: UnityReference,
    #[serde(rename = "m_IsTrigger")]
    m_is_trigger: i8,
    #[serde(rename = "m_Enabled")]
    m_enabled: i8,
}

impl UnityAsset for UnityMeshCollider {
    // fn set_file_id(&mut self, file_id: u32) {
    //     self.file_id = file_id;
    // }
    fn name(&self) -> &'static str {
        "UnityMeshCollider"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityBoxCollider {

    #[serde(skip_deserializing)]
    file_id: u32,
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
    #[serde(rename = "m_Size")]
    m_size: Position3,
    #[serde(rename = "m_Center")]
    m_center: Position3,
}

impl UnityAsset for UnityBoxCollider {
    // fn set_file_id(&mut self, file_id: u32) {
    //     self.file_id = file_id;
    // }
    fn name(&self) -> &'static str {
        "UnityBoxCollider"
    }
}
impl UnityAsset for UnityLight {
    // fn set_file_id(&mut self, file_id: u32) {
    //     self.file_id = file_id;
    // }
    fn name(&self) -> &'static str {
        "UnityLight"
    }
}

// CapsuleCollider
// ParticleSystem
// MonoBehaviour

#[derive(Debug, Serialize, Deserialize)]
pub struct UnityLight {
    // #[serde(rename = "m_ObjectHideFlags")]
    // pub object_hide_flags: i32,

    #[serde(rename = "m_GameObject")]
    pub game_object: Component,

    #[serde(rename = "m_Enabled")]
    pub enabled: i32,

    #[serde(rename = "serializedVersion")]
    pub serialized_version: i32,

    #[serde(rename = "m_Type")]
    pub light_type: i32,

    // #[serde(rename = "m_Shape")]
    // pub shape: i32,

    #[serde(rename = "m_Color")]
    pub color: Color,

    #[serde(rename = "m_Intensity")]
    pub intensity: f32,

    #[serde(rename = "m_Range")]
    pub range: f32,

    // #[serde(rename = "m_SpotAngle")]
    // pub spot_angle: f32,

    // #[serde(rename = "m_InnerSpotAngle")]
    // pub inner_spot_angle: f32,

    // #[serde(rename = "m_CookieSize")]
    // pub cookie_size: f32,

    // #[serde(rename = "m_Shadows")]
    // pub shadows: LightShadows,

    // #[serde(rename = "m_DrawHalo")]
    // pub draw_halo: i32,

    // #[serde(rename = "m_RenderingLayerMask")]
    // pub rendering_layer_mask: u32,

    // #[serde(rename = "m_Lightmapping")]
    // pub lightmapping: i32,

    // #[serde(rename = "m_LightShadowCasterMode")]
    // pub light_shadow_caster_mode: i32,

    // #[serde(rename = "m_BounceIntensity")]
    // pub bounce_intensity: f32,

    // #[serde(rename = "m_ColorTemperature")]
    // pub color_temperature: f32,

    // #[serde(rename = "m_UseColorTemperature")]
    // pub use_color_temperature: i32,

    // #[serde(rename = "m_BoundingSphereOverride")]
    // pub bounding_sphere_override: Vector4,

    // #[serde(rename = "m_UseBoundingSphereOverride")]
    // pub use_bounding_sphere_override: i32,
    //
    // #[serde(rename = "m_UseViewFrustumForShadowCasterCull")]
    // pub use_view_frustum_for_shadow_caster_cull: i32,

    // #[serde(rename = "m_ShadowRadius")]
    // pub shadow_radius: f32,

    // #[serde(rename = "m_ShadowAngle")]
    // pub shadow_angle: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnitySodaPointLight {
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
    #[serde(rename = "lightRenderer")]
    light_renderer: Component,
    #[serde(rename = "lightColor")]
    pub light_color: Color,
    #[serde(rename = "enviromentTint")]
    pub enviroment_tint: u8,
}

impl UnityAsset for UnitySodaPointLight {
    // fn set_file_id(&mut self, file_id: u32) {
    //     self.file_id = file_id;
    // }
    fn name(&self) -> &'static str {
        "UnitySodaPointLight"
    }
}

// 定义一个枚举来表示所有可能的类型
#[derive(Debug, Deserialize)]
#[serde(untagged)] // 尝试匹配第一个成功的变体
pub(crate) enum Asset {
    UnityGameObject(UnityGameObject),
    UnityMeshFilter(UnityMeshFilter),
    UnityMeshCollider(UnityMeshCollider),
    UnityTransform(UnityTransform),
    UnityMeshRenderer(UnityMeshRenderer),
    UnityBoxCollider(UnityBoxCollider),
    UnityLight(UnityLight),
    UnitySodaPointLight(UnitySodaPointLight),
}

pub(crate) fn preprocess_yaml(content: &str) -> String {
    // 匹配 guid: 后面的十六进制值，给它加引号
    let re = regex::Regex::new(r"guid:\s*([0-9a-fA-F]{32})").unwrap();
    re.replace_all(content, r#"guid: "$1""#).to_string()
}

impl UnityScene {
    pub fn new() -> Self {
        Self {
            // game_object: HashMap::new(),
            game_object_raw: HashMap::new(),
            // mesh_colliders: HashMap::new(),
            mesh_colliders_raw:  HashMap::new(),
            // transforms: HashMap::new(),
            transforms_raw:  HashMap::new(),
            // mesh_filters: HashMap::new(),
            mesh_filters_raw:  HashMap::new(),
            // mesh_renderers: HashMap::new(),
            mesh_renderers_raw:  HashMap::new(),
            box_colliders: HashMap::new(),
            index: HashMap::new(),
            // lights: HashMap::new(),
            lights_raw:  HashMap::new(),
            soda_lights: HashMap::new(),
        }
    }
    // 从str返回一个Unity场景对象，多个Object，解析.unity
    pub async fn from_str(&mut self, file_path: PathBuf) -> anyhow::Result<Self> {
        let mut start: bool = false;
        let mut content = String::new();
        let mut old_component_id: u32 = 0;
        let mut old_component_name: Option<String> = None;
        // 场景资源
        let mut unity_scene = UnityScene::new();
        use crate::resource::{ResourceManager};
        // // 根据平台使用不同的加载方式
        info!("Scene 加载: {}", file_path.display());
        let bytes = ResourceManager::load_binary(file_path.to_str().unwrap()).await?;

        let reader = BufReader::new(&bytes[..]);
        
        for line in reader.lines() {
            let line = line?;
            // --- 开始追加捕获
            if line.starts_with("---") {
                start = true;
        
                // 为空表示首次数据
                if !content.trim().is_empty() {
                    // let processed = preprocess_yaml(&content);
                    let name = &old_component_name;
                    match name {
                        Some(name) if name.as_str() == "GameObject" => {
                            unity_scene.game_object_raw.insert(old_component_id, content.clone());
                        }

                        Some(name) if name.as_str() == "MeshCollider" => {
                            unity_scene.mesh_colliders_raw.insert(old_component_id, content.clone());
                        }
                        Some(name) if name.as_str() == "Transform" => {
                            unity_scene.transforms_raw.insert(old_component_id, content.clone());
                        }
                        Some(name) if name.as_str() == "MeshRenderer" => {
                            unity_scene.mesh_renderers_raw.insert(old_component_id, content.clone());
                        }
                        // Some(name) if name.as_str() == "BoxCollider" => {
                        //
                        // }
                        Some(name) if name.as_str() == "Light" => {
                            unity_scene.lights_raw.insert(old_component_id, content.clone());
                        }
                        Some(name) if name.as_str() == "MeshFilter" => {
                            unity_scene.mesh_filters_raw.insert(old_component_id, content.clone());
                        }
                        _ => {}
                    }
                    unity_scene.index.insert(old_component_id, name.clone().unwrap().to_string());
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
                    Ok(o_id) => old_component_id = o_id as u32,
                    Err(_) => continue,
                }
        
                continue;
            }
            if !start {
                continue;
            }
            if !line.starts_with(' ') {
                old_component_name = Some(line.replace(':', ""));
                continue;
            }
            content = content + &line;
            content.push('\n');
        }

        // 清洗数据，包含light等都要清洗
        // fs::write("./save_j_lab_2.json", serde_json::to_string_pretty(&unity_scene).unwrap())?;

        Ok(unity_scene)
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
    pub vertex_count: usize,
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
    #[serde(rename = "m_LocalAABB")]
    pub m_local_aabb: Local_AABB,
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