use std::collections::HashMap;
use std::{env, fs};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use wgpu::{Device, Queue, SurfaceConfiguration};
use crate::entity::{Entity};
use crate::materials::{Material, Texture};
use crate::mesh::Mesh;
use crate::scene::Scene;
use crate::unity::UnityReference;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::*;
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let base = reqwest::Url::parse(&format!(
        "{}/",
        "http://localhost/duckov",
    )).unwrap();
    base.join(file_name).unwrap()
}

// OPFS 缓存辅助函数 (仅用于 wasm32)
#[cfg(target_arch = "wasm32")]
async fn get_from_opfs(file_name: &str) -> Option<Vec<u8>> {
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    use js_sys::{Uint8Array, ArrayBuffer};

    let navigator = web_sys::window()?.navigator();
    let storage = js_sys::Reflect::get(&navigator, &JsValue::from_str("storage")).ok()?;

    // 获取 OPFS 根目录
    let get_directory = js_sys::Reflect::get(&storage, &JsValue::from_str("getDirectory")).ok()?;
    let get_directory_fn = get_directory.dyn_ref::<js_sys::Function>()?;
    let root_promise = get_directory_fn.call0(&storage).ok()?;
    let root_result = JsFuture::from(js_sys::Promise::from(root_promise)).await.ok()?;

    // 获取文件句柄
    let get_file = js_sys::Reflect::get(&root_result, &JsValue::from_str("getFileHandle")).ok()?;
    let get_file_fn = get_file.dyn_ref::<js_sys::Function>()?;
    let file_promise = get_file_fn.call1(&root_result, &JsValue::from_str(file_name)).ok()?;
    let file_handle = JsFuture::from(js_sys::Promise::from(file_promise)).await.ok()?;

    // 获取文件对象
    let get_file_obj = js_sys::Reflect::get(&file_handle, &JsValue::from_str("getFile")).ok()?;
    let get_file_obj_fn = get_file_obj.dyn_ref::<js_sys::Function>()?;
    let file_obj_promise = get_file_obj_fn.call0(&file_handle).ok()?;
    let file_obj = JsFuture::from(js_sys::Promise::from(file_obj_promise)).await.ok()?;

    // 读取文件内容为 ArrayBuffer
    let array_buffer = js_sys::Reflect::get(&file_obj, &JsValue::from_str("arrayBuffer")).ok()?;
    let array_buffer_fn = array_buffer.dyn_ref::<js_sys::Function>()?;
    let buffer_promise = array_buffer_fn.call0(&file_obj).ok()?;
    let buffer = JsFuture::from(js_sys::Promise::from(buffer_promise)).await.ok()?;

    // 转换为 Vec<u8>
    let array = Uint8Array::new(&buffer);
    Some(array.to_vec())
}

#[cfg(target_arch = "wasm32")]
async fn save_to_opfs(file_name: &str, data: &[u8]) -> Option<()> {
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;
    use js_sys::Uint8Array;

    let navigator = web_sys::window()?.navigator();
    let storage = js_sys::Reflect::get(&navigator, &JsValue::from_str("storage")).ok()?;

    // 获取 OPFS 根目录
    let get_directory = js_sys::Reflect::get(&storage, &JsValue::from_str("getDirectory")).ok()?;
    let get_directory_fn = get_directory.dyn_ref::<js_sys::Function>()?;
    let root_promise = get_directory_fn.call0(&storage).ok()?;
    let root_result = JsFuture::from(js_sys::Promise::from(root_promise)).await.ok()?;

    // 创建或获取文件句柄
    let options = js_sys::Object::new();
    js_sys::Reflect::set(&options, &JsValue::from_str("create"), &JsValue::from_bool(true)).ok()?;

    let get_file = js_sys::Reflect::get(&root_result, &JsValue::from_str("getFileHandle")).ok()?;
    let get_file_fn = get_file.dyn_ref::<js_sys::Function>()?;
    let file_promise = get_file_fn.call2(&root_result, &JsValue::from_str(file_name), &options).ok()?;
    let file_handle = JsFuture::from(js_sys::Promise::from(file_promise)).await.ok()?;

    // 创建可写流
    let create_writable = js_sys::Reflect::get(&file_handle, &JsValue::from_str("createWritable")).ok()?;
    let create_writable_fn = create_writable.dyn_ref::<js_sys::Function>()?;
    let writable_promise = create_writable_fn.call0(&file_handle).ok()?;
    let writable = JsFuture::from(js_sys::Promise::from(writable_promise)).await.ok()?;

    // 写入数据
    let uint8_array = Uint8Array::from(data);
    let write = js_sys::Reflect::get(&writable, &JsValue::from_str("write")).ok()?;
    let write_fn = write.dyn_ref::<js_sys::Function>()?;
    let write_promise = write_fn.call1(&writable, &uint8_array).ok()?;
    JsFuture::from(js_sys::Promise::from(write_promise)).await.ok()?;

    // 关闭流
    let close = js_sys::Reflect::get(&writable, &JsValue::from_str("close")).ok()?;
    let close_fn = close.dyn_ref::<js_sys::Function>()?;
    let close_promise = close_fn.call0(&writable).ok()?;
    JsFuture::from(js_sys::Promise::from(close_promise)).await.ok()?;

    Some(())
}


