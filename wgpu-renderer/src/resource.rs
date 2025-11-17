use std::fs;
use std::path::Path;

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
        fs::read(path)?
    };

    Ok(data)
}