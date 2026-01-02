'use client';

import { useCallback, useEffect, useState } from 'react';
import { useTranslations } from 'next-intl';
import { useWasm } from '@/app/hooks/useWasm';
import { MapInfo } from '@/app/utils/wasm-manager';
import {
    MapLoadingOverlay,
    SceneLoadingState,
} from './components/Maploadingoverlay';
import { cn } from '@/lib/utils';

// 辅助函数：判断是否正在处理中（不应被打断）
const isBusyState = (state: SceneLoadingState): boolean => {
    return [
        SceneLoadingState.Initializing,
        SceneLoadingState.LoadingScene,
        SceneLoadingState.LoadingAssets,
        SceneLoadingState.Setting,
        SceneLoadingState.Building,
        SceneLoadingState.Switching,
        SceneLoadingState.HotReloading,
        SceneLoadingState.Unloading,
        SceneLoadingState.DisposingAssets,
        SceneLoadingState.DisposingScene,
        SceneLoadingState.DisposingAll,
        SceneLoadingState.Recovering,
        SceneLoadingState.Restarting,
    ].includes(state);
};

// 辅助函数：判断是否可以切换场景
const canChangeScene = (state: SceneLoadingState): boolean => {
    return [
        SceneLoadingState.Ready,
        SceneLoadingState.Running,
        SceneLoadingState.Paused,
    ].includes(state);
};

// 辅助函数：判断是否正在切换/清理场景
const isChangingScene = (state: SceneLoadingState): boolean => {
    return [
        SceneLoadingState.Switching,
        SceneLoadingState.Unloading,
        SceneLoadingState.DisposingScene,
        SceneLoadingState.DisposingAssets,
        SceneLoadingState.DisposingAll,
        SceneLoadingState.HotReloading,
    ].includes(state);
};

