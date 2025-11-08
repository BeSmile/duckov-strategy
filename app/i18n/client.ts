'use client'

import i18next from 'i18next';
import { initReactI18next } from 'react-i18next';
import resourcesToBackend from 'i18next-resources-to-backend';
import { defaultLanguage, type Language } from './config';
import { getCookies, setCookie } from '@/app/actions/cookies';
import { LANG_KEY, languages } from '@/app/constants';

const runsOnServerSide = typeof window === 'undefined';

// Initialize i18next
i18next
    .use(initReactI18next)
    .use(
        resourcesToBackend(
            (language: string, namespace: string) =>
                import(`./locales/${language}/${namespace}.json`)
        )
    )
    .init({
        lng: defaultLanguage,
        fallbackLng: defaultLanguage,
        supportedLngs: Object.keys(languages),
        defaultNS: 'common',
        fallbackNS: 'common',
        ns: ['common'],
        preload: runsOnServerSide ? Object.keys(languages) : [],
        interpolation: {
            escapeValue: false,
        },
        react: {
            useSuspense: false,
        },
    });

export default i18next;

export async function changeLanguage(lang: Language) {
    if (i18next.language !== lang) {
        await i18next.changeLanguage(lang);
        await setCookie(LANG_KEY, lang);
    }
}

export async function getStoredLanguage(): Promise<Language> {
    const defaultLang = await getCookies(LANG_KEY) || defaultLanguage;
    return defaultLang as Language;
}