use std::path::PathBuf;
use wgpu_renderer::run;
use crate::unity::UnityScene;

mod unity;

fn main() -> anyhow::Result<()> {
    run()?;
    // let mut uns = UnityScene::new();
    // let path = PathBuf::from("/Users/smile/Downloads/unity/My project/Assets/Scenes/Level_JLab/Level_JLab_2.unity");
    // uns.from_str(path).expect("TODO: panic message");
    Ok(())
}