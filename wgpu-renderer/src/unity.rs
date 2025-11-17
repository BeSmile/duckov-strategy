use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use serde::{Deserialize, Deserializer, Serialize};
use serde_yaml::Value;

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
pub struct UnityGameObject{
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
pub struct GameObject{
    data: UnityGameObject,
    mesh_colliders: Vec<UnityMeshCollider>,
    transforms: Vec<UnityTransform>,
    mesh_filters: Vec<UnityMeshFilter>,
    mesh_renderers: Vec<UnityMeshRenderer>,
    box_colliders: Vec<UnityBoxCollider>,
}

impl GameObject{
    pub fn new(resource: UnityGameObject) ->Self {
        Self{
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
pub struct UnityTransform{
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
pub struct UnityMeshRenderer{
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
    #[serde(rename = "m_Materials")]
    m_children: Vec<UnityReference>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UnityMeshFilter{
    #[serde(rename = "m_GameObject")]
    m_game_object: Component,
    #[serde(rename = "m_Mesh")]
    m_mesh: UnityReference,
}

impl UnityMeshFilter{
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
pub struct UnityMeshCollider{
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
pub struct UnityBoxCollider{
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
pub struct UnityScene{
    game_objects: Vec<GameObject>,
}

impl UnityScene{
    pub fn new() -> UnityScene{
        Self{
            game_objects: vec![],
        }
    }

    // 从str返回一个Unity场景对象，多个Object，解析.unity
    pub fn from_str(&mut self, file_path: PathBuf) -> anyhow::Result<()> {
        let mut start: bool = false;
        let mut content = String::new();
        let mut old_component_id:i32 = 0;

        let file = File::open(file_path)?;
        let mut mapping_objects:HashMap<i32, GameObject> = HashMap::new();

        let buffer = BufReader::new(file);

        for line_result in buffer.lines(){
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
                            let game_object = mapping_objects.get_mut(&mesh_collider.m_game_object.file_id).unwrap();
                            game_object.mesh_colliders.push(mesh_collider);
                        }
                        Ok(Asset::UnityTransform(unity_transform)) => {
                            let game_object = mapping_objects.get_mut(&unity_transform.m_game_object.file_id).unwrap();
                            game_object.transforms.push(unity_transform);
                        }
                        Ok(Asset::UnityMeshFilter(unity_mesh_filter)) => {
                            let game_object = mapping_objects.get_mut(&unity_mesh_filter.m_game_object.file_id).unwrap();
                            game_object.mesh_filters.push(unity_mesh_filter);
                        }
                        Ok(Asset::UnityMeshRenderer(unity_mesh_renderer)) => {
                            let game_object = mapping_objects.get_mut(&unity_mesh_renderer.m_game_object.file_id).unwrap();
                            game_object.mesh_renderers.push(unity_mesh_renderer);
                        }
                        Ok(Asset::UnityBoxCollider(unity_box_collider)) => {
                            let game_object = mapping_objects.get_mut(&unity_box_collider.m_game_object.file_id).unwrap();
                            game_object.box_colliders.push(unity_box_collider);
                        }
                        _ => {}
                    }
                    content.clear();// 清空内容
                }
                let tags = line.split(' ');
                // 当前组件的id，一般都是从Object 开始遍历，不用担心顺序的问题
                match tags.collect::<Vec<_>>().get(2).unwrap().replace("&", "").to_string().parse::<i32>() {
                    Ok(o_id) => {
                        old_component_id =o_id
                    }
                    Err(_) => {
                        continue
                    }
                }

                continue
            }
            if !start {
                continue;
            }
            if !line.starts_with(' ') {
                continue
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