'use client';

import {
    Archive,
    Inventory,
    ItemEntry,
    ItemTreeData,
    ItemVariable,
    SlotContent,
} from '@/app/[locale]/archived/types';
import { ItemLink } from '@/app/components/ClientProxy';
import React, { useRef, useState } from 'react';
import { Item, KeyValue } from '@/app/types/item';
import { MonsterData } from '@/app/types/monster';
import { useTranslations } from 'next-intl';
import { getMonsterName, getQualityConfig } from '@/app/constants';
import Image from 'next/image';
import { LocaleLink } from '@/app/components/LocaleLink';
import { IQuestGraph } from '@/app/types/quest';
import { ItemLinkProps } from '@/app/components/ItemCard';
import { LinkProps } from 'next/link';

function parseVariable(variable: ItemVariable) {
    const { dataType, data } = variable;

    const binaryString = atob(data);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
    }

    switch (dataType) {
        case 1: // Float
            return new Float32Array(bytes.buffer)[0];
        case 2: // Int32
            return new Int32Array(bytes.buffer)[0];
        case 3: // Boolean
            return bytes[0] === 1;
        default:
            return null;
    }
}


type ArchiveUploadProps = {
    items: Item[];
    monsters: MonsterData;
    monstersLangs: KeyValue;
    questData: IQuestGraph;
} & Pick<ItemLinkProps, 'itemsLangs' | 'locale'>;

type InventorysProps = {
    inventory: Inventory;
    title: string;
    icon?: React.ReactNode;
    items: Item[];
    isCharacter?: boolean;
} & Pick<ItemLinkProps, 'itemsLangs' | 'locale'>;

enum SORT_ORDER {
    DEFAULT = 'default',
    VALUE = 'value',
    RARITY = 'rarity',
}

type ItemInventory = {
    item: Item | undefined;
    // position: number;
    count: number;
};

const DogSvg = () => (
    <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 100 100"
        width="24"
        height="24"
    >
        <ellipse cx="50" cy="65" rx="18" ry="22" fill="white" />

        <circle cx="30" cy="35" r="8" fill="white" />

        <circle cx="42" cy="28" r="8" fill="white" />

        <circle cx="58" cy="28" r="8" fill="white" />

        <circle cx="70" cy="35" r="8" fill="white" />
    </svg>
);

type CharacterPanelProps = {
    itemData: ItemTreeData;
} & Omit<InventorysProps, 'title' | 'icon' | 'isCharacter' | 'inventory'>;

type ItemInfoModalProps = {
    slots: SlotContent[];
    instanceItems: ItemEntry[];
    items: Item[];
    item: Item;
} & Pick<InventorysProps, 'locale' | 'itemsLangs'>;

