'use client';

import Image from 'next/image';
import Link from 'next/link';
import { Monster } from '../types/monster';
import { Item, KeyValue } from '../types/item';
import { getName } from '@/app/components/ItemCard';
import { useTranslation } from 'react-i18next';

interface MonsterCardProps {
    monster: Monster;
    items: Item[];
    monsterLangs: KeyValue;
    itemsLangs: KeyValue;
}

export default function MonsterCard({
    monster,
    items,
    monsterLangs,
    itemsLangs,
}: MonsterCardProps) {
    const { t } = useTranslation();

    // Get items from drops
    const dropItems = monster.itemsToGenerate
        .flatMap((drop) =>
            drop.itemPool.entries.map((entry) => {
                const item = items.find((i) => i.id === entry.value.itemTypeID);
                return {
                    item,
                    chance: drop.chance,
                    weight: entry.weight,
                    comment: drop.comment,
                };
            })
        )
        .filter((dropItem) => dropItem.item);

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
                            {/*/{monster.m_Name}*/}
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
                        <div className="max-h-64 overflow-y-auto pr-1">
                            <div className="grid grid-cols-2 gap-2">
                                {dropItems.map((dropItem, idx) => (
                                    <Link
                                        key={idx}
                                        href={`/inventory/${dropItem.item!.id}`}
                                        className="flex items-center gap-2 p-2 bg-gray-50 dark:bg-gray-700 rounded border border-gray-200 dark:border-gray-600 h-14 hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
                                    >
                                        <div className="relative w-8 h-8 flex-shrink-0 bg-gray-100 dark:bg-gray-600 rounded">
                                            <Image
                                                src={`/images/${dropItem.item!.icon}`}
                                                alt={dropItem.item!.displayName}
                                                fill
                                                loading="lazy"
                                                className="object-contain"
                                                sizes="32px"
                                            />
                                        </div>
                                        <div className="flex-1 min-w-0">
                                            <p className="text-xs font-medium text-gray-900 dark:text-gray-100 truncate">
                                                {getName(
                                                    itemsLangs,
                                                    dropItem.item as Item
                                                )}
                                            </p>
                                            <p className="text-xs text-gray-500 dark:text-gray-400">
                                                {(
                                                    dropItem.chance * 100
                                                ).toFixed(1)}
                                                %
                                            </p>
                                        </div>
                                    </Link>
                                ))}
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}
