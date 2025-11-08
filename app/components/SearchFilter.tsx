'use client';

import { useRouter, useSearchParams } from 'next/navigation';
import {
    useState,
    MouseEvent,
    useLayoutEffect,
    useCallback,
    useRef,
} from 'react';
import { useTranslation } from 'react-i18next';

interface SearchFilterProps {
    allTags: string[];
}

export default function SearchFilter({ allTags }: SearchFilterProps) {
    const router = useRouter();
    const searchParams = useSearchParams();
    const filterRef = useRef<HTMLInputElement>(null);
    const { t } = useTranslation();

    const [searchTerm, setSearchTerm] = useState(
        searchParams.get('search') || ''
    );
    const [selectedTags, setSelectedTags] = useState<string[]>(
        searchParams.get('tags')?.split(',').filter(Boolean) || []
    );
    const [isTagsOpen, setIsTagsOpen] = useState(false);

    // Update URL when filters change
    const updateFilters = (search: string, tags: string[]) => {
        const params = new URLSearchParams();

        if (search) {
            params.set('search', search);
        }

        if (tags.length > 0) {
            params.set('tags', tags.join(','));
        }

        // Reset to page 1 when filtering
        params.set('page', '1');

        const queryString = params.toString();
        router.push(`/inventory${queryString ? `?${queryString}` : ''}`);
    };

    const handleSearchChange = (value: string) => {
        setSearchTerm(value);
    };

    const handleSearchSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        updateFilters(searchTerm, selectedTags);
    };

    const handleTagToggle = (tag: string) => {
        const newTags = selectedTags.includes(tag)
            ? selectedTags.filter((t) => t !== tag)
            : [...selectedTags, tag];

        setSelectedTags(newTags);
        updateFilters(searchTerm, newTags);
    };

    const handleClearFilters = () => {
        setSearchTerm('');
        setSelectedTags([]);
        router.push('/inventory');
    };

    const hasActiveFilters = searchTerm || selectedTags.length > 0;

    const handlePageClick = useCallback((e: MouseEvent) => {
        const target = e.target as HTMLElement;

        if (filterRef.current && !filterRef.current.contains(target)) {
            setIsTagsOpen(false);
        }
    }, []) as unknown as EventListener;

    useLayoutEffect(() => {
        document.addEventListener('click', handlePageClick);

        return () => {
            document.removeEventListener('click', handlePageClick);
        };
    }, [handlePageClick]);

    return (
        <div className="mb-6 space-y-4">
            <div className="flex flex-col sm:flex-row gap-4">
                {/* Search Box */}
                <form onSubmit={handleSearchSubmit} className="flex-1">
                    <div className="relative">
                        <input
                            type="text"
                            value={searchTerm}
                            onChange={(e) => handleSearchChange(e.target.value)}
                            placeholder={t('inventory.search_placeholder')}
                            className="w-full px-4 py-2 pr-20 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        />
                        <button
                            type="submit"
                            className="absolute right-2 top-1/2 -translate-y-1/2 px-3 py-1 bg-blue-500 hover:bg-blue-600 text-white text-sm rounded transition-colors"
                        >
                            {t('inventory.search')}
                        </button>
                    </div>
                </form>

                {/* Tag Filter Dropdown */}
                <div className="relative">
                    <button
                        onClick={() => setIsTagsOpen(!isTagsOpen)}
                        className="w-full sm:w-auto px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors flex items-center justify-between gap-2"
                    >
                        <span>
                            {t('inventory.tags')}{' '}
                            {selectedTags.length > 0 &&
                                `(${selectedTags.length})`}
                        </span>
                        <svg
                            className={`w-4 h-4 transition-transform ${isTagsOpen ? 'rotate-180' : ''}`}
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M19 9l-7 7-7-7"
                            />
                        </svg>
                    </button>

                    {isTagsOpen && (
                        <div className="absolute z-10 mt-2 w-full sm:w-80 max-h-96 overflow-y-auto bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg shadow-lg">
                            <div className="p-2 space-y-1">
                                {allTags.map((tag) => (
                                    <label
                                        key={tag}
                                        className="flex items-center gap-2 px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded cursor-pointer"
                                    >
                                        <input
                                            type="checkbox"
                                            checked={selectedTags.includes(tag)}
                                            onChange={() =>
                                                handleTagToggle(tag)
                                            }
                                            className="w-4 h-4 text-blue-500 border-gray-300 rounded focus:ring-blue-500"
                                        />
                                        <span className="text-sm text-gray-900 dark:text-gray-100">
                                            {tag}
                                        </span>
                                    </label>
                                ))}
                            </div>
                        </div>
                    )}
                </div>

                {/* Clear Filters Button */}
                {hasActiveFilters && (
                    <button
                        onClick={handleClearFilters}
                        className="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                    >
                        {t('inventory.clear_filters')}
                    </button>
                )}
            </div>

            {/* Active Filters Display */}
            {hasActiveFilters && (
                <div className="flex flex-wrap gap-2 items-center">
                    <span className="text-sm text-gray-600 dark:text-gray-400">
                        {t('inventory.active_filters')}:
                    </span>
                    {searchTerm && (
                        <span className="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm">
                            {t('inventory.search')}: {searchTerm}
                        </span>
                    )}
                    {selectedTags.map((tag) => (
                        <span
                            key={tag}
                            className="px-3 py-1 bg-green-100 dark:bg-green-900 text-green-800 dark:text-green-200 rounded-full text-sm flex items-center gap-1"
                        >
                            {tag}
                            <button
                                onClick={() => handleTagToggle(tag)}
                                className="hover:text-green-600 dark:hover:text-green-400"
                            >
                                ×
                            </button>
                        </span>
                    ))}
                </div>
            )}
        </div>
    );
}
