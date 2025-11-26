import { useState, useEffect, useCallback } from 'react';
import WasmManager from '../utils/wasm-manager';


export const useWasm = () => {
    const [isReady, setIsReady] = useState(false);
    const [error, setError] = useState<Error | null>(null);
    const [wasmManager] = useState(() => WasmManager.getInstance());

    useEffect(() => {
        console.log('wasmManager mounted');
        wasmManager
            .init()
            .then(() => setIsReady(true))
            .catch((err) => setError(err));
    }, [wasmManager]);

    const getMaps = useCallback(() => {
        return wasmManager.getMaps();
    }, [wasmManager]);

    const runWeb = useCallback(async () => {
        return wasmManager.runWeb();
    }, [wasmManager]);

    return {
        isReady,
        error,
        getMaps,
        runWeb,
    } as const;
};
