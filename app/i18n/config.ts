import { languages } from '@/app/constants';

export type Language = keyof typeof languages;

export const defaultLanguage: Language = 'zh-CN';

export const languageKeys = Object.keys(languages) as Language[];