#[cfg(not(target_arch = "wasm32"))]
fn transfer_file(file_path: &str)  -> anyhow::Result<Vec<u8>> {
    dotenv::dotenv().ok();
    let output_path = env::var("target_project").unwrap();
    let transfer: bool = env::var("transfer").unwrap().parse().unwrap_or(false);

    // 定义需要去除的前缀
    let prefix = "/Users/smile/Downloads/duckov/ExportedProject/Assets";

    // 去除前缀，获取相对路径
    let relative_path = file_path.strip_prefix(prefix)
        .unwrap_or(file_path)
        .trim_start_matches('/');

    // 构建目标路径
    let target_path = PathBuf::from(&output_path).join(relative_path);

    // 创建目标目录
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 如果需要转移文件，则复制
    if transfer {
        fs::copy(file_path, &target_path)?;
        println!("Transferred file: {} -> {:?}", file_path, target_path);
    }

    // 读取并返回文件内容
    let data = fs::read(file_path)?;
    Ok(data)
}

pub type MeshId = String;
pub type MaterialId = String;

#[derive(Debug)]
pub struct ResourceManager {
    materials: HashMap<Entity, Arc<Material>>,
    meshes: HashMap<Entity, Arc<Mesh>>,
    mesh_manifest: HashMap<MeshId, Arc<Mesh>>,
    material_manifest: HashMap<MaterialId, Arc<Material>>,
    manifest: HashMap<String, String>,
    texture_manifest: HashMap<String, Arc<Texture>>,
    white_texture: Arc<Texture>,
}

impl ResourceManager {
    pub fn new(device: &Device, queue: &Queue) -> Self {
        let white_texture = Texture::create_dummy_white(device, queue);
        Self{
            materials: Default::default(),
            meshes: Default::default(),
            mesh_manifest: Default::default(),
            material_manifest: Default::default(),
            manifest: HashMap::default(),
            texture_manifest: Default::default(),
            white_texture: Arc::new(white_texture),
        }
    }
    
    pub async fn loading_mapping(&mut self) -> anyhow::Result<()>{
        let guids = Self::load_binary("guid_full.json").await?;
        self.manifest = serde_json::from_str(std::str::from_utf8(&guids)?)?;  
        Ok(())
    }

    // 读取二进制数据
    pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
        #[cfg(target_arch = "wasm32")]
        let data = {
            // 首先尝试从 OPFS 缓存读取
            if let Some(cached_data) = get_from_opfs(file_name).await {
                println!("Loaded from OPFS cache: {}", file_name);
                cached_data
            } else {
                // 如果缓存不存在，从网络获取
                println!("Fetching from network: {}", file_name);
                let url = format_url(file_name);
                let response_data = reqwest::get(url).await?.bytes().await?.to_vec();

                // 异步保存到 OPFS（不等待完成）
                let data_clone = response_data.clone();
                let file_name_clone = file_name.to_string();
                wasm_bindgen_futures::spawn_local(async move {
                    if save_to_opfs(&file_name_clone, &data_clone).await.is_some() {
                        println!("Saved to OPFS cache: {}", file_name_clone);
                    } else {
                        println!("Failed to save to OPFS cache: {}", file_name_clone);
                    }
                });

                response_data
            }
        };
        #[cfg(not(target_arch = "wasm32"))]
        let data = {
            let path = Path::new(env!("OUT_DIR")).join("res").join(file_name);
            fs::read(path.clone()).map_err(|e| {
                println!("Loaded binary filename: {}, origin_path: {:?}, path: {:?}", file_name, &path.to_str(), Path::new(env!("OUT_DIR")).join("res").join(file_name));
                e
            })?
        };