const ItemInfoModal: React.FC<ItemInfoModalProps> = ({
    locale,
    item,
    items,
    slots,
    itemsLangs,
    instanceItems,

}) => {
    const t = useTranslations();
    return (
        <div className="absolute  shadow  bg-black/80 rounded-lg  left-full top-0 mt-2 w-160 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-10">
            <div className="text-center text-white p-4 font-bold text-lg">
                {itemsLangs?.[item.displayName] || item?.displayName}
            </div>
            <div className="text-center text-white p-4 font-bold text-sm">
                {itemsLangs?.[item.description] || item?.description}
            </div>
            <div className="p-4  grid grid-cols-6 gap-2">
                {slots.length > 0 &&
                    slots.map((slot, idx) => {
                        const itemEntry = instanceItems.find(
                            (item) => item.instanceID === slot.instanceID
                        );

                        if (!itemEntry) {
                            return <div key={idx}>-</div>;
                        }
                        const slotItem = items.find(
                            (item) => item.id === itemEntry.typeID
                        );

                        if (!slotItem) {
                            return <div key={idx}>-</div>;
                        }

                        const qualityConfig = getQualityConfig(
                            slotItem.quality || 0
                        );

                        return (
                            <div key={idx}>
                                <LocaleLink
                                    locale={locale}
                                    href={`/inventory/${slotItem!.id}`}
                                    key={idx}
                                    className={`relative cursor-pointer transform transition-all duration-200 hover:scale-105`}
                                >
                                    <div
                                        className={`border-t-red-50 border-2 p-4 ${qualityConfig.bgColor} relative rounded-2xl overflow-hidden aspect-square p-4`}
                                    >
                                        <div
                                            className={`flex h-full flex-col items-center justify-center `}
                                        >
                                            <Image
                                                src={`/images/${slotItem!.icon}`}
                                                alt={slotItem!.displayName}
                                                loading="lazy"
                                                fill
                                                className="object-contain"
                                            />
                                        </div>
                                        <div
                                            className={`text-[10px]  text-center right-0 dark:bg-black/50 p-1 ${qualityConfig.textColor} absolute bottom-0`}
                                        >
                                            {slotItem?.displayName
                                                ? itemsLangs?.[
                                                      slotItem?.displayName
                                                  ] || slotItem?.displayName
                                                : slotItem?.displayName}
                                        </div>
                                    </div>
                                    <div className="px-4 py-2 border-slate-700">
                                        <p className="text-white text-sm  text-center">
                                            {t(`slot.accessories.${slot?.slot}`)}
                                        </p>
                                    </div>
                                </LocaleLink>
                            </div>
                        );
                    })}
            </div>
        </div>
    );
};

const CharacterPanel: React.FC<CharacterPanelProps> = ({
    itemData,
    items,
    itemsLangs,
    locale,
}) => {
    const t = useTranslations('archived');
    const tSlot = useTranslations();
    const rootInstanceID = itemData.rootInstanceID; // root 物品
    const entries = itemData?.entries || [];
    const character = entries.find(
        (item) => item.instanceID === rootInstanceID
    );
    const slotContents = character?.slotContents || [];

    // 获取当前身体的id
    const ids = slotContents.flatMap(slot => {
        // 获取物品id
        const item = entries.filter(item => item.instanceID === slot.instanceID);
        // 查看是否存在slot
        return item.flatMap(i => {
            const childrenSlots = entries.filter(item => {
                const childrenSlots = i?.slotContents || [];
                return childrenSlots.filter(slot => slot.instanceID === item.instanceID).length;
            });
            return [i.typeID, ...childrenSlots.map(slo => slo.typeID)];
        })
    })

    const sortSlots=[
        "PrimaryWeapon",
        "SecondaryWeapon",
        "MeleeWeapon",
        "Helmat",
        "Armor",
        "FaceMask",
        "Headset",
        "Backpack",
        "Totem1",
        "Totem2",
    ]

    return (
        <div className="bg-gray-800 rounded-lg shadow  border border-gray-200 dark:border-gray-700">
            <div className="grid grid-cols-5 gap-6 p-4">
                {sortSlots.map((slotName, idx) => {
                    const slotConfig = slotContents.find(item => item.slot === slotName);
                    const entry = entries.find(
                        (entry) => entry.instanceID === slotConfig?.instanceID
                    );
                    const item = items.find(
                        (item) => item.id === entry?.typeID
                    );

                    const qualityConfig = getQualityConfig(item?.quality || 0);

                    const props = (item ? {
                        href: `/inventory/${item!.id}`
                    } : {}) as unknown as LinkProps

                    return (
                        <div key={idx} className="relative inline-block group">
                            <LocaleLink
                                {...props}
                                locale={locale}
                                className={`relative cursor-pointer transform transition-all duration-200 hover:scale-105`}
                            >
                                <div
                                    className={`border-2 p-4 ${qualityConfig.bgColor} relative rounded-2xl overflow-hidden aspect-square p-4`}
                                >
                                    <div
                                        className={`flex h-full flex-col items-center justify-center `}
                                    >
                                        <Image
                                            src={item ?`/images/${item!.icon}` : '/images/cross.webp'}
                                            alt={item?.displayName || 'not found'}
                                            fill
                                            loading="lazy"
                                            className="object-contain"
                                            sizes="(max-width: 768px) 64px, (min-width: 1024px) 80px, 96px"
                                        />
                                    </div>
                                    {item && (
                                        <div
                                            className={`font-normal text-center right-0 dark:bg-black/50 p-1 ${qualityConfig.textColor} absolute bottom-0`}
                                        >
                                            {item?.displayName
                                                ? itemsLangs?.[item?.displayName] ||
                                                item?.displayName
                                                : item?.displayName}
                                        </div>
                                    )}
                                </div>
                                <div className="bg-slate-800 px-4 py-2 border-slate-700">
                                    <p className="text-white font-medium  text-center text-sm">
                                        {tSlot(`slot.${slotName}`)}
                                    </p>
                                </div>
                            </LocaleLink>
                            {item && ['PrimaryWeapon', 'SecondaryWeapon'].includes(slotName) && (
                                <ItemInfoModal
                                    item={item}
                                    items={items}
                                    instanceItems={entries}
                                    itemsLangs={itemsLangs}
                                    locale={locale}
                                    slots={entry?.slotContents || []}
                                />
                            )}
                        </div>
                    );
                })}
            </div>
            <Inventorys
                title={t('inventory.title')}
                isCharacter
                locale={locale}
                itemsLangs={itemsLangs}
                inventory={
                    {
                        capacity: itemData.entries.length,
                        entries: itemData.entries.filter(item => !ids.includes(item.typeID)),
                    } as unknown as Inventory
                }
                items={items}
            />
        </div>
    );
};

