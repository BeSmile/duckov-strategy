import 'server-only';
import { getLocale } from '@/app/actions/cookies';
import type { Language } from './config';

// Type for the translation keys
type TranslationKeys = {
    [key: string]: string | TranslationKeys;
};

// Cache for loaded translations
const translationsCache: Map<string, TranslationKeys> = new Map();

async function loadTranslation(
    locale: Language,
    namespace: string = 'common'
): Promise<TranslationKeys> {
    const cacheKey = `${locale}-${namespace}`;

    if (translationsCache.has(cacheKey)) {
        return translationsCache.get(cacheKey)!;
    }

    try {
        const translation = await import(
            `./locales/${locale}/${namespace}.json`
        );
        translationsCache.set(cacheKey, translation.default);
        return translation.default;
    } catch (error) {
        console.error(`Failed to load translation: ${cacheKey}`, error);
        return {};
    }
}

function getNestedValue(obj: TranslationKeys, path: string): string {
    const keys = path.split('.');

    let current: TranslationKeys | string = obj;

    for (const key of keys) {
        if (current && typeof current === 'object' && key in current) {
            current = current[key];
        } else {
            return path; // Return the key itself if not found
        }
    }

    return typeof current === 'string' ? current : path;
}

export async function getServerTranslation(namespace: string = 'common') {
    const locale = await getLocale();

    return {
        t: async (key: string): Promise<string> => {
            const translations = await loadTranslation(locale, namespace);
            return getNestedValue(translations, key);
        },
        locale,
    };
}