import Image from 'next/image';
import { Item, PrefabItem } from '@/app/types/item';
import { MonsterData } from '@/app/types/monster';
import { fetchAllByFile } from '@/app/utils/request';
import { Metadata } from 'next';
import { getTranslations, setRequestLocale } from 'next-intl/server';
import { getQualityConfig } from '@/app/constants/quality';
import { findMonsterDropSources } from '@/app/utils/monster';
import { PageParamsProps } from '@/app/types/router';
import { GoBack } from '@/app/components/ClientProxy';
import { ItemAttribute, ItemConstant, ItemSlots } from '@/app/components/ItemConstant';
import { getItemKey, getItemName, getMonsterName } from '@/app/utils/lang';

import { languageKeys } from '@/app/i18n/config';


// export async function generateMetadata(
//     props: PageProps<'/[locale]/inventory/[id]'> & PageParamsProps
// ): Promise<Metadata> {
//     const { id, locale } = await props.params;
//     setRequestLocale(locale);
//     const t = await getTranslations({ locale });
//     const items = fetchAllByFile<Item[]>('items.json');
//     const item = items.find((item) => item.id === Number(id));
//
//     if (!item) {
//         return {
//             title: t('seo.item_not_found_title'),
//             description: t('seo.item_not_found_description'),
//         };
//     }
//
//     const itemName = getItemName(t, item);
//     const itemDescription = getItemKey(t, item, 'description');
//     const itemTags = item.tags.map((tag) => t(`tags.Tag_${tag}`) || tag);
//     const description = t('seo.item_detail_description', {
//         name: itemName,
//         description: itemDescription,
//         price: item.priceEach.toString(),
//         stack: item.maxStackCount.toString(),
//     });
//
//     const baseKeywords = t('seo.item_detail_keywords').split(',');
//
//     return {
//         title: itemName,
//         description,
//         keywords: [itemName, ...itemTags, ...baseKeywords],
//         openGraph: {
//             title: `${itemName} | ${t('seo.site_name')}`,
//             description: itemDescription,
//             type: 'article',
//             images: [
//                 {
//                     url: `/images/${item.icon}`,
//                     width: 256,
//                     height: 256,
//                     alt: itemName,
//                 },
//             ],
//         },
//     };
// }