const Inventorys: React.FC<InventorysProps> = ({
    inventory,
    title,
    icon,
    items,
    itemsLangs,
    locale,
    isCharacter,
}) => {
    const [sortBy, setSortBy] = useState<SORT_ORDER>(SORT_ORDER.DEFAULT);

    const t = useTranslations();

    if (!inventory.entries || inventory.entries.length <= 0) {
        return (
            <p className="text-gray-500 dark:text-gray-400">
                {t('archived.inventory.empty')}
            </p>
        );
    }

    function getItemById(typeId: number): Item | undefined {
        return items.find((item) => item.id === typeId);
    }

    const allItems: ItemInventory[] = [];
    let total: number = 0;

    const mapFn = (itemEntry: ItemEntry) => {
        const item = getItemById(itemEntry.typeID);
        const counts = itemEntry.variables
            .filter((item) => item.key === 'Count')
            .reduce((acc, item) => {
                acc += parseVariable(item) as number;
                return acc;
            }, 0);
        total += Number(item?.priceEach) * (counts || 1) || 0;

        allItems.push({
            item,
            count: counts,
        });
    };

    if (isCharacter) {
        (inventory.entries as unknown as ItemEntry[]).forEach(mapFn);
    } else {
        inventory.entries.forEach((entry) => {
            return entry.itemTreeData.entries.forEach(mapFn);
        });
    }

    // 排序函数
    const getSortedEntries = (allItems: ItemInventory[]) => {
        switch (sortBy) {
            case SORT_ORDER.RARITY:
                return allItems.sort((a, b) => {
                    const aQuality = a.item?.quality || 0;
                    const bQuality = b.item?.quality || 0;
                    if (aQuality > 6 || bQuality > 6) {
                        return -1;
                    }
                    return bQuality - aQuality;
                });
            case SORT_ORDER.VALUE:
                return allItems.sort((a, b) => {
                    const aValue =
                        (Number(a.item?.priceEach) || 0) * (a.count || 1);
                    const bValue =
                        (Number(b.item?.priceEach) || 0) * (b.count || 1);

                    return bValue - aValue;
                });
            default:
                return allItems;
        }
    };

    return (
        <div className="bg-white rounded-lg shadow overflow-hidden">
            <div className="bg-gray-800 px-6 py-4 flex items-center justify-between">
                <h2 className="text-xl font-semibold text-white">
                    <div className="flex">
                        {icon && icon}
                        {title}
                    </div>
                </h2>
                <div className="flex items-center gap-4">
                    <div className="text-sm text-gray-300">
                        {t('archived.inventory.capacity')}: {inventory.capacity} | {t('archived.inventory.items_count')}:{' '}
                        {inventory.entries?.length || 0} | {t('archived.inventory.total_value')}: {total}
                    </div>
                    {/* 排序选择器 */}
                    <div className="flex items-center gap-2">
                        <span className="text-sm text-gray-400">{t('archived.inventory.sort_by')}:</span>
                        <select
                            value={sortBy}
                            onChange={(e) =>
                                setSortBy(e.target.value as SORT_ORDER)
                            }
                            className="bg-gray-700 text-white text-sm px-3 py-1 rounded border border-gray-600 focus:outline-none focus:border-blue-500"
                        >
                            <option value="default">{t('archived.inventory.sort.default')}</option>
                            <option value="value">{t('archived.inventory.sort.value')}</option>
                            <option value="rarity">{t('archived.inventory.sort.rarity')}</option>
                        </select>
                    </div>
                </div>
            </div>
            <div className="px-4 py-4">
                <div className="max-h-96 overflow-y-auto pr-1">
                    <div className="grid grid-cols-6 gap-2">
                        {getSortedEntries(allItems).map((itemData, idx) => {
                            if (!itemData.item) return null;
                            const item = itemData.item;
                            return (
                                <ItemLink
                                    qualityBorder
                                    itemsLangs={itemsLangs}
                                    key={idx}
                                    item={item}
                                    locale={locale}
                                    extra={
                                        <p className="text-xs text-gray-500 dark:text-gray-400">
                                            {item.priceEach || 0}{' '}
                                            {itemData.count > 0 &&
                                                `x${itemData.count}`}
                                        </p>
                                    }
                                />
                            );
                        })}
                    </div>
                </div>
            </div>
        </div>
    );
};

