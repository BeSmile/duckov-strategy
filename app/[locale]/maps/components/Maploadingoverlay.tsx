'use client';

import { useEffect, useState } from 'react';
import { useTranslations } from 'next-intl';
import { cn } from '@/lib/utils';
import { SceneLoadingState } from '@/public/wasm/wgpu_renderer';

type TF = ReturnType<typeof useTranslations>;

interface MapLoadingOverlayProps {
    state: SceneLoadingState;
    progress: number;
    message: string;
    className?: string;
    onRetry?: () => void;
}

interface StateConfig {
    label: string;
    icon: string;
    color: string;
    category: 'idle' | 'loading' | 'disposing' | 'ready' | 'error' | 'recovery';
}

const getStateConfig = (state: SceneLoadingState, t: TF): StateConfig => {
    const configs: Record<SceneLoadingState, StateConfig> = {
        // 基础状态
        [SceneLoadingState.Idle]: {
            label: t('maps.loading.idle'),
            icon: '○',
            color: 'text-zinc-400',
            category: 'idle',
        },

        // 初始化阶段
        [SceneLoadingState.Initializing]: {
            label: t('maps.loading.initializing'),
            icon: '◐',
            color: 'text-amber-500',
            category: 'loading',
        },
        [SceneLoadingState.InitFailed]: {
            label: t('maps.loading.init_failed'),
            icon: '✕',
            color: 'text-red-500',
            category: 'error',
        },

        // 加载阶段
        [SceneLoadingState.LoadingScene]: {
            label: t('maps.loading.loading_scene'),
            icon: '◑',
            color: 'text-sky-500',
            category: 'loading',
        },
        [SceneLoadingState.LoadingAssets]: {
            label: t('maps.loading.loading_assets'),
            icon: '◒',
            color: 'text-violet-500',
            category: 'loading',
        },
        [SceneLoadingState.LoadingProgress]: {
            label: t('maps.loading.loading_progress'),
            icon: '◒',
            color: 'text-violet-500',
            category: 'loading',
        },

        // 设置阶段
        [SceneLoadingState.Setting]: {
            label: t('maps.loading.setting'),
            icon: '◓',
            color: 'text-emerald-500',
            category: 'loading',
        },
        [SceneLoadingState.Building]: {
            label: t('maps.loading.building'),
            icon: '◔',
            color: 'text-teal-500',
            category: 'loading',
        },

        // 就绪状态
        [SceneLoadingState.Ready]: {
            label: t('maps.loading.ready'),
            icon: '●',
            color: 'text-emerald-400',
            category: 'ready',
        },
        [SceneLoadingState.Running]: {
            label: t('maps.loading.running'),
            icon: '▶',
            color: 'text-emerald-400',
            category: 'ready',
        },
        [SceneLoadingState.Paused]: {
            label: t('maps.loading.paused'),
            icon: '❚❚',
            color: 'text-amber-400',
            category: 'ready',
        },

        // 场景切换
        [SceneLoadingState.Unloading]: {
            label: t('maps.loading.unloading'),
            icon: '↓',
            color: 'text-orange-500',
            category: 'disposing',
        },
        [SceneLoadingState.Switching]: {
            label: t('maps.loading.switching'),
            icon: '⇄',
            color: 'text-sky-500',
            category: 'disposing',
        },
        [SceneLoadingState.HotReloading]: {
            label: t('maps.loading.hot_reloading'),
            icon: '↻',
            color: 'text-amber-500',
            category: 'loading',
        },

        // 资源管理
        [SceneLoadingState.DisposingAssets]: {
            label: t('maps.loading.disposing_assets'),
            icon: '◌',
            color: 'text-orange-400',
            category: 'disposing',
        },
        [SceneLoadingState.DisposingScene]: {
            label: t('maps.loading.disposing_scene'),
            icon: '◌',
            color: 'text-orange-500',
            category: 'disposing',
        },
        [SceneLoadingState.DisposingAll]: {
            label: t('maps.loading.disposing_all'),
            icon: '◌',
            color: 'text-orange-600',
            category: 'disposing',
        },

        // 错误状态
        [SceneLoadingState.Error]: {
            label: t('maps.loading.error'),
            icon: '✕',
            color: 'text-red-500',
            category: 'error',
        },
        [SceneLoadingState.AssetLoadError]: {
            label: t('maps.loading.asset_load_error'),
            icon: '✕',
            color: 'text-red-500',
            category: 'error',
        },
        [SceneLoadingState.SceneParseError]: {
            label: t('maps.loading.scene_parse_error'),
            icon: '✕',
            color: 'text-red-500',
            category: 'error',
        },
        [SceneLoadingState.RenderError]: {
            label: t('maps.loading.render_error'),
            icon: '✕',
            color: 'text-red-500',
            category: 'error',
        },

        // 恢复状态
        [SceneLoadingState.Recovering]: {
            label: t('maps.loading.recovering'),
            icon: '↺',
            color: 'text-amber-500',
            category: 'recovery',
        },
        [SceneLoadingState.Restarting]: {
            label: t('maps.loading.restarting'),
            icon: '⟳',
            color: 'text-amber-500',
            category: 'recovery',
        },
    };
    return configs[state];
};

