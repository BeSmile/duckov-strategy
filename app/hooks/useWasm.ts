import { useState, useEffect, useCallback } from 'react';
import WasmManager, { MapInfo } from '../utils/wasm-manager';


export const useWasm = () => {
    const [isReady, setIsReady] = useState(false);
    const [error, setError] = useState<Error | null>(null);
    const [wasmManager] = useState(() => WasmManager.getInstance());

    useEffect(() => {
        wasmManager
            .init()
            .then(() => setIsReady(true))
            .catch((err) => setError(err));
    }, [wasmManager]);

    const getMaps = useCallback(() => {
        return wasmManager.getMaps() as MapInfo[];
    }, [wasmManager]);

    const setScenePath = useCallback((path: string) => {
        return wasmManager.setScenePath(path);
    }, [wasmManager]);

    const changeScene = useCallback((path: string) => {
        return wasmManager.changeScene(path);
    }, [wasmManager]);

    const runWeb = useCallback(async () => {
        return wasmManager.runWeb();
    }, [wasmManager]);

    const getLoadingProgress = useCallback(async () => {
        return wasmManager.getLoadingProgress();
    }, [wasmManager]);

    const getLoadingState = useCallback(async () => {
        return wasmManager.getLoadingState();
    }, [wasmManager]);

    const getLoadingMessage = useCallback(async () => {
        return wasmManager.getLoadingMessage();
    }, [wasmManager]);

    return {
        isReady,
        error,
        getMaps,
        setScenePath,
        changeScene,
        runWeb,
        getLoadingProgress,
        getLoadingState,
        getLoadingMessage,
    } as const;
};
