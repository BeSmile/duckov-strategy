import { getRequestConfig } from 'next-intl/server';
import { BUFF_KEY, CHARACTER_KEY, ITEM_KEY, LANG_KEY, LOCALES, TAG_KEY } from '@/app/constants';
import { generateKeyValueFetch } from '@/app/utils/request';
import { defaultLanguage, Language } from '@/app/i18n/config';
import { cookies, headers } from 'next/headers';

const fetchBuffsLangs = generateKeyValueFetch(BUFF_KEY);
const fetchTagsLangs = generateKeyValueFetch(TAG_KEY);
const fetchItemI18 = generateKeyValueFetch(ITEM_KEY);
const fetchCharacterI18 = generateKeyValueFetch(CHARACTER_KEY);

export default getRequestConfig(async ({ locale  }) => {

    // 处理非国际化路径（sitemap, robots, api 等）
    if (!locale) {
        const cookieStore = await cookies();
        locale = cookieStore.get(LANG_KEY)?.value;
    }

    // 如果还没有，尝试从 Accept-Language header 获取
    if (!locale) {
        const headersList = await headers();
        const acceptLanguage = headersList.get('accept-language');
        // 简单解析，实际可能需要更复杂的逻辑
        locale = acceptLanguage?.split(',')[0]?.split('-')[0];
    }

    // 验证 locale 是否有效
    if (!locale || !LOCALES.includes(locale)) {
        locale = defaultLanguage;
    }

    const lang = locale as Language;

    const locales = fetchBuffsLangs(lang);
    const tagLocales = fetchTagsLangs(lang);
    const itemLocales = fetchItemI18(lang);
    const monstersLocales = fetchCharacterI18(lang);

    return {
        locale: lang as string,
        messages: {
            buffs: locales,
            tags: tagLocales,
            items: itemLocales,
            characters: monstersLocales,
            ...(await import(`../locales/${lang}/common.json`)).default,
            ...(await import(`../locales/${lang}/entry.json`)).default,
        },
        // 关键配置：处理缺失的翻译
        onError: (error) => {
            if (error.code === 'MISSING_MESSAGE') {
                console.warn('Missing translation:', error.originalMessage);
                // 不抛出错误，只警告
            } else {
                console.error(error);
            }
        },
        getMessageFallback: ({key}) => {
            // 返回 fallback 内容而不是崩溃
            return `${key}`;
        }
    };
});
