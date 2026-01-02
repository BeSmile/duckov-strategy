'use client'

import {
    SyncInitInput,
    get_maps,
    run_web,
    default as init,
} from '@/public/wasm';

export
enum SceneLoadingState {
    // ===== 基础状态 =====
    Idle = 0,               // 空闲，无任何操作

    // ===== 初始化阶段 =====
    Initializing = 1,       // 正在初始化 WGPU
    InitFailed = 2,         // 初始化失败（GPU 不支持等）

    // ===== 加载阶段 =====
    LoadingScene = 3,       // 正在加载场景文件
    LoadingAssets = 4,      // 正在加载资源
    LoadingProgress ,

    // ===== 设置阶段 =====
    Setting = 6,            // 正在设置场景
    Building = 7,           // 正在构建渲染管线

    // ===== 就绪状态 =====
    Ready = 8,              // 准备完成
    Running = 9,            // 正在运行/渲染中
    Paused = 10,             // 暂停渲染

    // ===== 场景切换 =====
    Unloading = 11,         // 正在卸载当前场景
    Switching = 12,         // 正在切换场景
    HotReloading = 13,      // 热重载中

    // ===== 资源管理 =====
    DisposingAssets = 14,   // 正在释放资源
    DisposingScene = 15,    // 正在清理场景对象
    DisposingAll = 16,      // 正在清理所有资源

    // ===== 错误状态 =====
    Error = 17,             // 通用错误
    AssetLoadError = 18,    // 资源加载错误
    SceneParseError = 19,   // 场景解析错误
    RenderError = 20,       // 渲染错误

    // ===== 恢复状态 =====
    Recovering = 21,        // 从错误中恢复
    Restarting = 22,        // 重启渲染器
}


export interface MapInfo {
    id: number;
    name: string;
    cn: string;
    path: string;
    disabled_ids: number[];
}

type WasmModule = {
    get_maps: () => MapInfo[];
    run_web: typeof run_web;
    default: typeof init;
    get_loading_progress: () => number;
    get_loading_message: () => string;
    get_loading_state: () =>  SceneLoadingState;

    Commander: {
        change_scene: (path: string) => void;
        set_scene_path: (path: string) => void;
    };
};

export default class WasmManager {
    private static instance: WasmManager;
    private wasmModule: SyncInitInput & WasmModule | null = null;
    private initialized: boolean = false;
    // private commander: Commander | null = null

    private constructor() {}

    static getInstance(): WasmManager {
        if (!WasmManager.instance) {
            WasmManager.instance = new WasmManager();
        }
        return WasmManager.instance;
    }

    async init() {
        if (this.initialized) {
            return this.wasmModule;
        }

        try {
            const wasm = await import('@/public/wasm/');
            await wasm.default();
            console.log(wasm, 'wasm');
            this.wasmModule = wasm as unknown as SyncInitInput & WasmModule;
            // this.commander = new this.wasmModule.Commander();
            this.initialized = true;
            console.log('WASM module initialized successfully');
            return wasm;
        } catch (error) {
            console.error('WASM initialization failed:', error);
            throw error;
        }
    }

    getMaps() {
        if (!this.initialized || !this.wasmModule) {
            throw new Error('WASM module not initialized');
        }
        return this.wasmModule.get_maps();
    }

    getLoadingProgress() {
        if (!this.initialized || !this.wasmModule) {
            throw new Error('WASM module not initialized');
        }
        return this.wasmModule.get_loading_progress();
    }

    getLoadingState() {
        if (!this.initialized || !this.wasmModule) {
            throw new Error('WASM module not initialized');
        }
        return this.wasmModule.get_loading_state();
    }

    getLoadingMessage() {
        if (!this.initialized || !this.wasmModule) {
            throw new Error('WASM module not initialized');
        }
        return this.wasmModule.get_loading_message();
    }

    /**
     * 设置场景路径（必须在 runWeb() 之前调用）
     * @param path 场景路径，例如 "Scenes/Level_JLab/Level_JLab_2.unity"
     */
    setScenePath(path: string) {
        if (!this.initialized || !this.wasmModule) {
            throw new Error('WASM module not initialized');
        }
        console.log('Setting scene path:', path);
        this.wasmModule.Commander.set_scene_path(path);
    }

    /**
     * 动态切换场景（运行时调用，无需重新加载页面）
     * @param path 场景路径，例如 "Scenes/Level_JLab/Level_JLab_2.unity"
     */
    changeScene(path: string) {
        if (!this.initialized || !this.wasmModule) {
            throw new Error('WASM module not initialized');
        }
        console.log('Changing scene to:', path);
        return this.wasmModule.Commander.change_scene(path);
    }

    runWeb() {
        if (!this.initialized || !this.wasmModule) {
            throw new Error('WASM module not initialized');
        }
        return this.wasmModule.run_web();
    }
}