import Link from 'next/link';
import { getServerTranslation } from '@/app/i18n/server';

export default async function Home() {
    const { t } = await getServerTranslation();

    return (
        <div className="flex min-h-screen items-center justify-center bg-gray-50 dark:bg-gray-900">
            <main className="flex flex-col items-center gap-8 py-16 px-8">
                <div className="text-center max-w-3xl">
                    <h1 className="text-5xl font-bold text-gray-900 dark:text-gray-100 mb-4">
                        {await t('home.title')}
                    </h1>
                    <p className="text-lg text-gray-600 dark:text-gray-400 mb-6">
                        {await t('home.subtitle')}
                    </p>

                    {/* Steam Link */}
                    <a
                        href="https://store.steampowered.com/app/3167020/_/?l=schinese"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center gap-3 px-6 py-3 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 text-white font-semibold rounded-lg shadow-lg hover:shadow-xl transition-all duration-200"
                    >
                        <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
                            <path d="M11.979 0C5.678 0 .511 4.86.022 11.037l6.432 2.658c.545-.371 1.203-.59 1.912-.59.063 0 .125.004.188.006l2.861-4.142V8.91c0-2.495 2.028-4.524 4.524-4.524 2.494 0 4.524 2.031 4.524 4.527s-2.03 4.525-4.524 4.525h-.105l-4.076 2.911c0 .052.004.105.004.159 0 1.875-1.515 3.396-3.39 3.396-1.635 0-3.016-1.173-3.331-2.727L.436 15.27C1.862 20.307 6.486 24 11.979 24c6.627 0 11.999-5.373 11.999-12S18.605 0 11.979 0zM7.54 18.21l-1.473-.61c.262.543.714.999 1.314 1.25 1.297.539 2.793-.076 3.332-1.375.263-.63.264-1.319.005-1.949s-.75-1.121-1.377-1.383c-.624-.26-1.29-.249-1.878-.03l1.523.63c.956.4 1.409 1.5 1.009 2.455-.397.957-1.497 1.41-2.454 1.012H7.54zm11.415-9.303c0-1.662-1.353-3.015-3.015-3.015-1.665 0-3.015 1.353-3.015 3.015 0 1.665 1.35 3.015 3.015 3.015 1.663 0 3.015-1.35 3.015-3.015zm-5.273-.005c0-1.252 1.013-2.266 2.265-2.266 1.249 0 2.266 1.014 2.266 2.266 0 1.251-1.017 2.265-2.266 2.265-1.253 0-2.265-1.014-2.265-2.265z"/>
                        </svg>
                        <span>{await t('home.view_on_steam')}</span>
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                        </svg>
                    </a>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-6 w-full max-w-4xl mt-8">
                    <Link
                        href="/inventory"
                        className="group relative overflow-hidden rounded-lg bg-white dark:bg-gray-800 p-8 shadow-md hover:shadow-xl transition-all border border-gray-200 dark:border-gray-700 hover:border-blue-500 dark:hover:border-blue-500"
                    >
                        <div className="flex flex-col items-center gap-4">
                            <div className="text-4xl">📦</div>
                            <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-100">
                                {await t('home.items_title')}
                            </h2>
                            <p className="text-center text-gray-600 dark:text-gray-400">
                                {await t('home.items_desc')}
                            </p>
                        </div>
                    </Link>

                    <Link
                        href="/monsters"
                        className="group relative overflow-hidden rounded-lg bg-white dark:bg-gray-800 p-8 shadow-md hover:shadow-xl transition-all border border-gray-200 dark:border-gray-700 hover:border-blue-500 dark:hover:border-blue-500"
                    >
                        <div className="flex flex-col items-center gap-4">
                            <div className="text-4xl">👾</div>
                            <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-100">
                                {await t('home.monsters_title')}
                            </h2>
                            <p className="text-center text-gray-600 dark:text-gray-400">
                                {await t('home.monsters_desc')}
                            </p>
                        </div>
                    </Link>

                    <Link
                        href="/quests"
                        className="group relative overflow-hidden rounded-lg bg-white dark:bg-gray-800 p-8 shadow-md hover:shadow-xl transition-all border border-gray-200 dark:border-gray-700 hover:border-blue-500 dark:hover:border-blue-500"
                    >
                        <div className="flex flex-col items-center gap-4">
                            <div className="text-4xl">📋</div>
                            <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-100">
                                {await t('home.quests_title')}
                            </h2>
                            <p className="text-center text-gray-600 dark:text-gray-400">
                                {await t('home.quests_desc')}
                            </p>
                        </div>
                    </Link>
                </div>
            </main>
        </div>
    );
}
