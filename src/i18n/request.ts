import { getRequestConfig } from 'next-intl/server';
import { getLocale } from '@/app/actions/cookies';
import { BUFF_KEY } from '@/app/constants';
import { generateKeyValueFetch } from '@/app/utils/request';

const fetchBuffsLangs = generateKeyValueFetch(BUFF_KEY);

export default getRequestConfig(async () => {
    const locale = await getLocale() || 'en';
    const locales = fetchBuffsLangs(locale);

    return {
        locale: locale as string,
        messages: {
            buffs: locales,
            ...(await import(`../../public/locales/${locale}/common.json`)).default,
        }
    };
});
