use std::sync::Mutex;
use log::info;
use once_cell::sync::Lazy;
use wasm_bindgen::prelude::wasm_bindgen;

// 场景加载状态枚举
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SceneLoadingState {
    // ===== 基础状态 =====
    Idle,               // 空闲，无任何操作

    // ===== 初始化阶段 =====
    Initializing,       // 正在初始化 WGPU
    InitFailed,         // 初始化失败（GPU 不支持等）

    // ===== 加载阶段 =====
    LoadingScene,       // 正在加载场景文件（解析 YAML/JSON）
    LoadingAssets,      // 正在加载资源（模型、贴图、材质等）
    LoadingProgress,    // 加载中（可配合进度百分比使用）

    // ===== 设置阶段 =====
    Setting,            // 正在设置场景（创建渲染对象、绑定资源）
    Building,           // 正在构建渲染管线/场景图

    // ===== 就绪状态 =====
    Ready,              // 准备完成，可以渲染
    Running,            // 正在运行/渲染中
    Paused,             // 暂停渲染

    // ===== 场景切换 =====
    Unloading,          // 正在卸载当前场景
    Switching,          // 正在切换场景（卸载旧场景 + 加载新场景）
    HotReloading,       // 热重载中（保留部分状态的重新加载）

    // ===== 资源管理 =====
    DisposingAssets,    // 正在释放资源（贴图、Buffer 等）
    DisposingScene,     // 正在清理场景对象
    DisposingAll,       // 正在清理所有资源

    // ===== 错误状态 =====
    Error,              // 通用错误
    AssetLoadError,     // 资源加载错误
    SceneParseError,    // 场景解析错误
    RenderError,        // 渲染错误

    // ===== 恢复状态 =====
    Recovering,         // 从错误中恢复
    Restarting,         // 重启渲染器
}

impl SceneLoadingState {
    /// 是否正在处理中（不应被打断）
    pub fn is_busy(&self) -> bool {
        matches!(
            self,
            SceneLoadingState::Initializing
                | SceneLoadingState::LoadingScene
                | SceneLoadingState::LoadingAssets
                | SceneLoadingState::Setting
                | SceneLoadingState::Building
                | SceneLoadingState::Switching
                | SceneLoadingState::HotReloading
                | SceneLoadingState::Unloading
                | SceneLoadingState::DisposingAssets
                | SceneLoadingState::DisposingScene
                | SceneLoadingState::DisposingAll
                | SceneLoadingState::Recovering
                | SceneLoadingState::Restarting
        )
    }

    /// 是否可以开始新的加载操作
    pub fn can_load(&self) -> bool {
        matches!(
            self,
            SceneLoadingState::Idle
                | SceneLoadingState::Ready
                | SceneLoadingState::Running
                | SceneLoadingState::Paused
                | SceneLoadingState::Error
                | SceneLoadingState::InitFailed
                | SceneLoadingState::AssetLoadError
                | SceneLoadingState::SceneParseError
                | SceneLoadingState::RenderError
        )
    }

    /// 是否处于错误状态
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            SceneLoadingState::Error
                | SceneLoadingState::InitFailed
                | SceneLoadingState::AssetLoadError
                | SceneLoadingState::SceneParseError
                | SceneLoadingState::RenderError
        )
    }
}

impl Default for SceneLoadingState {
    fn default() -> Self {
        SceneLoadingState::Idle
    }
}

// 加载进度信息
#[derive(Clone, Default)]
pub struct LoadingProgress {
    pub state: SceneLoadingState,
    pub progress: f32,          // 0.0 - 1.0
    pub message: String,        // 当前状态描述
    pub error: Option<String>,  // 错误信息
}


// 全局加载状态
pub static LOADING_PROGRESS: Lazy<Mutex<LoadingProgress>> =
    Lazy::new(|| Mutex::new(LoadingProgress::default()));

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_loading_state() -> SceneLoadingState {
    LOADING_PROGRESS
        .lock()
        .map(|p| p.state)
        .unwrap_or(SceneLoadingState::Error)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_loading_progress() -> f32 {
    info!("Getting progress");
    LOADING_PROGRESS
        .lock()
        .map(|p| p.progress)
        .unwrap_or(0.0)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_loading_message() -> String {

    LOADING_PROGRESS
        .lock()
        .map(|p| p.message.clone())
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_loading_error() -> Option<String> {
    LOADING_PROGRESS
        .lock()
        .ok()
        .and_then(|p| p.error.clone())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn is_scene_ready() -> bool {
    get_loading_state() == SceneLoadingState::Ready
}


// ============ 内部更新函数 ============

pub(crate) fn set_loading_state(state: SceneLoadingState, progress: f32, message: &str) {
    if let Ok(mut p) = LOADING_PROGRESS.lock() {
        p.state = state;
        p.progress = progress;
        p.message = message.to_string();
        p.error = None;

        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("[{}%] {}", (progress * 100.0) as i32, message).into());
    }
}

pub(crate) fn set_loading_error(error: &str) {
    if let Ok(mut p) = LOADING_PROGRESS.lock() {
        p.state = SceneLoadingState::Error;
        p.error = Some(error.to_string());
        p.message = "Loading failed".to_string();

        #[cfg(target_arch = "wasm32")]
        web_sys::console::error_1(&format!("Loading error: {}", error).into());
    }
}


// ==================== 命令队列系统 ====================

/// 场景命令枚举
#[derive(Clone, Debug)]
pub enum SceneCommand {
    ChangeScene { path: String },
    SetCameraPosition { x: f32, y: f32, z: f32 },
    SetCameraTarget { x: f32, y: f32, z: f32 },
}

/// 命令队列（线程安全）
pub static COMMAND_QUEUE: Lazy<Mutex<Vec<SceneCommand>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

/// 查询结果（线程安全）
pub static QUERY_RESULTS: Lazy<Mutex<QueryResults>> =
    Lazy::new(|| Mutex::new(QueryResults::default()));

#[derive(Default, Clone, Debug)]
pub struct QueryResults {
    pub camera_position: [f32; 3],
    pub camera_target: [f32; 3],
    pub entity_count: usize,
    pub visible_count: usize,
    pub current_scene: String,
}