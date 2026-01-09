import QuestList from '@/app/components/QuestList';
import { IQuestGraph } from '@/app/types/quest';
import QuestGraph from '@/app/components/QuesGraph';
import { fetchAllByFile } from '@/app/utils/request';
import QuestControls from '@/app/components/QuestControls';
import { Suspense } from 'react';
import { Metadata } from 'next';
import { generateStaticParams } from '@/lib/getStatic';
import { QuestType } from '@/app/constants/quest';
import { PageParams } from '@/app/types/router';
import { getTranslations, setRequestLocale } from 'next-intl/server';

interface QuestsPageProps {
    searchParams: Promise<{
        search?: string;
        view?: QuestType;
    }>;
    params: PageParams
}

export async function generateMetadata({ params }: QuestsPageProps): Promise<Metadata> {
    const { locale } = await params;
    setRequestLocale(locale);
    const t = await getTranslations({ locale });

    const title = t('seo.quests_title');
    const description = t('seo.quests_description');
    const keywords = t('seo.quests_keywords').split(',');

    return {
        title,
        description,
        keywords,
        openGraph: {
            title: `${title} | ${t('seo.site_name')}`,
            description,
            type: 'website',
        },
    };
}

export default async function QuestsPage({ searchParams, params }: QuestsPageProps) {
    const sParams = await searchParams;
    const { locale } = await params;
    setRequestLocale(locale);
    const searchTerm = sParams.search || '';
    const viewMode = sParams.view || QuestType.LIST;
    const questData = fetchAllByFile<IQuestGraph>('quest.json');
    const t = await getTranslations({ locale });

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
                <Suspense fallback={
                    <div className="mb-6 h-12 bg-gray-200 dark:bg-gray-700 rounded animate-pulse" />
                }>
                    <QuestControls />
                </Suspense>

                {/* Content */}
                {viewMode === 'list' ? (
                    <QuestList
                        quests={questData.nodes}
                        searchTerm={searchTerm}
                        translations={{
                            showing: t('quests.showing'),
                            of: t('quests.of'),
                            quests: t('quests.quests'),
                            npc: t('quests.npc'),
                            proxyQuest: t('quests.proxy_quest'),
                            noResults: t('quests.no_results'),
                        }}
                    />
                ) : (
                    <div className="text-center py-20 " style={{ height: 800 }}>
                        <QuestGraph nodes={questData.nodes} edges={questData.connections} />
                    </div>
                )}
            </main>
        </div>
    );
}

export { generateStaticParams };
// export const dynamicParams = false;