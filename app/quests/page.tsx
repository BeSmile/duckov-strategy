'use client';

import { useState, useEffect } from 'react';
import QuestList from '../components/QuestList';
import { IQuestGraph } from '../types/quest';
import QuestGraph from '../components/QuesGraph';
import { useTranslation } from 'react-i18next';

export default function QuestsPage() {
    const [questData, setQuestData] = useState<IQuestGraph | null>(null);
    const [searchTerm, setSearchTerm] = useState('');
    const [viewMode, setViewMode] = useState<'list' | 'graph'>('list');
    const [loading, setLoading] = useState(true);
    const { t } = useTranslation();

    useEffect(() => {
        // Fetch quest data
        fetch('/quest.json')
            .then((res) => res.json())
            .then((data) => {
                setQuestData(data);
                setLoading(false);
            })
            .catch((err) => {
                console.error('Failed to load quest data:', err);
                setLoading(false);
            });
    }, []);

    if (loading) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
                <main className="max-w-7xl mx-auto">
                    <div className="flex items-center justify-center py-20">
                        <div className="text-xl text-gray-500 dark:text-gray-400">
                            {t('quests.loading')}
                        </div>
                    </div>
                </main>
            </div>
        );
    }

    if (!questData) {
        return (
            <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
                <main className="max-w-7xl mx-auto">
                    <div className="flex items-center justify-center py-20">
                        <div className="text-xl text-red-500">
                            {t('quests.failed_load')}
                        </div>
                    </div>
                </main>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
            <main className="max-w-7xl mx-auto">
                {/* Header */}
                <div className="mb-8">
                    <h1 className="text-4xl font-bold text-gray-900 dark:text-gray-100 mb-2">
                        {t('quests.title')}
                    </h1>
                    <p className="text-gray-600 dark:text-gray-400">
                        {t('quests.total_quests')}: {questData.nodes.length}
                    </p>
                </div>

                {/* Controls */}
                <div className="mb-6 flex flex-col sm:flex-row gap-4">
                    {/* Search */}
                    <div className="flex-1">
                        <input
                            type="text"
                            value={searchTerm}
                            onChange={(e) => setSearchTerm(e.target.value)}
                            placeholder={t('quests.search_placeholder')}
                            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        />
                    </div>

                    {/* View Toggle */}
                    <div className="flex border border-gray-300 dark:border-gray-600 rounded-lg overflow-hidden">
                        <button
                            onClick={() => setViewMode('list')}
                            className={`px-4 py-2 text-sm font-medium transition-colors ${
                                viewMode === 'list'
                                    ? 'bg-blue-500 text-white'
                                    : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700'
                            }`}
                        >
                            {t('quests.list_view')}
                        </button>
                        <button
                            onClick={() => setViewMode('graph')}
                            className={`px-4 py-2 text-sm font-medium transition-colors border-l border-gray-300 dark:border-gray-600 ${
                                viewMode === 'graph'
                                    ? 'bg-blue-500 text-white'
                                    : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700'
                            }`}
                            title={t('quests.graph_view')}
                        >
                            {t('quests.graph_view')}
                        </button>
                    </div>
                </div>

                {/* Content */}
                {viewMode === 'list' ? (
                    <QuestList
                        quests={questData.nodes}
                        searchTerm={searchTerm}
                    />
                ) : (
                    <div className="text-center py-20 " style={{height: 800}}>
                        <QuestGraph nodes={questData.nodes} edges={questData.connections}/>
                    </div>
                )}
            </main>
        </div>
    );
}
