import React from 'react';
import { fetchAllByFile } from '@/app/utils/request';
import { GameAssets } from '@/app/types/game';
import { getTranslations } from 'next-intl/server';
import { generateStaticParams } from '@/lib/getStatic';

export default async function Buff() {
    const allBuffs = fetchAllByFile<GameAssets>('game_assets.json');

    const buffs = allBuffs.buffs;
    const t = await getTranslations();

    const safeGetI18 = (key: string) => {
        const hasI18 = t.has(`buffs.${key}`);
        return hasI18 ? t(`buffs.${key}`) : key;
    }
    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
            <main className="max-w-7xl mx-auto">
                <div className="grid grid-cols-2 gap-2">
                    {buffs?.map((buffEntry,) => {

                        return (
                            <div
                                key={buffEntry.guid}
                                className="p-4 mb-2 rounded-lg border border-gray-200 hover:border-gray-300 hover:shadow-sm transition-all"
                            >
                                <div className="flex items-center gap-3 mb-2">
                                    <span className="text-xs font-semibold text-gray-600 bg-gray-100 px-2 py-1 rounded">
                                        #{buffEntry.raw.id}
                                    </span>
                                    <h3 className="font-medium text-gray-900 dark:text-gray-400">
                                        {safeGetI18(buffEntry.raw?.displayName)}
                                    </h3>
                                </div>
                                <p className="text-sm text-gray-500 leading-relaxed">
                                    {safeGetI18(buffEntry.raw?.description)}
                                </p>
                            </div>
                        )
                    })}
                </div>
            </main>
        </div>
    )
}

export { generateStaticParams };
export const dynamicParams = false;