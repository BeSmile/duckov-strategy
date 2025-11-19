use std::env;

// 十六进数据解析过程， 1个十六进制2^4代表4位，1个字节等于8位，1个f32需要4个16进制数据
pub fn parse_f32_le(hex: &str) -> f32 {
    // 每次处理4个字节
    let mut bytes = [0u8; 4];
    for i in 0..4 {
        // 每次间隔2
        let byte_hex = &hex[i*2..i*2+2];
        bytes[i] = u8::from_str_radix(byte_hex, 16).unwrap_or(0);
    }
    f32::from_le_bytes(bytes)
}


pub fn parse_unity_index_buffer(hex_string: &str) -> Vec<u32> {
    // 移除可能的空格和换行
    let hex_string = hex_string.replace(|c: char| c.is_whitespace(), "");

    // 每8个字符代表一个u32 (小端序)
    hex_string
        .as_bytes()
        .chunks_exact(8)
        .map(|chunk| {
            let hex_str = std::str::from_utf8(chunk).unwrap();
            // Unity 使用小端序存储
            u32::from_str_radix(hex_str, 16)
                .map(|v| u32::from_le(v))
                .unwrap()
        })
        .collect()
}

pub fn get_background_color() -> wgpu::Color {
    #[cfg(target="wasm32")]
    {
        wgpu::Color{
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
    
    #[cfg(not(target="wasm32"))]
    {
        dotenv::dotenv().ok();
        wgpu::Color{
            r: env::var("BG_R".to_string()).unwrap_or("1.0".to_string()).parse().expect("REASON"),
            g: env::var("BG_G".to_string()).unwrap_or("1.0".to_string()).parse().expect("REASON"),
            b: env::var("BG_B".to_string()).unwrap_or("1.0".to_string()).parse().expect("REASON"),
            a: env::var("BG_A".to_string()).unwrap_or("1.0".to_string()).parse().expect("REASON"),
        }
    }
}

pub fn get_block_mesh() -> bool{
    #[cfg(not(target="wasm32"))]
    {
        dotenv::dotenv().ok();
        env::var("BLOCK_MESH".to_string()).unwrap_or("false".to_string()).parse().expect("REASON")
    }
    #[cfg(target="wasm32")]
    false
}