// 辅助函数
const isLoadingState = (config: StateConfig): boolean => {
    return config.category === 'loading' || config.category === 'disposing' || config.category === 'recovery';
};

const isErrorState = (config: StateConfig): boolean => {
    return config.category === 'error';
};

const isHiddenState = (state: SceneLoadingState): boolean => {
    return state === SceneLoadingState.Idle ||
        state === SceneLoadingState.Ready ||
        state === SceneLoadingState.Running ||
        state === SceneLoadingState.Paused;
};

// 步骤指示器的状态顺序
const loadingSteps = [
    SceneLoadingState.Initializing,
    SceneLoadingState.LoadingScene,
    SceneLoadingState.LoadingAssets,
    SceneLoadingState.Setting,
    SceneLoadingState.Building,
];

const disposingSteps = [
    SceneLoadingState.DisposingScene,
    SceneLoadingState.DisposingAssets,
    SceneLoadingState.DisposingAll,
];

export function MapLoadingOverlay({
                                      state,
                                      progress,
                                      message,
                                      className,
                                      onRetry,
                                  }: MapLoadingOverlayProps) {
    const t = useTranslations();
    const [dots, setDots] = useState('');
    const config = getStateConfig(state, t);
    const isLoading = isLoadingState(config);
    const isError = isErrorState(config);
    const isDisposing = config.category === 'disposing';

    // 动态省略号动画
    useEffect(() => {
        if (!isLoading) return;
        const interval = setInterval(() => {
            setDots((prev) => (prev.length >= 3 ? '' : prev + '.'));
        }, 400);
        return () => clearInterval(interval);
    }, [isLoading]);

    // 准备完成时自动隐藏
    if (isHiddenState(state)) {
        return null;
    }

    // 获取当前显示的步骤指示器
    const currentSteps = isDisposing ? disposingSteps : loadingSteps;

    // 计算步骤状态
    const getStepStatus = (stepState: SceneLoadingState): 'completed' | 'active' | 'pending' => {
        if (isDisposing) {
            const currentIndex = disposingSteps.indexOf(state);
            const stepIndex = disposingSteps.indexOf(stepState);
            if (stepIndex < currentIndex) return 'completed';
            if (stepIndex === currentIndex) return 'active';
            return 'pending';
        }

        // 加载状态
        const stateOrder = [
            SceneLoadingState.Initializing,
            SceneLoadingState.LoadingScene,
            SceneLoadingState.LoadingAssets,
            SceneLoadingState.Setting,
            SceneLoadingState.Building,
        ];
        const currentIndex = stateOrder.indexOf(state);
        const stepIndex = stateOrder.indexOf(stepState);

        if (currentIndex === -1) return 'pending'; // 当前状态不在列表中
        if (stepIndex < currentIndex) return 'completed';
        if (stepIndex === currentIndex) return 'active';
        return 'pending';
    };

    // 获取渐变颜色
    const getGradientColors = () => {
        if (isError) return 'from-red-500/10 via-transparent to-transparent';
        if (isDisposing) return 'from-orange-500/10 via-transparent to-transparent';
        return 'from-sky-500/10 via-transparent to-transparent';
    };

    const getProgressGradient = () => {
        if (isError) return 'linear-gradient(90deg, #ef4444, #f97316)';
        if (isDisposing) return 'linear-gradient(90deg, #f97316, #eab308, #22c55e)';
        return 'linear-gradient(90deg, #0ea5e9, #8b5cf6, #10b981)';
    };

    return (
        <div
            className={cn(
                'absolute inset-0 z-50 flex items-center justify-center',
                'bg-zinc-950/90 backdrop-blur-md',
                className
            )}
            role={isError ? 'alert' : 'status'}
            aria-live={isError ? 'assertive' : 'polite'}
            aria-label={isError ? `${t('maps.loading.error')}: ${config.label}` : `${config.label} ${Math.round(progress)}%`}
        >
            {/* 背景装饰 */}
            <div className="absolute inset-0 overflow-hidden" aria-hidden="true">
                <div className={cn(
                    "absolute -top-1/2 -left-1/2 w-full h-full bg-gradient-to-br rounded-full blur-3xl animate-pulse",
                    getGradientColors()
                )} />
                <div className={cn(
                    "absolute -bottom-1/2 -right-1/2 w-full h-full bg-gradient-to-tl rounded-full blur-3xl animate-pulse delay-700",
                    isError ? 'from-orange-500/10 via-transparent to-transparent' :
                        isDisposing ? 'from-yellow-500/10 via-transparent to-transparent' :
                            'from-violet-500/10 via-transparent to-transparent'
                )} />
            </div>

            {/* 网格背景 */}
            <div
                className="absolute inset-0 opacity-[0.03]"
                style={{
                    backgroundImage: `
                        linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px),
                        linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px)
                    `,
                    backgroundSize: '40px 40px',
                }}
                aria-hidden="true"
            />

            {/* 主内容区 */}
            <div className="relative flex flex-col items-center gap-8 px-8 py-10 max-w-md w-full">
                {/* 加载动画 - 同心圆环 */}
                <div
                    className="relative w-32 h-32"
                    role="img"
                    aria-label={`${t('maps.loading.progress')} ${Math.round(progress)}%`}
                >
                    {/* 外圈 - 旋转 */}
                    <svg
                        className={cn(
                            "absolute inset-0 w-full h-full",
                            isError ? "" : "animate-spin"
                        )}
                        style={{ animationDuration: isDisposing ? '2s' : '3s' }}
                        viewBox="0 0 100 100"
                        aria-hidden="true"
                    >
                        <circle
                            cx="50"
                            cy="50"
                            r="46"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="1"
                            className="text-zinc-800"
                        />
                        <circle
                            cx="50"
                            cy="50"
                            r="46"
                            fill="none"
                            stroke={isError ? "#ef4444" : "url(#gradient1)"}
                            strokeWidth="2"
                            strokeLinecap="round"
                            strokeDasharray={`${progress * 2.89} 289`}
                            transform="rotate(-90 50 50)"
                            className="transition-all duration-300"
                        />
                        <defs>
                            <linearGradient
                                id="gradient1"
                                x1="0%"
                                y1="0%"
                                x2="100%"
                                y2="100%"
                            >
                                {isDisposing ? (
                                    <>
                                        <stop offset="0%" stopColor="#f97316" />
                                        <stop offset="50%" stopColor="#eab308" />
                                        <stop offset="100%" stopColor="#22c55e" />
                                    </>
                                ) : (
                                    <>
                                        <stop offset="0%" stopColor="#0ea5e9" />
                                        <stop offset="50%" stopColor="#8b5cf6" />
                                        <stop offset="100%" stopColor="#10b981" />
                                    </>
                                )}
                            </linearGradient>
                        </defs>
                    </svg>

                    {/* 中圈 - 反向旋转 */}
                    <svg
                        className={cn(
                            "absolute inset-3 w-[calc(100%-24px)] h-[calc(100%-24px)]",
                            isError ? "" : "animate-spin"
                        )}
                        style={{
                            animationDuration: '2s',
                            animationDirection: 'reverse',
                        }}
                        viewBox="0 0 100 100"
                        aria-hidden="true"
                    >
                        <circle
                            cx="50"
                            cy="50"
                            r="46"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="1"
                            strokeDasharray="4 8"
                            className={isError ? "text-red-900" : "text-zinc-700"}
                        />
                    </svg>

                    {/* 内圈 - 进度数字或错误图标 */}
                    <div
                        className={cn(
                            "absolute inset-6 flex items-center justify-center rounded-full bg-zinc-900/80 border",
                            isError ? "border-red-800" : "border-zinc-800"
                        )}
                        aria-hidden="true"
                    >
                        {isError ? (
                            <span className="text-3xl text-red-500">✕</span>
                        ) : (
                            <div className="text-center">
                                <span className="text-3xl font-light tracking-tight text-white tabular-nums">
                                    {Math.round(progress)}
                                </span>
                                <span className="text-lg text-zinc-500">%</span>
                            </div>
                        )}
                    </div>

                    {/* 脉冲点 */}
                    {!isError && (
                        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2" aria-hidden="true">
                            <div className={cn(
                                "w-2 h-2 rounded-full animate-ping opacity-75",
                                isDisposing ? "bg-orange-500" : "bg-sky-500"
                            )} />
                        </div>
                    )}
                </div>

                {/* 状态信息 */}
                <div className="flex flex-col items-center gap-3 text-center">
                    {/* 状态标签 */}
                    <div className="flex items-center gap-2">
                        <span
                            className={cn(
                                'text-xl',
                                isLoading && 'animate-pulse',
                                config.color
                            )}
                        >
                            {config.icon}
                        </span>
                        <span className="text-lg font-medium text-zinc-200">
                            {config.label}
                            {isLoading && (
                                <span className="text-zinc-500">{dots}</span>
                            )}
                        </span>
                    </div>

                    {/* 详细消息 */}
                    {message && (
                        <p className={cn(
                            "text-sm max-w-xs truncate px-4",
                            isError ? "text-red-400" : "text-zinc-500"
                        )}>
                            {message}
                        </p>
                    )}
                </div>

                {/* 进度条 */}
                <div className="w-full max-w-xs">
                    <div
                        className="h-1 bg-zinc-800 rounded-full overflow-hidden"
                        role="progressbar"
                        aria-valuenow={Math.round(progress)}
                        aria-valuemin={0}
                        aria-valuemax={100}
                        aria-label={`${t('maps.loading.progress')}: ${Math.round(progress)}%`}
                    >
                        <div
                            className="h-full rounded-full transition-all duration-300 ease-out"
                            style={{
                                width: `${progress}%`,
                                background: getProgressGradient(),
                            }}
                        />
                    </div>
                </div>

                {/* 步骤指示器 */}
                <div
                    className="flex items-center gap-2"
                    role="group"
                    aria-label={t('maps.loading.steps')}
                >
                    {currentSteps.map((stepState) => {
                        const status = getStepStatus(stepState);
                        return (
                            <div
                                key={stepState}
                                className={cn(
                                    'w-2 h-2 rounded-full transition-all duration-300',
                                    status === 'completed' && 'bg-emerald-500',
                                    status === 'active' && cn(
                                        'scale-125 animate-pulse',
                                        isDisposing ? 'bg-orange-500' : 'bg-sky-500'
                                    ),
                                    status === 'pending' && 'bg-zinc-700'
                                )}
                                role="status"
                                aria-label={`${getStateConfig(stepState, t).label} - ${
                                    status === 'completed' ? t('maps.loading.completed') :
                                    status === 'active' ? t('maps.loading.active') : t('maps.loading.pending')
                                }`}
                                title={getStateConfig(stepState, t).label}
                            />
                        );
                    })}
                </div>

                {/* 错误状态操作区 */}
                {isError && (
                    <div className="flex flex-col items-center gap-3 mt-2" role="alert">
                        <div className="px-4 py-2 bg-red-500/10 border border-red-500/20 rounded-lg">
                            <p className="text-sm text-red-400">
                                {message || getErrorMessage(state, t)}
                            </p>
                        </div>
                        <div className="flex gap-2">
                            <button
                                className="px-4 py-2 text-sm font-medium text-white bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-sky-500/50"
                                onClick={onRetry || (() => window.location.reload())}
                                aria-label={t('maps.loading.retry')}
                            >
                                {t('maps.loading.retry')}
                            </button>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}

// 根据错误状态获取默认错误消息
function getErrorMessage(state: SceneLoadingState, t: TF): string {
    switch (state) {
        case SceneLoadingState.InitFailed:
            return t('maps.loading.error_init_failed');
        case SceneLoadingState.AssetLoadError:
            return t('maps.loading.error_asset_load');
        case SceneLoadingState.SceneParseError:
            return t('maps.loading.error_scene_parse');
        case SceneLoadingState.RenderError:
            return t('maps.loading.error_render');
        default:
            return t('maps.loading.error_generic');
    }
}