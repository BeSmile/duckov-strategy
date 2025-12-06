'use client'

import { SyncInitInput, get_maps, run_web, default as init } from '@/wgpu-renderer/pkg';


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
}

export default class WasmManager {
    private static instance: WasmManager;
    private wasmModule: SyncInitInput & WasmModule | null = null;
    private initialized: boolean = false;

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
            const wasm = await import('@/wgpu-renderer/pkg/');
            await wasm.default();
            this.wasmModule = wasm;
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

    runWeb() {
        if (!this.initialized || !this.wasmModule) {
            throw new Error('WASM module not initialized');
        }
        return this.wasmModule.run_web();
    }
}