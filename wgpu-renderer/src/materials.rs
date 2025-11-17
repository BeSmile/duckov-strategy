use wgpu::{Device, Queue};

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture{
    pub fn from_bytes() {
        
    }
    
    // pub fn from_image(device: &Device, queue: &Queue, image: &image::DynamicImage, label: &str) -> Result<Self>{
    //     
    // }
}

pub struct Material{
    pub name: String,
    pub diffuse_texture: Texture,// texture散射计算
    pub bind_group: wgpu::BindGroup,// 定义的bing_group数据
}