export default function DuckMap() {
    const t = useTranslations();
    const [selectedMapId, setSelectedMapId] = useState<number>(1012);
    const [map, setMap] = useState<MapInfo[]>([]);
    const [state, setState] = useState<SceneLoadingState>(
        SceneLoadingState.Idle
    );
    const [progress, setProgress] = useState<number>(0);
    const [message, setMessage] = useState<string>('');
    const [canvasInitialized, setCanvasInitialized] = useState(false);

    const {
        isReady,
        getMaps,
        setScenePath,
        changeScene,
        runWeb,
        getLoadingMessage,
        getLoadingProgress,
        getLoadingState,
    } = useWasm();

    // 派生状态
    const isBusy = isBusyState(state);
    const isSceneChanging = isChangingScene(state);

    useEffect(() => {
        let timer: ReturnType<typeof setInterval> | null = null;
        if (isReady) {
            timer = setInterval(async () => {
                const newState = await getLoadingState();
                const newProgress = await getLoadingProgress();
                const newMessage = await getLoadingMessage();

                setState((oldState: SceneLoadingState) => {
                    if (oldState === newState) return oldState;
                    return newState;
                });
                setProgress(newProgress);
                setMessage(newMessage);
            }, 1000 / 60);
        }
        return () => {
            if (timer) {
                clearInterval(timer);
            }
        };
    }, [isReady, getLoadingState, getLoadingProgress, getLoadingMessage]);

    useEffect(() => {
        if (isReady) {
            const maps = getMaps();
            setMap(maps);
            console.log(1, 2, 3, maps);

            if (maps.length > 0 && !canvasInitialized) {
                const defaultMap =
                    maps.find((m) => m.id === selectedMapId) || maps[0];
                console.log('Setting initial scene2:', defaultMap.path);
                setScenePath(defaultMap.path);
            }
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [isReady, setScenePath, selectedMapId, canvasInitialized]);

    const initCanvas = useCallback(async () => {
        if (isReady && !canvasInitialized) {
            await runWeb();
            setCanvasInitialized(true);
        }
    }, [isReady, runWeb, canvasInitialized]);

    useEffect(() => {
        void initCanvas();
    }, [initCanvas]);


    const handleMapSelect = useCallback(
        (mapInfo: MapInfo) => {
            console.log('Selected map:', mapInfo);

            // 如果正在处理中，不允许操作
            if (isBusyState(state)) {
                console.log('Cannot change scene while busy, current state:', state);
                return;
            }

            if (canChangeScene(state)) {
                console.log('Changing scene to:', mapInfo.path);
                try {
                    changeScene(mapInfo.path);
                    setSelectedMapId(mapInfo.id);
                } catch (error) {
                    console.error('Failed to change scene:', error);
                }
            } else {
                setSelectedMapId(mapInfo.id);
                setScenePath(mapInfo.path);
            }
        },
        [changeScene, setScenePath, state]
    );

    useEffect(() => {
        const savedMapId = localStorage.getItem('selectedMapId');
        if (savedMapId) {
            setSelectedMapId(parseInt(savedMapId));
        }
    }, []);

    // 获取状态显示文本
    const getStatusText = (): string => {
        if (isSceneChanging) return t('maps.switching_scene');
        if (isBusy) return t('maps.processing');
        return '';
    };

    return (
        <div className="min-h-screen bg-zinc-950 py-8 px-4">
            <main className="max-w-7xl mx-auto">
                {/* 标题区 */}
                <header className="mb-8">
                    <div className="flex items-center justify-between mb-6">
                        <div className="flex items-center gap-4">
                            <div className="w-1 h-8 bg-gradient-to-b from-sky-500 to-violet-500 rounded-full" aria-hidden="true" />
                            <h1 className="text-3xl font-semibold tracking-tight text-zinc-100">
                                {t('maps.title')}
                            </h1>
                        </div>
                        {isBusy && (
                            <div
                                className="flex items-center gap-3 px-4 py-2 bg-zinc-900 border border-zinc-800 rounded-lg"
                                role="status"
                                aria-live="polite"
                            >
                                <div className="relative w-4 h-4" aria-hidden="true">
                                    <div className="absolute inset-0 rounded-full border-2 border-sky-500/30" />
                                    <div className="absolute inset-0 rounded-full border-2 border-sky-500 border-t-transparent animate-spin" />
                                </div>
                                <span className="text-sm font-medium text-zinc-300">
                                    {getStatusText()}
                                </span>
                            </div>
                        )}
                    </div>

                    {/* 地图选择网格 */}
                    <nav
                        className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-3"
                        aria-label={t('maps.map_list')}
                        role="navigation"
                    >
                        {map?.map((mapInfo) => (
                            <button
                                key={mapInfo.id}
                                onClick={() => handleMapSelect(mapInfo)}
                                disabled={isBusy}
                                aria-label={`${t('maps.select_map')}: ${mapInfo.cn}`}
                                aria-pressed={selectedMapId === mapInfo.id}
                                aria-disabled={isBusy}
                                className={cn(
                                    'group relative p-4 rounded-xl border transition-all duration-200 text-left',
                                    'focus:outline-none focus:ring-2 focus:ring-sky-500/50',
                                    isBusy && 'cursor-not-allowed opacity-50',
                                    selectedMapId === mapInfo.id
                                        ? 'bg-gradient-to-br from-sky-500/10 to-violet-500/10 border-sky-500/50'
                                        : 'bg-zinc-900/50 border-zinc-800 hover:border-zinc-700 hover:bg-zinc-900'
                                )}
                            >
                                {/* 选中指示器 */}
                                {selectedMapId === mapInfo.id && (
                                    <div
                                        className="absolute top-2 right-2 w-2 h-2 rounded-full bg-sky-500 animate-pulse"
                                        aria-hidden="true"
                                    />
                                )}

                                <div className="font-medium text-zinc-100 mb-1 truncate">
                                    {mapInfo.cn}
                                </div>
                                <p
                                    className="text-xs text-zinc-500 truncate"
                                    title={mapInfo.path}
                                >
                                    {mapInfo.path}
                                </p>
                                {mapInfo.disabled_ids.length > 0 && (
                                    <div className="mt-2 flex items-center gap-1">
                                        <span className="w-1.5 h-1.5 rounded-full bg-amber-500" aria-hidden="true" />
                                        <span className="text-xs text-amber-500/80">
                                            {t('maps.disabled_count', { count: mapInfo.disabled_ids.length })}
                                        </span>
                                    </div>
                                )}
                            </button>
                        ))}
                    </nav>
                </header>

                {/* 画布容器 */}
                <section
                    className="relative bg-zinc-900 rounded-2xl border border-zinc-800 overflow-hidden"
                    aria-label={t('maps.map_view')}
                >
                    {/* 装饰角标 */}
                    <div className="absolute top-0 left-0 w-16 h-16 border-l-2 border-t-2 border-zinc-700 rounded-tl-2xl pointer-events-none" aria-hidden="true" />
                    <div className="absolute top-0 right-0 w-16 h-16 border-r-2 border-t-2 border-zinc-700 rounded-tr-2xl pointer-events-none" aria-hidden="true" />
                    <div className="absolute bottom-0 left-0 w-16 h-16 border-l-2 border-b-2 border-zinc-700 rounded-bl-2xl pointer-events-none" aria-hidden="true" />
                    <div className="absolute bottom-0 right-0 w-16 h-16 border-r-2 border-b-2 border-zinc-700 rounded-br-2xl pointer-events-none" aria-hidden="true" />

                    {/* 加载遮罩 */}
                    <MapLoadingOverlay
                        state={state}
                        progress={progress * 100}
                        message={message}
                    />

                    {/* Canvas */}
                    <canvas
                        id="canvas"
                        className="w-full aspect-video"
                        style={{ minHeight: '500px' }}
                        aria-label={`${t('maps.map_view')} - ${map.find(m => m.id === selectedMapId)?.cn || t('maps.select_map')}`}
                        role="img"
                    >
                        {t('maps.loading.canvas_unsupported')}
                    </canvas>
                </section>

                {/* 底部状态栏 */}
                <footer
                    className="mt-4 flex items-center justify-between text-xs text-zinc-600"
                    role="contentinfo"
                    aria-label={t('maps.status_info')}
                >
                    <div className="flex items-center gap-4">
                        {selectedMapId && (
                            <span className="text-zinc-500" aria-label={t('maps.current_map', { id: selectedMapId })}>
                                {t('maps.current_map', { id: selectedMapId })}
                            </span>
                        )}
                    </div>
                </footer>
            </main>
        </div>
    );
}