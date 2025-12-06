use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct MapInfo {
    pub id: u32,
    pub name: String,
    pub cn: String,
    pub path: String,
    pub disabled_ids: Vec<u32>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_maps() -> JsValue {
    let maps: Vec<MapInfo> = vec![
        // MapInfo {
        //     id: 1000,
        //     name: "Level_GroundZero_1".to_string(),
        //     cn: "零号区".to_string(),
        //     path: "Scenes/Level_GroundZero/Level_GroundZero_1.unity".to_string(),
        //     disabled_ids: vec![],
        // },
        MapInfo {
            id: 1001,
            name: "Level_GroundZero_1".to_string(),
            cn: "零号区".to_string(),
            path: "Scenes/Level_GroundZero/Level_GroundZero_1.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1002,
            name: "Level_GroundZero_Cave".to_string(),
            cn: "零号区洞穴".to_string(),
            path: "Scenes/Level_GroundZero/Level_GroundZero_Cave.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1003,
            name: "Level_JLab_1".to_string(),
            cn: "实验室地下一层".to_string(),
            path: "Scenes/Level_JLab/Level_JLab_1.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1004,
            name: "Level_JLab_2".to_string(),
            cn: "实验室地下二层".to_string(),
            path: "Scenes/Level_JLab/Level_JLab_2.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1005,
            name: "Level_HiddenWarehouse".to_string(),
            cn: "仓库区".to_string(),
            path: "Scenes/Level_HiddenWarehouse/Level_HiddenWarehouse.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1006,
            name: "Level_Farm_01".to_string(),
            cn: "农场镇".to_string(),
            path: "Scenes/Level_OpenWorldTest/Level_Farm_01.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1007,
            name: "Level_StormZone_1".to_string(),
            cn: "风暴区".to_string(),
            path: "Scenes/Level_StormZone/Level_StormZone_1.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1008,
            name: "Level_StormZone_B0".to_string(),
            cn: "风暴区B0".to_string(),
            path: "Scenes/Level_StormZone/Level_StormZone_B0.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1009,
            name: "Level_StormZone_B1".to_string(),
            cn: "风暴区B1".to_string(),
            path: "Scenes/Level_StormZone/Level_StormZone_B1.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1010,
            name: "Level_StormZone_B2".to_string(),
            cn: "风暴区B2".to_string(),
            path: "Scenes/Level_StormZone/Level_StormZone_B2.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1011,
            name: "Level_StormZone_B3".to_string(),
            cn: "风暴区B3".to_string(),
            path: "Scenes/Level_StormZone/Level_StormZone_B3.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1012,
            name: "Level_StormZone_B4".to_string(),
            cn: "风暴区B4".to_string(),
            path: "Scenes/Level_StormZone/Level_StormZone_B4.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1013,
            name: "Base_SceneV2".to_string(),
            cn: "主场景".to_string(),
            path: "Scenes/Base_SceneV2.unity".to_string(),
            disabled_ids: vec![],
        },
        MapInfo {
            id: 1014,
            name: "Base_SceneV2_Sub_01".to_string(),
            cn: "主场景地下通道".to_string(),
            path: "Scenes/Base_SceneV2_Sub_01.unity".to_string(),
            disabled_ids: vec![],
        },
    ];
    serde_wasm_bindgen::to_value(&maps).unwrap()
}
