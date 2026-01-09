import { PageParamsProps } from '@/app/types/router';
import ArchiveUpload from '@/app/components/ArchiveUpload';
import { fetchAllByFile } from '@/app/utils/request';
import { Item } from '@/app/types/item';
import { MonsterData } from '@/app/types/monster';
import { IQuestGraph } from '@/app/types/quest';
import { setRequestLocale } from 'next-intl/server';

type ArchivedProps = PageParamsProps

export default async function Archived({ params }: ArchivedProps) {

    const { locale } = await params;
    setRequestLocale(locale);
    const items = fetchAllByFile<Item[]>('items.json');// 获取物品
    const monsterData = fetchAllByFile<MonsterData>('loot.json');// 获取怪物信息
    const questData = fetchAllByFile<IQuestGraph>('quest.json');

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900 py-8 px-4">
            <main className="max-w-7xl mx-auto">
                <ArchiveUpload questData={questData} monsters={monsterData} items={items} locale={locale} />
            </main>
        </div>
    )
}

export { generateStaticParams } from '@/lib/getStatic';
export const dynamic = 'force-static';