        Ok(data)
    }

     pub fn has_mesh(&self, guid: &str) -> Option<Arc<Mesh>> {
        self.mesh_manifest.get(guid).map(Arc::clone)
    }
    
     pub fn has_material(&self, guid: &str) -> Option<Arc<Material>> {
        self.material_manifest.get(guid).map(Arc::clone)
    }

    // 加载Mesh资源，顶点格式数据之类的
    pub async fn load_mesh(&mut self, m_mesh: &UnityReference, entity: Entity, device: &Device, scene: &Scene, material: &Material, config: &SurfaceConfiguration) -> anyhow::Result<u32> {
        let guid = &m_mesh.guid;
        let file_id = &m_mesh.file_id;
        println!("Loading mesh {:?}", m_mesh);

        let mesh: Arc<Mesh> = if let Some(mesh) = self.has_mesh(guid) {
            mesh
        } else {
            // 常见的 Unity 内置 Mesh fileID：
            // fileIDMesh 类型
            // 10202 Cube（立方体）
            // 10206 Cylinder（圆柱体）
            // 10207 Sphere（球体）
            // 10208 Capsule（胶囊体）
            // 10209 Plane（平面，10×10 单位）
            // 10210 Quad（四边形，1×1 单位）
            // todo 需要处理quad， cube等默认的材质  0000000000000000e000000000000000-> Cube
            let mesh = match (file_id, guid.as_str()) {
                (10202, "0000000000000000e000000000000000") => {
                    Mesh::create_default_cube(guid, device, scene, material, config)
                },
                (10210, "0000000000000000e000000000000000") => {
                    Mesh::create_default_quad(guid, device, scene, material, config)
                },
                _ => {
                    let file_path = self.manifest.get(guid).unwrap();

                    #[cfg(not(target_arch = "wasm32"))]
                    let bytes = transfer_file(&file_path)?;

                    #[cfg(target_arch = "wasm32")]
                    let bytes = ResourceManager::load_binary(file_path).await.map_err(|e| {
                        println!("load_mesh error: {:?}", e);
                        e
                    })?;

                    Mesh::from_unity_data(&bytes, guid, device, scene, material, config).await.map_err(|e| {
                        println!("Failed to load mesh: {:?}", guid);
                        e
                    })?
                }
            };
            
            let mesh_arc = Arc::new(mesh);
            self.mesh_manifest.insert(guid.to_string(), Arc::clone(&mesh_arc));
            mesh_arc
        };
        self.meshes.insert(entity, mesh);

        Ok(entity.id())
    }

    // 加载mat资源材质包，暂时使用实体的Id
    pub async fn load_material(&mut self, entity: Entity, guid: &MaterialId, device: &Device, queue: &Queue) -> anyhow::Result<u32> {
        println!("Loading {:?} material: {:?}", &entity, guid);
        // 处理材默认材质问题
        let material: Arc<Material> = if let Some(mat) = self.has_material(guid) {
            mat
        } else {
            let file_path = self.manifest.get(guid).unwrap();
            #[cfg(not(target_arch = "wasm32"))]
            let mat_bytes = transfer_file(&file_path)?;
            #[cfg(target_arch = "wasm32")]
            let mat_bytes = ResourceManager::load_binary(file_path).await.map_err(|e| {
                println!("Load mat asset error: {:?}, file_name: {:?}", e, guid);
                e
            })?;

            // 后续处理多布局layout的问题, 可能共用mesh, 会有优化部分, 先使用entity_id
            let material = Material::from_unity_bytes(&mat_bytes, guid, device, queue, self).await?;

            let material_arc = Arc::new(material);
            self.material_manifest.insert(guid.to_string(), Arc::clone(&material_arc));
            material_arc
        };
        self.materials.insert(entity, material);
        Ok(entity.id())
    }
    
    // 加载贴图
    pub async fn load_texture(&mut self,  device: &Device, queue: &Queue,guid: &str,) -> anyhow::Result<Arc<Texture>> {
        println!("Loading_texture: {:?}", guid);
        let texture: Arc<Texture> = if let Some(tex) = self.has_texture(guid) {
            tex
        } else {
            let tex = if let Some(texture) = Texture::from_unity_guid(device, queue, guid) {
                texture
            } else {
                let file_path = self.manifest.get(guid).unwrap();

                #[cfg(not(target_arch = "wasm32"))]
                let texture_bytes = transfer_file(&file_path)?;
                #[cfg(target_arch = "wasm32")]
                let texture_bytes = ResourceManager::load_binary(file_path).await.map_err(|e| {
                    println!("Load mat asset error: {:?}, file_name: {:?}", e, guid);
                    e
                })?;

                // 后续处理多布局layout的问题, 可能共用mesh, 会有优化部分, 先使用entity_id
                Texture::from_bytes(device, queue, texture_bytes, &guid)?
            };
            
            let texture_arc = Arc::new(tex);
            self.texture_manifest.insert(guid.to_string(), Arc::clone(&texture_arc));
            texture_arc
        };
        self.texture_manifest.insert(guid.to_string(), Arc::clone(&texture));
        
        Ok(texture)
    }

    pub fn get_material(&self, entity: &Entity) -> Option<&Arc<Material>> {
        self.materials.get(entity)
    }

    pub fn get_mesh(&self, entity: &Entity) -> Option<&Arc<Mesh>> {
        self.meshes.get(entity)
    }
    
    pub fn has_texture(&self, guid: &str) -> Option<Arc<Texture>> {
        self.texture_manifest.get(guid).map(Arc::clone)
    }
    
    pub fn get_guid_file(&self, id: &String) -> Option<&String> {
        self.manifest.get(id)
    }
    pub fn get_white_texture(&self) -> Arc<Texture> {
        Arc::clone(&self.white_texture)
    }
}