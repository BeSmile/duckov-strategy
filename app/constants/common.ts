export const LANG = ['chinesesimplified', 'chinesetraditional', 'japanese'];

export const TAG_KEY = 'Tags' as const;
export const CHARACTER_KEY = 'Characters' as const;
export const ITEM_KEY = 'Items' as const;
export const QUEST_KEY = 'Items' as const;
export const languages = {
    'zh-CN': { name: '简体中文', dir: 'chinesesimplified' },
    'zh-TW': { name: '繁體中文', dir: 'chinesetraditional' },
    'ja': { name: '日本語', dir: 'japanese' },
    'en': { name: 'English', dir: 'english' },
} as const;

export const LANG_KEY = 'locale';

export type EnumKeys =
    | typeof CHARACTER_KEY
    | typeof TAG_KEY
    | typeof ITEM_KEY
    | typeof QUEST_KEY;
