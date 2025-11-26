'use client'

import { useCallback, useEffect, useState } from 'react';
import { useWasm } from '@/app/hooks/useWasm';
import { MapInfo } from '@/app/utils/wasm-manager';


export default function DuckMap () {
    const [selectedMapId, setSelectedMapId] = useState<number>(0);
    const [map, setMap] = useState<MapInfo[]>([]);

    const { isReady, getMaps, runWeb } = useWasm();

    useEffect(() => {
        if (isReady) {
            // eslint-disable-next-line react-hooks/exhaustive-deps
            setMap(() => getMaps());
        }
    }, [isReady, getMaps]);

    // 初始化画布
    const initCanvas = useCallback(async () => {
        if (isReady){
            await runWeb();
        }
    }, [isReady, runWeb]);

    useEffect(() => {
        void initCanvas();
    }, [initCanvas]);

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
            <main className="max-w-7xl mx-auto">
                <div className="mb-6">
                    <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-4">
                        地图展览
                    </h1>

                    <div className="grid grid-cols-1 md:grid-cols-6 gap-4 mb-6">
                        {map?.map(mapInfo => (
                            <div
                                key={mapInfo.id}
                                onClick={() => setSelectedMapId(mapInfo.id)}
                                className={`
                                    p-4 rounded-lg border-2 cursor-pointer transition-all
                                    ${selectedMapId === mapInfo.id
                                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                                        : 'border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 hover:border-blue-300 dark:hover:border-blue-700'
                                    }
                                `}
                            >
                                <h3 className="font-semibold text-lg text-gray-900 dark:text-gray-100 mb-2">
                                    {mapInfo.cn}
                                </h3>
                                {/*<p className="text-sm text-gray-600 dark:text-gray-400 mb-1">*/}
                                {/*    ID: {mapInfo.id}*/}
                                {/*</p>*/}
                                <p className="text-xs text-gray-500 dark:text-gray-500 truncate" title={mapInfo.path}>
                                    {mapInfo.path}
                                </p>
                                {mapInfo.disabled_ids.length > 0 && (
                                    <p className="text-xs text-red-600 dark:text-red-400 mt-2">
                                        禁用对象: {mapInfo.disabled_ids.join(', ')}
                                    </p>
                                )}
                            </div>
                        ))}
                    </div>

                    {/*{selectedMapId && (*/}
                    {/*    <div className="p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-300 dark:border-gray-700">*/}
                    {/*        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">*/}
                    {/*            当前选择: {maps[selectedMapId].name}*/}
                    {/*        </h2>*/}
                    {/*        <p className="text-sm text-gray-600 dark:text-gray-400">*/}
                    {/*            路径: {maps[selectedMapId].path}*/}
                    {/*        </p>*/}
                    {/*    </div>*/}
                    {/*)}*/}
                </div>

                <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-300 dark:border-gray-700">
                    <canvas className="w-full h-full" id="canvas"/>
                </div>
            </main>
        </div>
    )
}