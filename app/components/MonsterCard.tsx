import { Monster } from '../types/monster';
import { Item, KeyValue } from '../types/item';
import { ItemLink } from '@/app/components/ItemCard';

import { getTranslations } from 'next-intl/server';
import { Language } from '@/app/i18n/config';
import React from 'react';

type MonsterCardProps = {
    monster: Monster;
    items: Item[];
    monsterLangs: KeyValue;
    itemsLangs: KeyValue;
    locale: Language;
};

export default async function MonsterCard({
    monster,
    items,
    monsterLangs,
    itemsLangs,
    locale,
}: MonsterCardProps) {
    const t = await getTranslations();

    // item有2层爆率
    const dropItems = monster.itemsToGenerate
        .map((drop) => {
            const entries = drop.itemPool.entries;
            const hasPer = entries?.some((item) => item.percent); // 有掉率
            const needCalPer = !hasPer; // 需要计算
            const isSingle = entries.length <= 1;
            const itemPools = entries
                .flatMap((entry) => {
                    const item = items.find(
                        (i) => i.id === entry.value.itemTypeID
                    );
                    // 隐藏玩家不可见
                    if (item?.tags.includes('DestroyOnLootBox')) {
                        return {
                            item: undefined,
                        };
                    }
                    const chance = entry?.percent
                        ? Number(entry.percent.replace('%', '')) / 100
                        : needCalPer && !isSingle
                          ? 1 / entries.length
                          : drop.chance;

                    return {
                        item,
                        chance,
                    };
                })
                .filter((obj) => obj?.item);

            return {
                chance: drop.chance,
                itemPools,
                // weight: entry.weight,
                comment: drop.comment,
            };
        })
        .filter((dropItem) => dropItem.itemPools);

    const enName = monster.nameKey || monster.m_Name;

    return (
        <div className="border border-gray-200 rounded-lg p-6 hover:shadow-lg transition-shadow dark:border-gray-700 bg-white dark:bg-gray-800">
            <div className="flex flex-col gap-4">
                {/* Monster Header */}
                <div className="flex items-center gap-4 border-b border-gray-200 dark:border-gray-700 pb-4">
                    <div className="flex-1">
                        <h3 className="font-bold text-lg text-gray-900 dark:text-gray-100">
                            {monsterLangs?.[monster.nameKey] || enName}
                        </h3>
                        <p className="text-sm text-gray-500 dark:text-gray-400">
                            {enName}
                            {/*/{monster.ts.m_Name}*/}
                        </p>
                    </div>
                </div>

                {/* Stats */}
                <div className="grid grid-cols-2 gap-3 text-sm">
                    <div className="flex items-center gap-2">
                        <span className="text-red-500 font-semibold">
                            {t('monsters.hp')}:
                        </span>
                        <span className="text-gray-900 dark:text-gray-100 font-medium">
                            {monster.health}
                        </span>
                    </div>
                    <div className="flex items-center gap-2">
                        <span className="text-blue-500 font-semibold">
                            {t('monsters.exp')}:
                        </span>
                        <span className="text-gray-900 dark:text-gray-100 font-medium">
                            {monster.exp}
                        </span>
                    </div>
                    {monster.moveSpeedFactor && (
                        <div className="flex items-center gap-2">
                            <span className="text-green-500 font-semibold">
                                {t('monsters.speed')}:
                            </span>
                            <span className="text-gray-900 dark:text-gray-100">
                                {monster.moveSpeedFactor}x
                            </span>
                        </div>
                    )}
                    {monster.damageMultiplier && (
                        <div className="flex items-center gap-2">
                            <span className="text-orange-500 font-semibold">
                                {t('monsters.damage')}:
                            </span>
                            <span className="text-gray-900 dark:text-gray-100">
                                {monster.damageMultiplier}x
                            </span>
                        </div>
                    )}
                </div>

                {/* Element Resistances */}
                {(monster.elementFactor_Physics !== undefined ||
                    monster.elementFactor_Fire !== undefined ||
                    monster.elementFactor_Poison !== undefined ||
                    monster.elementFactor_Electricity !== undefined ||
                    monster.elementFactor_Space !== undefined ||
                    monster.elementFactor_Ghost !== undefined) && (
                    <div className="border-t border-gray-200 dark:border-gray-700 pt-3">
                        <h4 className="text-xs font-semibold text-gray-600 dark:text-gray-400 mb-2">
                            {t('monsters.element_resistance')}
                        </h4>
                        <div className="grid grid-cols-3 gap-2 text-xs">
                            {monster.elementFactor_Physics !== undefined && (
                                <div className="flex justify-between">
                                    <span className="text-gray-600 dark:text-gray-400">
                                        {t('monsters.physics')}:
                                    </span>
                                    <span className="font-medium text-gray-900 dark:text-gray-100">
                                        {monster.elementFactor_Physics}x
                                    </span>
                                </div>
                            )}
                            {monster.elementFactor_Fire !== undefined && (
                                <div className="flex justify-between">
                                    <span className="text-gray-600 dark:text-gray-400">
                                        {t('monsters.fire')}:
                                    </span>
                                    <span className="font-medium text-gray-900 dark:text-gray-100">
                                        {monster.elementFactor_Fire}x
                                    </span>
                                </div>
                            )}
                            {monster.elementFactor_Poison !== undefined && (
                                <div className="flex justify-between">
                                    <span className="text-gray-600 dark:text-gray-400">
                                        {t('monsters.poison')}:
                                    </span>
                                    <span className="font-medium text-gray-900 dark:text-gray-100">
                                        {monster.elementFactor_Poison}x
                                    </span>
                                </div>
                            )}
                            {monster.elementFactor_Electricity !==
                                undefined && (
                                <div className="flex justify-between">
                                    <span className="text-gray-600 dark:text-gray-400">
                                        {t('monsters.elec')}:
                                    </span>
                                    <span className="font-medium text-gray-900 dark:text-gray-100">
                                        {monster.elementFactor_Electricity}x
                                    </span>
                                </div>
                            )}
                            {monster.elementFactor_Space !== undefined && (
                                <div className="flex justify-between">
                                    <span className="text-gray-600 dark:text-gray-400">
                                        {t('monsters.space')}:
                                    </span>
                                    <span className="font-medium text-gray-900 dark:text-gray-100">
                                        {monster.elementFactor_Space}x
                                    </span>
                                </div>
                            )}
                            {monster.elementFactor_Ghost !== undefined && (
                                <div className="flex justify-between">
                                    <span className="text-gray-600 dark:text-gray-400">
                                        {t('monsters.ghost')}:
                                    </span>
                                    <span className="font-medium text-gray-900 dark:text-gray-100">
                                        {monster.elementFactor_Ghost}x
                                    </span>
                                </div>
                            )}
                        </div>
                    </div>
                )}

                {/* Drops */}
                {dropItems.length > 0 && (
                    <div className="border-t border-gray-200 dark:border-gray-700 pt-3">
                        <h4 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">
                            {t('monsters.drops_items', {
                                count: dropItems.length,
                            })}
                        </h4>
                        <div className="max-h-64 overflow-y-auto pr-1 relative">
                            <div className="grid  grid-cols-2 gap-2">
                                {dropItems.map((dropItem, ix) => {
                                    const mapping = [
                                        'border-pink-500 dark:border-pink-300',
                                        'border-orange-400 dark:border-orange-200',
                                        'border-emerald-500 dark:border-emerald-300',
                                        'border-cyan-500 dark:border-cyan-300',
                                    ];
                                    const group_show =
                                        dropItem.itemPools.length > 1;
                                    const tipShow =
                                        group_show && dropItem.chance < 1;

                                    const color = !group_show
                                        ? undefined
                                        : mapping[ix % 3];

                                    // 生成svg连起来
                                    return dropItem.itemPools.map(
                                        (itemPool, idx) => {
                                            return (
                                                <ItemLink
                                                    border={color}
                                                    key={`${ix}-${idx}-${itemPool.item?.id}`}
                                                    extra={
                                                        <p
                                                            className={`text-xs text-gray-500 dark:text-gray-400`}
                                                        >
                                                            {tipShow ? (
                                                                <>
                                                                    <span>
                                                                        {dropItem.chance *
                                                                            100}
                                                                        %
                                                                    </span>
                                                                    x(
                                                                    {(
                                                                        (itemPool?.chance ||
                                                                            0) *
                                                                        100
                                                                    ).toFixed(
                                                                        2
                                                                    )}
                                                                    %)
                                                                </>
                                                            ) : (
                                                                <>
                                                                    {(
                                                                        (itemPool?.chance ||
                                                                            0) *
                                                                        100
                                                                    ).toFixed(
                                                                        2
                                                                    )}
                                                                    %
                                                                </>
                                                            )}
                                                        </p>
                                                    }
                                                    locale={locale}
                                                    item={
                                                        itemPool.item as unknown as Item
                                                    }
                                                    itemsLangs={itemsLangs}
                                                />
                                            );
                                        }
                                    );
                                })}
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
