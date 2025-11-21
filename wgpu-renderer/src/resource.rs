use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use cgmath::num_traits::ops::bytes;
use wgpu::{Device, Queue, SurfaceConfiguration};
use crate::entity::{Entity, Mesh};
use crate::materials::{Material, Texture};
use crate::scene::Scene;

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let base = reqwest::Url::parse(&format!(
        "{}/{}/",
        "http://localhost",
        option_env!("RES_PATH").unwrap_or("res"),
    ))
        .unwrap();
    base.join(file_name).unwrap()
}

// 读取二进制数据
pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    let data = {
        let url = format_url(file_name);
        reqwest::get(url).await?.bytes().await?.to_vec()
    };
    #[cfg(not(target_arch = "wasm32"))]
    let data = {
        let path = Path::new(env!("OUT_DIR")).join("res").join(file_name);
        fs::read(path).map_err(|e| {
            println!("Loaded binary: {:?}", Path::new(env!("OUT_DIR")).join("res").join(file_name)); 
            e
        })?
    };

    Ok(data)
}


#[derive(Debug)]
pub struct ResourceManager {
    materials: HashMap<Entity, Arc<Material>>,
    meshes: HashMap<Entity, Arc<Mesh>>,
    mesh_manifest: HashMap<String, Arc<Mesh>>,
    material_manifest: HashMap<String, Arc<Material>>,
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
        let guids = load_binary("guid.json").await?;
        self.manifest = serde_json::from_str(std::str::from_utf8(&guids)?)?;  
        Ok(())
    }

    async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
        #[cfg(target_arch = "wasm32")]
        let data = {
            let url = format_url(file_name);
            reqwest::get(url).await?.bytes().await?.to_vec()
        };
        #[cfg(not(target_arch = "wasm32"))]
        let data = {
            let path = Path::new(env!("OUT_DIR")).join("res").join(file_name);
            fs::read(path)?
        };

        Ok(data)
    }

     fn has_mesh(&self, guid: &str) -> Option<Arc<Mesh>> {
        self.mesh_manifest.get(guid).map(Arc::clone)
    }
    
     fn has_material(&self, guid: &str) -> Option<Arc<Material>> {
        self.material_manifest.get(guid).map(Arc::clone)
    }

    // 加载Mesh资源，顶点格式数据之类的
    pub async fn load_mesh(&mut self, guid: &str, entity: Entity, device: &Device, scene: &Scene, material: &Material, config: &SurfaceConfiguration) -> anyhow::Result<u32> {
        let file_path = self.manifest.get(guid).unwrap();

        let mesh: Arc<Mesh> = if let Some(mesh) = self.has_mesh(guid) {
            mesh
        } else {
            let bytes = fs::read(&file_path)?;

            // let bytes = load_binary(file_name).await.map_err(|e| {
            //     println!("load_mesh error: {:?}", e);
            //     e
            // })?;
            
            println!("Start Loaded mesh: {:?}", guid);
            let mesh = Mesh::from_unity_data(&bytes, device, scene, material, config).await.map_err(|e| {
                println!("Failed to load mesh: {:?}", guid);
                e
            })?;
            let mesh_arc = Arc::new(mesh);
            self.mesh_manifest.insert(guid.to_string(), Arc::clone(&mesh_arc));
            mesh_arc
        };
        self.meshes.insert(entity, mesh);

        Ok(entity.id())
    }

    // 加载mat资源材质包，暂时使用实体的Id
    pub async fn load_material(&mut self, entity: Entity, guid: &str, device: &Device, queue: &Queue) -> anyhow::Result<u32> {
        // println!("Loading material: {:?}", guid);
        let file_path = self.manifest.get(guid).unwrap();
        let material: Arc<Material> = if let Some(mat) = self.has_material(guid) {
            mat
        } else {
            let mat_bytes = fs::read(&file_path)?;
            // let mat_bytes = load_binary(file_path).await.map_err(|e| {
            //     println!("Load mat asset error: {:?}, file_name: {:?}", e, guid);
            //     e
            // })?;

            // 后续处理多布局layout的问题, 可能共用mesh, 会有优化部分, 先使用entity_id
            let material = Material::from_unity_bytes(&mat_bytes, device, queue, self).await?;

            let material_arc = Arc::new(material);
            self.material_manifest.insert(guid.to_string(), Arc::clone(&material_arc));
            material_arc
        };
        self.materials.insert(entity, material);
        Ok(entity.id())
    }
    
    // 加载贴图
    pub async fn load_texture(&mut self,  device: &Device, queue: &Queue,guid: &str,) -> anyhow::Result<Arc<Texture>> {
        // println!("Loading material: {:?}", guid);
        let file_path = self.manifest.get(guid).unwrap();
        let texture: Arc<Texture> = if let Some(tex) = self.has_texture(guid) {
            tex
        } else {
            let texture_bytes = fs::read(&file_path)?;
            // let mat_bytes = load_binary(file_path).await.map_err(|e| {
            //     println!("Load mat asset error: {:?}, file_name: {:?}", e, guid);
            //     e
            // })?;

            // 后续处理多布局layout的问题, 可能共用mesh, 会有优化部分, 先使用entity_id
            let tex = Texture::from_bytes(device, queue, texture_bytes, &guid)?;
            
            let texture_arc = Arc::new(tex);
            self.texture_manifest.insert(guid.to_string(), Arc::clone(&texture_arc));
            texture_arc
        };
        self.texture_manifest.insert(guid.to_string(), Arc::clone(&texture));
        
        Ok(texture)
    }

    pub fn get_material(&self, entity: Entity) -> Option<&Arc<Material>> {
        self.materials.get(&entity)
    }

    pub fn get_mesh(&self, entity: Entity) -> Option<&Arc<Mesh>> {
        self.meshes.get(&entity)
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