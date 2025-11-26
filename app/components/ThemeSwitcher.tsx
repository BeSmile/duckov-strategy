'use client';

import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';

export default function ThemeSwitcher() {
    const { theme, setTheme } = useTheme();
    const [mounted, setMounted] = useState(false);

    useEffect(() => {
        setMounted(true);
    }, []);

    if (!mounted) {
        return (
            <div className="w-9 h-9 rounded-lg bg-gray-200 dark:bg-gray-800 animate-pulse" />
        );
    }

    const themes = [
        { value: 'light', label: '☀️', title: '浅色主题' },
        { value: 'dark', label: '🌙', title: '深色主题' },
        { value: 'cassette', label: '📼', title: '磁带未来主义' },
    ];

    return (
        <div className="flex items-center gap-1 p-1 rounded-lg bg-gray-100 dark:bg-gray-800">
            {themes.map((t) => (
                <button
                    key={t.value}
                    onClick={() => setTheme(t.value)}
                    className={`
                        w-9 h-9 rounded-md flex items-center justify-center text-lg
                        transition-all duration-200
                        ${
                            theme === t.value
                                ? 'bg-white dark:bg-gray-700 shadow-sm scale-105'
                                : 'hover:bg-gray-200 dark:hover:bg-gray-700 opacity-60 hover:opacity-100'
                        }
                    `}
                    title={t.title}
                    aria-label={t.title}
                >
                    {t.label}
                </button>
            ))}
        </div>
    );
}