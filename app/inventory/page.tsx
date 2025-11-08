import ItemCard from '@/app/components/ItemCard';
import Pagination from '@/app/components/Pagination';
import SearchFilter from '@/app/components/SearchFilter';
import { Item } from '@/app/types/item';
import { fetchAllByFile, generateKeyValueFetch } from '@/app/utils/request';
import { ITEM_KEY, TAG_KEY } from '@/app/constants';
import { Suspense } from 'react';
import { getLocale } from '@/app/actions/cookies';
import { getServerTranslation } from '@/app/i18n/server';

const ITEMS_PER_PAGE = 20; // 分页

const fetchTags = generateKeyValueFetch(TAG_KEY);
const fetchItemI18 = generateKeyValueFetch(ITEM_KEY);

interface HomeProps {
    searchParams: Promise<{ page?: string; search?: string; tags?: string }>;
}

export default async function Home({ searchParams }: HomeProps) {
    const params = await searchParams;
    const currentPage = Number(params.page) || 1;
    const searchTerm = params.search?.toLowerCase() || '';
    const selectedTags = params.tags?.split(',').filter(Boolean) || [];
    const lang = await getLocale();
    const { t } = await getServerTranslation();

    // Fetch all data
    const allData = fetchAllByFile<Item[]>('items.json');
    const langs = fetchItemI18(lang);
    const tags = fetchTags(lang);

    // Collect all unique tags for filter dropdown
    const allUniqueTags = Array.from(
        new Set(
            allData.flatMap((item) =>
                item.tags.map((tag) => tags?.[`Tag_${tag}`] || tag)
            )
        )
    ).sort();

    // Filter data based on search and tags
    let filteredData = allData;

    // Apply search filter
    if (searchTerm) {
        filteredData = filteredData.filter((item) => {
            const itemId = item.id.toString();
            const itemName = item.name.toLowerCase();
            const itemDisplayName = (
                langs?.[item.displayName] || ''
            ).toLowerCase();

            return (
                itemId.includes(searchTerm) ||
                itemName.includes(searchTerm) ||
                itemDisplayName.includes(searchTerm)
            );
        });
    }

    // Apply tag filter
    if (selectedTags.length > 0) {
        filteredData = filteredData.filter((item) => {
            const itemTags = item.tags.map(
                (tag) => tags?.[`Tag_${tag}`] || tag
            );
            return selectedTags.every((selectedTag) =>
                itemTags.includes(selectedTag)
            );
        });
    }

    // Calculate pagination on filtered data
    const totalItems = filteredData.length;
    const totalPages = Math.ceil(totalItems / ITEMS_PER_PAGE);

    // Ensure current page is within valid range
    const validPage = Math.max(1, Math.min(currentPage, totalPages || 1));

    // Get items for current page
    const startIndex = (validPage - 1) * ITEMS_PER_PAGE;
    const endIndex = startIndex + ITEMS_PER_PAGE;
    const paginatedData = filteredData.slice(startIndex, endIndex);

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
            <main className="max-w-7xl mx-auto">
                <div className="mb-8">
                    <h1 className="text-4xl font-bold text-gray-900 dark:text-gray-100 mb-2">
                        {await t('inventory.title')}
                    </h1>
                    <p className="text-gray-600 dark:text-gray-400">
                        {await t('common.total')}: {allData.length} {await t('inventory.items')}
                        {totalItems !== allData.length &&
                            ` | ${await t('common.filtered')}: ${totalItems} ${await t('inventory.items')}`}
                        {totalItems > 0 &&
                            ` | ${await t('common.page')} ${validPage} ${await t('common.of')} ${totalPages}`}
                    </p>
                </div>

                {/* Search and Filter */}
                <Suspense
                    fallback={
                        <div className="mb-6 h-12 bg-gray-200 dark:bg-gray-700 rounded animate-pulse" />
                    }
                >
                    <SearchFilter allTags={allUniqueTags} />
                </Suspense>

                {/* Results */}
                {paginatedData.length > 0 ? (
                    <>
                        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
                            {paginatedData.map((item: Item) => (
                                <ItemCard
                                    tags={tags}
                                    langs={langs}
                                    item={item}
                                    key={item.id}
                                />
                            ))}
                        </div>

                        {/* Pagination */}
                        {totalPages > 1 && (
                            <Suspense
                                fallback={
                                    <div className="mt-8 h-12 bg-gray-200 dark:bg-gray-700 rounded animate-pulse" />
                                }
                            >
                                <Pagination
                                    currentPage={validPage}
                                    totalPages={totalPages}
                                    baseUrl="/inventory"
                                />
                            </Suspense>
                        )}
                    </>
                ) : (
                    <div className="text-center py-12">
                        <p className="text-xl text-gray-500 dark:text-gray-400">
                            {await t('inventory.no_results')}
                        </p>
                        <p className="text-sm text-gray-400 dark:text-gray-500 mt-2">
                            {await t('inventory.try_adjust_filters')}
                        </p>
                    </div>
                )}
            </main>
        </div>
    );
}