export default async function ItemDetailPage(
    props: PageProps<'/[locale]/inventory/[id]'> & PageParamsProps
) {
    const { id, locale } = await props.params;
    setRequestLocale(locale);
    const t = await getTranslations({ locale });

    const itemPrefab = fetchAllByFile<PrefabItem>(`prefabs/${id}.prefab`);

    const item = itemPrefab?.base;

    if (!item) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
                <main className="max-w-4xl mx-auto">
                    <div className="mb-6">
                        <GoBack />
                    </div>
                    <div className="bg-white dark:bg-gray-800 rounded-lg p-8 text-center">
                        <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                            {t('inventory.item_not_found')}
                        </h1>
                        <p className="text-gray-600 dark:text-gray-400 mt-2">
                            {t('inventory.item_not_exist')}
                        </p>
                    </div>
                </main>
            </div>
        );
    }

    // const cnTag = item.tags.map((tag) => tags?.[`Tag_${tag}`] || tag);
    const qualityConfig = getQualityConfig(item.quality);
    const qualityName = t(qualityConfig.nameKey);

    // Find monster.ts drop sources
    const lootData = fetchAllByFile<MonsterData>('loot.json');
    const isGunPrefab = !item.tags.includes("DestroyOnLootBox") && item.tags.includes("Gun");
    const gunItem = fetchAllByFile<PrefabItem>(`prefabs/${id}.prefab`);

    const monsterDrops = findMonsterDropSources(item.typeID, lootData);
    // const monsterLangs = fetchCharacter(locale as Language);

    const slots = gunItem?.slots?.list || []
    const attrs = gunItem?.attributes?.list || []

    const descriptionKey = `${item.displayName}_Desc`;

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
            <main className="max-w-4xl mx-auto">
                {/* Back Button */}
                <div className="mb-6">
                    <GoBack />
                </div>

                {/* Item Detail Card */}
                <div className={`bg-white dark:bg-gray-800 rounded-lg shadow-lg overflow-hidden border-2 ${qualityConfig.borderColor}`}>
                    {/* Header Section */}
                    <div className="bg-gradient-to-r from-blue-500 to-purple-600 p-6">
                        <div className="flex items-center gap-6">
                            <div className="relative w-24 h-24 bg-white dark:bg-gray-700 rounded-lg p-2 flex-shrink-0">
                                <Image
                                    src={`/images/${item.icon}`}
                                    alt={item.displayName}
                                    fill
                                    className="object-contain"
                                />
                            </div>
                            <div className="flex-1">
                                <h1 className="text-3xl font-bold text-white mb-2">
                                    {getItemKey(t, item, 'displayName')}
                                </h1>
                                <span className={`inline-block px-3 py-1 rounded-full text-sm font-semibold ${qualityConfig.bgColor} ${qualityConfig.textColor}`}>
                                    {qualityName}
                                </span>
                                <p className="text-blue-100 mt-2">ID: {item.typeID}</p>
                            </div>
                        </div>
                    </div>

                    {/* Content Section */}
                    <div className="p-6 space-y-6">
                        {/* Description */}
                        <div>
                            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
                                {t('inventory.description')}
                            </h2>
                            <p className="text-gray-700 dark:text-gray-300">

                                {getItemKey(t, item as unknown as Item, descriptionKey as keyof Item)}
                            </p>
                        </div>

                        {/* Basic Info */}
                        <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                                <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">
                                    {t('inventory.price')}
                                </p>
                                <p className="text-xl font-bold text-gray-900 dark:text-gray-100">
                                    {item.value}
                                </p>
                            </div>
                            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                                <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">
                                    {t('inventory.max_stack')}
                                </p>
                                <p className="text-xl font-bold text-gray-900 dark:text-gray-100">
                                    {item.maxStackCount}
                                </p>
                            </div>

                            {item?.weight && (
                                <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                                    <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">
                                        {t('inventory.weight')}
                                    </p>
                                    <p className="text-xl font-bold text-gray-900 dark:text-gray-100">
                                        {item.weight}Kg
                                    </p>
                                </div>
                            )}
                            {gunItem && (
                                <ItemConstant {...gunItem} locale={locale} />
                            )}
                        </div>
                        {isGunPrefab && slots.length > 0 && (
                            <ItemSlots locale={locale} slots={gunItem?.slots?.list?.map(item => item.key) || []} />
                        )}
                        {/* Tags */}
                        {item.tags.length > 0 && (
                            <div>
                                <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
                                    {t('inventory.tags')}
                                </h2>
                                <div className="flex flex-wrap gap-2">
                                    {item.tags.map((tag) => (
                                        <span
                                            key={tag}
                                            className="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm font-medium"
                                        >
                                            {t(`tags.Tag_${tag}`)}
                                        </span>
                                    ))}
                                </div>
                            </div>
                        )}

                        {/* Monster Drops */}
                        <div>
                            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
                                {t('inventory.monster_drops')}
                            </h2>
                            {monsterDrops.length > 0 ? (
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                                    {monsterDrops.map((drop, idx) => {
                                        const monsterName = getMonsterName(t, drop.monster);
                                        return (
                                            <div
                                                key={idx}
                                                className="bg-gray-50 dark:bg-gray-700 rounded-lg p-3 border border-gray-200 dark:border-gray-600"
                                            >
                                                <div className="flex justify-between items-start mb-2">
                                                    <h3 className="font-semibold text-gray-900 dark:text-gray-100 text-sm">
                                                        {monsterName}
                                                    </h3>
                                                    <span className="text-xs bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200 px-2 py-0.5 rounded">
                                                        {(drop.chance * 100).toFixed(0)}%
                                                    </span>
                                                </div>
                                                {drop.comment && (
                                                    <p className="text-xs text-gray-500 dark:text-gray-400 mb-2">
                                                        {drop.comment}
                                                    </p>
                                                )}
                                                <div className="flex items-center gap-2 text-xs text-gray-600 dark:text-gray-300">
                                                    <span>HP: {drop.monster.health}</span>
                                                    <span>•</span>
                                                    <span>EXP: {drop.monster.exp}</span>
                                                </div>
                                            </div>
                                        );
                                    })}
                                </div>
                            ) : (
                                <p className="text-gray-500 dark:text-gray-400 text-sm">
                                    {t('inventory.no_monster_drops')}
                                </p>
                            )}
                        </div>

                        {attrs.length > 0 && (
                            <ItemAttribute locale={locale} attrs={attrs} />
                        )}
                    </div>
                </div>
            </main>
        </div>
    );
}

// export const dynamic = 'force-static';
export const dynamicParams = true;

//
// export function generateStaticParams() {
//     const items = fetchAllByFile<Item[]>('items.json');
//
//     return languageKeys.flatMap((locale) =>
//         items.map((item) => ({
//             locale,
//             id: item.id.toString(),
//         }))
//     );
// }

export async function generateStaticParams() {
    return [];
}