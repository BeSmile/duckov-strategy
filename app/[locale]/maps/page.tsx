'use client'

// import { useEffect } from 'react';

export default function DuckMap () {
    // const initCanvas = async () => {
    //     console.log('inside duckmap');
    //     const wasm = await import('@/wgpu-renderer/pkg/');
    //     console.log(wasm);
    //     await wasm.default();
    //     await wasm.run_web();
    // }
    //
    // useEffect(() => {
    //     void initCanvas();
    // }, []);

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
            <main className="max-w-7xl mx-auto">
                <canvas className="w-full h-full" id="canvas"/>
            </main>
        </div>
    )
}