export const ArchiveUpload: React.FC<ArchiveUploadProps> = ({
    items,
    locale,
    monstersLangs,
    monsters,
    itemsLangs,
    questData,
}) => {
    const t = useTranslations('archived');
    const [archive, setArchive] = useState<Archive | null>(() => {
        const storageArchived = typeof window==='undefined'? null : localStorage.getItem('archived') || null;
        return storageArchived ? JSON.parse(storageArchived) || null : null;
    });
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isDragging, setIsDragging] = useState(false);


    const fileInputRef = useRef<HTMLInputElement>(null);
    const nodes = questData?.nodes || [];
    async function processTextFile(file: File) {
        if (!file.name.endsWith('.sav')) {
            setError(t('upload.error_format'));
            return;
        }

        setLoading(true);
        setError(null);
        setArchive(null);

        try {
            const arrayBuffer = await file.arrayBuffer();
            const uint8Array = new Uint8Array(arrayBuffer);

            const wasm = await import('@/savefile-parse/pkg/');
            await wasm.default();

            const resultJson = wasm.process_file(uint8Array);
            localStorage.setItem('archived', resultJson);// 暂时保存
            const parsedArchive: Archive = JSON.parse(resultJson);

            setArchive(parsedArchive);
        } catch (error) {
            setError(
                error instanceof Error ? error.message : t('upload.error_parse')
            );
        } finally {
            setLoading(false);
        }
    }

    function handleFileChange(e: React.ChangeEvent<HTMLInputElement>) {
        const file = e.target.files?.[0];
        if (file) {
            void processTextFile(file);
        }
    }

    function handleDragOver(e: React.DragEvent) {
        e.preventDefault();
        setIsDragging(true);
    }

    function handleDragLeave(e: React.DragEvent) {
        e.preventDefault();
        setIsDragging(false);
    }

    function handleDrop(e: React.DragEvent) {
        e.preventDefault();
        setIsDragging(false);

        const file = e.dataTransfer.files?.[0];
        if (file) {
            void processTextFile(file);
        }
    }

    return (
        <>
            <div className="text-center mb-8">
                <h1 className="text-3xl font-bold text-gray-900">
                    {t('title')}
                </h1>
                <p className="mt-2 text-gray-600">{t('subtitle')}</p>
                <p className="mt-2 text-gray-400">{t('game_version')}</p>
            </div>

            {/* 文件上传区域 */}
            <div
                className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
                    isDragging
                        ? 'border-blue-500 bg-blue-50'
                        : 'border-gray-300 hover:border-gray-400'
                }`}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                onDrop={handleDrop}
            >
                <input
                    ref={fileInputRef}
                    type="file"
                    accept=".sav"
                    onChange={handleFileChange}
                    className="hidden"
                />

                <div className="space-y-4">
                    <svg
                        className="mx-auto h-12 w-12 text-gray-400"
                        stroke="currentColor"
                        fill="none"
                        viewBox="0 0 48 48"
                    >
                        <path
                            d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02"
                            strokeWidth={2}
                            strokeLinecap="round"
                            strokeLinejoin="round"
                        />
                    </svg>

                    <div>
                        <button
                            onClick={() => fileInputRef.current?.click()}
                            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                        >
                            {t('upload.button')}
                        </button>
                        <p className="mt-2 text-sm text-gray-500">
                            {t('upload.drag_drop')}
                        </p>
                        <p className="mt-1 text-xs text-gray-400">
                            {t('upload.format')}
                        </p>
                    </div>
                </div>
            </div>

            {/* 加载状态 */}
            {loading && (
                <div className="mt-8 bg-white rounded-lg shadow p-6 text-center">
                    <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    <p className="mt-2 text-gray-600">
                        {t('upload.loading')}
                    </p>
                </div>
            )}

            {/* 错误信息 */}
            {error && (
                <div className="mt-8 bg-red-50 border border-red-200 rounded-lg p-4">
                    <div className="flex">
                        <div className="flex-shrink-0">
                            <svg
                                className="h-5 w-5 text-red-400"
                                viewBox="0 0 20 20"
                                fill="currentColor"
                            >
                                <path
                                    fillRule="evenodd"
                                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                                    clipRule="evenodd"
                                />
                            </svg>
                        </div>
                        <div className="ml-3">
                            <p className="text-sm text-red-800">{error}</p>
                        </div>
                    </div>
                </div>
            )}

            {/* 解析结果 */}
            {archive && (
                <div className="mt-8 space-y-6">
                    {/* 基本信息 */}
                    <div className="bg-white rounded-lg shadow overflow-hidden">
                        <div className="bg-gray-800 px-6 py-4">
                            <h2 className="text-xl font-semibold text-white">
                                {t('basic_info.title')}
                            </h2>
                        </div>
                        <div className="px-6 py-4 grid grid-cols-2 gap-4">
                            <div>
                                <p className="text-sm text-gray-500">
                                    {t('basic_info.version')}
                                </p>
                                <p className="mt-1 text-lg font-medium text-gray-900">
                                    {archive.version}
                                </p>
                            </div>
                            <div>
                                <p className="text-sm text-gray-500">
                                    {t('basic_info.level')}
                                </p>
                                <p className="mt-1 text-lg font-medium text-gray-900">
                                    {archive.level}
                                </p>
                            </div>
                            <div>
                                <p className="text-sm text-gray-500">
                                    {t('basic_info.exp')}
                                </p>
                                <p className="mt-1 text-lg font-medium text-gray-900">
                                    {archive.exp}
                                </p>
                            </div>
                            <div>
                                <p className="text-sm text-gray-500">
                                    {t('basic_info.money')}
                                </p>
                                <p className="mt-1 text-lg font-medium text-gray-900">
                                    {archive.economyData?.money}
                                </p>
                            </div>
                            <div>
                                <p className="text-sm text-gray-500">
                                    {t('basic_info.days')}
                                </p>
                                <p className="mt-1 text-lg font-medium text-gray-900">
                                    {archive.gameClock.days}
                                </p>
                            </div>
                            <div>
                                <p className="text-sm text-gray-500">{t('basic_info.hp')}</p>
                                <p className="mt-1 text-lg font-medium text-gray-900">
                                    {archive.health || '-'}
                                </p>
                            </div>
                        </div>
                    </div>


                    {/* 个人宝宝 */}
                    {archive.characterItemData.entries && (
                        <CharacterPanel
                            items={items}
                            locale={locale}
                            itemsLangs={itemsLangs}
                            itemData={archive.characterItemData}
                        />
                    )}

                    {/* 主钥匙 */}
                    {archive.masterKeys &&
                        archive.masterKeys.length > 0 && (
                            <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden border border-gray-200 dark:border-gray-700">
                                <div className="bg-gray-800 px-6 py-4">
                                    <h2 className="text-xl font-semibold text-white">
                                        {t('master_keys.activated')}
                                        {t('master_keys.title')} (
                                        {archive.masterKeys.length})
                                    </h2>
                                </div>
                                <div className="px-6 py-4">
                                    <div className="max-h-64 overflow-y-auto pr-1">
                                        <div className="grid grid-cols-4 gap-2">
                                            {archive.masterKeys.map(
                                                (key, idx) => {
                                                    // Find master key item from items.json (usually items with "MasterKey" tag)
                                                    const keyItem =
                                                        items.find(
                                                            (item) =>
                                                                item.tags.includes(
                                                                    'Key'
                                                                ) &&
                                                                item.id ===
                                                                key.id &&
                                                                key.active
                                                        );

                                                    return (
                                                        keyItem && (
                                                            <ItemLink
                                                                qualityBorder
                                                                itemsLangs={
                                                                    itemsLangs
                                                                }
                                                                key={idx}
                                                                item={
                                                                    keyItem
                                                                }
                                                                locale={
                                                                    locale
                                                                }
                                                            />
                                                        )
                                                    );
                                                }
                                            )}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )}

                    {/* 任务数据 */}
                    {archive.quests && (
                        <div className="bg-white rounded-lg shadow overflow-hidden">
                            <div className="bg-gray-800 px-6 py-4">
                                <h2 className="text-xl font-semibold text-white">
                                    {t('quests.title')}
                                </h2>
                            </div>
                            <div className="px-6 py-4 grid grid-cols-1 sm:grid-cols-2 gap-4">
                                <div>
                                    <p className="text-sm text-gray-500">
                                        {t('quests.active')}
                                    </p>
                                    <p className="mt-1 text-2xl font-semibold text-blue-600">
                                        {archive.quests.activeQuestsData
                                            ?.length || 0}
                                    </p>
                                </div>
                                <div>
                                    <p className="text-sm text-gray-500">
                                        {t('quests.completed')}
                                    </p>
                                    <p className="mt-1 text-2xl font-semibold text-green-600">
                                        {archive.quests.historyQuestsData
                                            ?.length || 0}
                                    </p>
                                </div>
                            </div>
                            <div className="max-h-120 space-y-4 overflow-y-auto p-4">
                                {archive.quests.activeQuestsData?.map((quest,) => {
                                    const que = nodes.find(item => item.questID === quest.id);
                                    return que && (
                                        <div
                                            key={quest.id} // 使用 quest.id 而不是 idx
                                            className="p-4 rounded-lg border border-gray-200 hover:border-gray-300 hover:shadow-sm transition-all"
                                        >
                                            <div className="flex items-center gap-3 mb-2">
                                                {/*<span className="text-xs font-semibold text-gray-600 bg-gray-100 px-2 py-1 rounded">*/}
                                                {/*    #{quest.id}*/}
                                                {/*</span>*/}
                                                <h3 className="font-medium text-gray-900">
                                                    {que.name}
                                                </h3>
                                            </div>
                                            <p className="text-sm text-gray-500 leading-relaxed">
                                                {que.comment}
                                            </p>
                                        </div>
                                    )
                                })}
                            </div>
                        </div>
                    )}

                    {/* 击杀统计 */}
                    {archive.kills && archive.kills.length > 0 && (
                        <div className="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden border border-gray-200 dark:border-gray-700">
                            <div className="bg-gray-800 px-6 py-4 dark:pb-0">
                                <h2 className="text-xl font-semibold  text-white">
                                    {t('kills.title')} ({archive.kills.length})
                                </h2>
                            </div>
                            <div className="px-6 py-4">
                                <div className="max-h-96 overflow-y-auto pr-1">
                                    <div className="grid grid-cols-4 gap-2">
                                        {archive.kills
                                            .sort(
                                                (a, b) => b.count - a.count
                                            ) // Sort by count descending
                                            .map((kill, index) => {
                                                // Try to find monster.ts info
                                                const monsterInfo =
                                                    getMonsterName(
                                                        monsters,
                                                        kill.masterName
                                                    );
                                                if (!monsterInfo) {
                                                    return <></>;
                                                }
                                                const i18Name =
                                                    monstersLangs?.[
                                                        monsterInfo?.nameKey
                                                        ] ||
                                                    monsterInfo?.nameKey;
                                                return (
                                                    <div
                                                        key={index}
                                                        className="flex items-center gap-2 p-2 bg-gray-50 dark:bg-gray-700 rounded border border-gray-200 dark:border-gray-600 h-14 hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
                                                    >
                                                        {/* Placeholder for monster.ts icon - you can add actual icons if available */}
                                                        <div className=" w-8 h-8 flex-shrink-0 bg-red-100 dark:bg-red-900/20 rounded flex items-center justify-center">
                                                                <span className="text-red-600 dark:text-red-400 text-xs font-bold">
                                                                    ☠️
                                                                </span>
                                                        </div>
                                                        <div className="flex-1 min-w-0">
                                                            <p className="text-xs font-medium text-gray-900 dark:text-gray-100 truncate">
                                                                {i18Name}
                                                            </p>
                                                            <p className="text-xs text-red-600 dark:text-red-400">
                                                                {t('kills.count')}:{' '}
                                                                {kill.count}
                                                            </p>
                                                        </div>
                                                    </div>
                                                );
                                            })}
                                    </div>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* 狗背包 */}
                    {archive.inventorySafe && (
                        <Inventorys
                            title={t('dog_inventory.title')}
                            icon={<DogSvg />}
                            locale={locale}
                            itemsLangs={itemsLangs}
                            inventory={archive.inventorySafe}
                            items={items}
                        />
                    )}

                    {/* 仓库信息 */}
                    {archive.characterItemData && (
                        <Inventorys
                            title={t('storage.title')}
                            locale={locale}
                            itemsLangs={itemsLangs}
                            inventory={archive.playerStorage}
                            items={items}
                        />
                    )}

                    {/* 死亡列表 */}
                    {archive.deathList && archive.deathList.length > 0 && (
                        <div className="bg-white rounded-lg shadow overflow-hidden">
                            <div className="bg-gray-800 px-6 py-4">
                                <h2 className="text-xl font-semibold text-white">
                                    {t('death_list.title')}
                                </h2>
                            </div>
                            <div className="px-6 py-4">
                                <p className="text-sm text-gray-600">
                                    {t('death_list.count')}:{' '}
                                    <span className="font-semibold text-red-600">
                                            {archive.deathList.length}
                                        </span>
                                </p>
                            </div>
                        </div>
                    )}
                </div>
            )}
        </>
    );
};

export default ArchiveUpload;
