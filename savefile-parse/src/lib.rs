mod extractor;
mod format;
mod archived;

use log::{info, debug, error};
use wasm_bindgen::prelude::*;
use crate::archived::Archive;
use crate::format::fix_json_format;

macro_rules! load_optional_field {
    ($source:expr, $target:expr, $field:ident, $getter:ident) => {
        if let Some(value) = $source.$getter() {
            $target.$field = value.into();
        }
    };
}


macro_rules! load_optional_fields {
    ($source:expr, $target:expr, { $($field:ident => $getter:ident),* $(,)? }) => {
        $(
            if let Some(value) = $source.$getter() {
                $target.$field = value.into();
            }
        )*
    };
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn init() {
    // 设置 panic hook
    console_error_panic_hook::set_once();

    // 初始化日志系统
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Debug).unwrap_throw();
        info!("🚀 WASM 模块初始化完成");
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
}

#[wasm_bindgen]
pub fn process_file(data: &[u8]) -> Result<String, JsValue> {
    // info!("开始处理文件, 数据长度: {} 字节", data.len());

    let json_content = std::str::from_utf8(data).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // info!("解析{}", json_content);

    let save_data = crate::extractor::SaveData::from_json(&fix_json_format(&json_content))
        .expect("无法解析存档文件");

    let mut archive = Archive::default();

    if let Some(version) = save_data.get_version() {
        archive.version = format!("游戏版本: {}.{}.{}{}",
                 version.main_version,
                 version.sub_version,
                 version.build_version,
                 version.suffix
        );
    }

    // if let Some(economy_data) = save_data.get_currency() {
    //     archive.economy_data = economy_data;
    // }

   

    // if let Some(exp) = save_data.get_exp() {
    //     archive.exp = exp;
    // }
    // if let Some(keys) = save_data.get_master_keys() {
    //     archive.master_key = keys;
    // }

    
    // if let Some(death) = save_data.get_death_list() {
    //     archive.death_list = death;
    // }

    // if let Some(save_time) = save_data.get_save_time() {
    //     archive.save_time = save_time;
    // }
    // if let Some(inventory) = save_data.get_player_storage() {
    //     archive.player_storage = inventory;
    // }
    // if let Some(inventory) = save_data.get_inventory_safe() {
    //     archive.inventory_safe = inventory;
    // }
    // if let Some(health) = save_data.get_health() {
    //     archive.health = health;
    // }
    // if let Some(items) = save_data.get_character_items() {
    //     archive.character_item_data = items;
    // }

    load_optional_fields!(save_data, archive, {
        economy_data => get_currency,
        exp => get_exp,
        master_key => get_master_keys,
        death_list => get_death_list,
        save_time => get_save_time,
        player_storage => get_player_storage,
        inventory_safe => get_inventory_safe,
        health => get_health,
        character_item_data => get_character_items,
        game_lock => get_game_lock,
        quests => get_quests,
    });
    
    // if let Some(game_lock) = save_data.get_game_lock() {
    //     archive.game_lock = game_lock;
    // }
    archive.extract_kill_counts(&save_data.data);
    
    // if let Some(quests) = save_data.get_quests() {
    //     archive.quests = quests;
    // }

    let archive_json = serde_json::to_string(&archive).unwrap();

    Ok(archive_json)
}