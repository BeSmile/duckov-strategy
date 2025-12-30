import { defineRouting } from 'next-intl/routing';
import { defaultLanguage, languageKeys } from '@/app/i18n/config';
import { LANG_KEY } from '@/app/constants';

export const routing = defineRouting({
    locales: languageKeys,
    // Used when no locale matches
    defaultLocale: defaultLanguage,
    localePrefix: 'always',
    localeCookie: {
        // Custom cookie name
        name: LANG_KEY,
        // Expire in one year
        maxAge: 60 * 60 * 24 * 365
    },
    localeDetection: true,
});