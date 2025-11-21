use serde::{Deserialize, Deserializer, Serialize};
use serde_yaml::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use cgmath::Vector3;
use wgpu::VertexAttribute;

#[derive(Debug, Serialize, Deserialize)]
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
    m_name: String,
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
    pub game_object: HashMap<u32, UnityGameObject>,
    pub mesh_colliders: HashMap<u32, UnityMeshCollider>,
    pub transforms: HashMap<u32, UnityTransform>,
    pub mesh_filters: HashMap<u32, UnityMeshFilter>,
    pub mesh_renderers: HashMap<u32, UnityMeshRenderer>,
    pub lights: HashMap<u32, UnityLight>,
    pub soda_lights: HashMap<u32, UnitySodaPointLight>,
    pub box_colliders: HashMap<u32, UnityBoxCollider>,// 不太需要
    pub index: HashMap<u32, String>,
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
    #[serde(skip_deserializing)]
    file_id: u32,
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
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
    #[serde(skip_deserializing)]
    file_id: u32,
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
pub struct UnityMeshCollider {
    #[serde(skip_deserializing)]
    file_id: u32,
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
enum Asset {
    UnityGameObject(UnityGameObject),
    UnityMeshCollider(UnityMeshCollider),
    UnityTransform(UnityTransform),
    UnityMeshFilter(UnityMeshFilter),
    UnityMeshRenderer(UnityMeshRenderer),
    UnityBoxCollider(UnityBoxCollider),
    UnityLight(UnityLight),
    UnitySodaPointLight(UnitySodaPointLight),
    // 您可以添加其他可能的类型
}

impl UnityScene {
    pub fn new() -> Self {
        Self {
            game_object: HashMap::new(),
            mesh_colliders: HashMap::new(),
            transforms: HashMap::new(),
            mesh_filters: HashMap::new(),
            mesh_renderers: HashMap::new(),
            box_colliders: HashMap::new(),
            index: HashMap::new(),
            lights: HashMap::new(),
            soda_lights: HashMap::new(),
        }
    }
    // 从str返回一个Unity场景对象，多个Object，解析.unity
    pub fn from_str(&mut self, file_path: PathBuf) -> anyhow::Result<Self> {
        let mut start: bool = false;
        let mut content = String::new();
        let mut old_component_id: u32 = 0;
        // 场景资源
        let mut unity_scene = UnityScene::new();

        let file = File::open(file_path)?;
        let mut deleted_ids:HashSet<u32> = HashSet::new();

        // let mut mapping_objects: HashMap<i32, GameObject> = HashMap::new();

        let buffer = BufReader::new(file);

        for line_result in buffer.lines() {
            let line = line_result?;
            // --- 开始追加捕获
            if line.starts_with("---") {
                start = true;

                // 为空表示首次数据
                if !content.trim().is_empty() {
                    match serde_yaml::from_str::<Asset>(&content) {
                        // m_IsActive需要处理连续隐藏的问题
                        Ok(Asset::UnityGameObject(mut unity_game_object)) => {
                            // if unity_game_object.m_is_active == 1 {
                            //     // unity_game_object.set_file_id(unity_game_object.file_id);
                            //     unity_scene.game_object.insert(old_component_id, unity_game_object);
                            // } else {
                            //     for conp in unity_game_object.m_component {
                            //         deleted_ids.insert(conp.component.file_id);
                            //     }
                            //     deleted_ids.insert(old_component_id);
                            // }
                            unity_scene.game_object.insert(old_component_id, unity_game_object);
                        }
                        Ok(Asset::UnityMeshCollider(mut mesh_collider)) => {
                            // 在已删除内就跳过
                            if deleted_ids.contains(&mesh_collider.m_game_object.file_id) {
                                continue
                            }
                            // mesh_collider.set_file_id(old_component_id);
                            unity_scene.index.insert(old_component_id, mesh_collider.name().to_string());
                            unity_scene.mesh_colliders.insert(old_component_id, mesh_collider);
                        }
                        Ok(Asset::UnityTransform(mut unity_transform)) => {
                            if deleted_ids.contains(&unity_transform.m_game_object.file_id) {
                                continue
                            }
                            // unity_transform.set_file_id(old_component_id);
                            unity_scene.index.insert(old_component_id, unity_transform.name().to_string());
                            unity_scene.transforms.insert(old_component_id, unity_transform);
                        }
                        Ok(Asset::UnityMeshFilter(mut unity_mesh_filter)) => {
                            if deleted_ids.contains(&old_component_id) {
                                continue
                            }
                            // unity_mesh_filter.set_file_id(old_component_id);
                            unity_scene.index.insert(old_component_id, unity_mesh_filter.name().to_string());

                            unity_scene.mesh_filters.insert(old_component_id, unity_mesh_filter);

                        }
                        Ok(Asset::UnityMeshRenderer(mut unity_mesh_renderer)) => {
                            if deleted_ids.contains(&old_component_id) {
                                continue
                            }

                            // unity_mesh_renderer.set_file_id(old_component_id);
                            unity_scene.index.insert(old_component_id, unity_mesh_renderer.name().to_string());

                            unity_scene.mesh_renderers.insert(old_component_id, unity_mesh_renderer);
                        }
                        Ok(Asset::UnityBoxCollider(mut unity_box_collider)) => {
                            if deleted_ids.contains(&old_component_id) {
                                continue
                            }

                            // unity_box_collider.set_file_id(old_component_id);
                            unity_scene.index.insert(old_component_id, unity_box_collider.name().to_string());

                            unity_scene.box_colliders.insert(old_component_id, unity_box_collider);
                        }
                        Ok(Asset::UnityLight(mut light)) => {
                            if deleted_ids.contains(&old_component_id) {
                                continue
                            }

                            // unity_box_collider.set_file_id(old_component_id);
                            unity_scene.index.insert(old_component_id, light.name().to_string());

                            unity_scene.lights.insert(old_component_id, light);
                        }
                        Ok(Asset::UnitySodaPointLight(mut light)) => {
                            if deleted_ids.contains(&old_component_id) {
                                continue
                            }
                        
                            // unity_box_collider.set_file_id(old_component_id);
                            unity_scene.index.insert(old_component_id, light.name().to_string());
                        
                            unity_scene.soda_lights.insert(old_component_id, light);
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
                    Ok(o_id) => old_component_id = o_id as u32,
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

        // 清洗数据，包含light等都要清洗

        // fs::write("./save_j_lab_1.json", serde_json::to_string_pretty(&unity_scene).unwrap())?;

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