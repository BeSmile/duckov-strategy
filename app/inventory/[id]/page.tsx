import Image from 'next/image';
import Link from 'next/link';
import { Item } from '@/app/types/item';
import { fetchAllByFile, generateKeyValueFetch } from '@/app/utils/request';
import { ITEM_KEY, TAG_KEY } from '@/app/constants';
import { getLocale } from '@/app/actions/cookies';
import { getServerTranslation } from '@/app/i18n/server';

const fetchTags = generateKeyValueFetch(TAG_KEY);
const fetchItemI18 = generateKeyValueFetch(ITEM_KEY);

export default async function ItemDetailPage(
    props: PageProps<'/inventory/[id]'>
) {
    const { id } = await props.params;
    const { t } = await getServerTranslation();

    const items = fetchAllByFile<Item[]>('items.json');
    const lang = await getLocale();
    const langs = fetchItemI18(lang);
    const tags = fetchTags(lang);

    const item = items.find((item) => item.id === Number(id));

    if (!item) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
                <main className="max-w-4xl mx-auto">
                    <div className="mb-6">
                        <Link
                            href="/inventory"
                            className="text-blue-500 hover:text-blue-600 dark:text-blue-400 dark:hover:text-blue-300"
                        >
                            ← {await t('inventory.back_to_inventory')}
                        </Link>
                    </div>
                    <div className="bg-white dark:bg-gray-800 rounded-lg p-8 text-center">
                        <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                            {await t('inventory.item_not_found')}
                        </h1>
                        <p className="text-gray-600 dark:text-gray-400 mt-2">
                            {await t('inventory.item_not_exist')}
                        </p>
                    </div>
                </main>
            </div>
        );
    }

    const cnTag = item.tags.map((tag) => tags?.[`Tag_${tag}`] || tag);

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
            <main className="max-w-4xl mx-auto">
                {/* Back Button */}
                <div className="mb-6">
                    <Link
                        href="/inventory"
                        className="text-blue-500 hover:text-blue-600 dark:text-blue-400 dark:hover:text-blue-300 flex items-center gap-2"
                    >
                        ← {await t('inventory.back_to_inventory')}
                    </Link>
                </div>

                {/* Item Detail Card */}
                <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg overflow-hidden">
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
                                    {langs?.[item.displayName] || item.name}
                                </h1>
                                <p className="text-blue-100">ID: {item.id}</p>
                            </div>
                        </div>
                    </div>

                    {/* Content Section */}
                    <div className="p-6 space-y-6">
                        {/* Description */}
                        <div>
                            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
                                {await t('inventory.description')}
                            </h2>
                            <p className="text-gray-700 dark:text-gray-300">
                                {langs?.[item.description] ||
                                    item.description ||
                                    await t('inventory.no_description')}
                            </p>
                        </div>

                        {/* Basic Info */}
                        <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                                <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">
                                    {await t('inventory.price')}
                                </p>
                                <p className="text-xl font-bold text-gray-900 dark:text-gray-100">
                                    {item.priceEach}
                                </p>
                            </div>
                            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                                <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">
                                    {await t('inventory.max_stack')}
                                </p>
                                <p className="text-xl font-bold text-gray-900 dark:text-gray-100">
                                    {item.maxStackCount}
                                </p>
                            </div>
                            {/*{item.weight !== undefined && (*/}
                            {/*  <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">*/}
                            {/*    <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">*/}
                            {/*      Weight*/}
                            {/*    </p>*/}
                            {/*    <p className="text-xl font-bold text-gray-900 dark:text-gray-100">*/}
                            {/*      {item.weight}*/}
                            {/*    </p>*/}
                            {/*  </div>*/}
                            {/*)}*/}
                        </div>

                        {/* Tags */}
                        {cnTag.length > 0 && (
                            <div>
                                <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
                                    {await t('inventory.tags')}
                                </h2>
                                <div className="flex flex-wrap gap-2">
                                    {cnTag.map((tag, idx) => (
                                        <span
                                            key={idx}
                                            className="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm font-medium"
                                        >
                                            {tag}
                                        </span>
                                    ))}
                                </div>
                            </div>
                        )}

                        {/* Additional Properties */}
                        <div>
                            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
                                {await t('inventory.properties')}
                            </h2>
                            <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 space-y-2">
                                <div className="flex justify-between text-sm">
                                    <span className="text-gray-600 dark:text-gray-400">
                                        {await t('inventory.name')}:
                                    </span>
                                    <span className="text-gray-900 dark:text-gray-100 font-medium">
                                        {item.name}
                                    </span>
                                </div>
                                <div className="flex justify-between text-sm">
                                    <span className="text-gray-600 dark:text-gray-400">
                                        {await t('inventory.display_name')}:
                                    </span>
                                    <span className="text-gray-900 dark:text-gray-100 font-medium">
                                        {item.displayName}
                                    </span>
                                </div>
                                {item.description && (
                                    <div className="flex justify-between text-sm">
                                        <span className="text-gray-600 dark:text-gray-400">
                                            {await t('inventory.description_key')}:
                                        </span>
                                        <span className="text-gray-900 dark:text-gray-100 font-medium">
                                            {item.description}
                                        </span>
                                    </div>
                                )}
                            </div>
                        </div>

                        {/* Raw Data */}
                        <details className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4">
                            <summary className="text-lg font-semibold text-gray-900 dark:text-gray-100 cursor-pointer">
                                {await t('inventory.raw_data')} (JSON)
                            </summary>
                            <pre className="mt-4 text-xs text-gray-700 dark:text-gray-300 overflow-x-auto">
                                {JSON.stringify(item, null, 2)}
                            </pre>
                        </details>
                    </div>
                </div>
            </main>
        </div>
    );
}
