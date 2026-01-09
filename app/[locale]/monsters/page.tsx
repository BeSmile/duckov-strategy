import MonsterCard from '@/app/components/MonsterCard';
import { MonsterData } from '@/app/types/monster';
import { Item } from '@/app/types/item';
import { fetchAllByFile } from '@/app/utils/request';
import { Metadata } from 'next';
import { getTranslations, setRequestLocale } from 'next-intl/server';
import { generateStaticParams } from '@/lib/getStatic';
import { Language } from '@/app/i18n/config';
import { PageParamsProps } from '@/app/types/router';

export async function generateMetadata({ params }: PageParamsProps): Promise<Metadata> {
    const { locale } = await params;
    setRequestLocale(locale);
    const t = await getTranslations({ locale });
    const lootData = fetchAllByFile<MonsterData>('loot.json');
    const monstersCount = Object.keys(lootData).length;

    const title = t('seo.monsters_title');
    const description = t('seo.monsters_description', {
        count: monstersCount.toString(),
    });
    const keywords = t('seo.monsters_keywords').split(',');

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

export default async function MonstersPage({
    params,
}: {
    params: Promise<{ locale: Language }>;
}) {
    const { locale: lang } = await params;
    setRequestLocale(lang);
    const t = await getTranslations({ locale: lang });
    // Load loot.json
    const lootData = fetchAllByFile<MonsterData>('loot.json');

    // Load items.json
    const items = fetchAllByFile<Item[]>('items.json');

    // Convert monster.ts data object to array
    const monsters = Object.entries(lootData).map(([key, monster]) => ({
        ...monster,
        key,
    }));

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
            <main className="max-w-7xl mx-auto">
                <div className="mb-8">
                    <h1 className="text-4xl font-bold text-gray-900 dark:text-gray-100 mb-2">
                        {t('monsters.monsters_enemies')}
                    </h1>
                    <p className="text-gray-600 dark:text-gray-400">
                        {t('common.total')}: {monsters.length} {t('monsters.monsters')}
                    </p>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {monsters.map((monster) => (
                        <MonsterCard
                            locale={lang}
                            key={monster.key}
                            monster={monster}
                            items={items}
                        />
                    ))}
                </div>
            </main>
        </div>
    );
}

export { generateStaticParams };

// export const dynamicParams = false;
export const dynamic = 'force-static';