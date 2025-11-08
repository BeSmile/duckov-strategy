import path from 'path';
import fs from 'fs';
import { KeyValue } from '@/app/types/item';
import { EnumKeys, languages } from '@/app/constants';
import { Language } from '@/app/i18n/config';

export function generateKeyValueFetch<T extends EnumKeys>(key: T) {
    return function fetchFn(locale: Language) {
        const lang: string = languages[locale].dir;
        const filePath = path.join(
            process.cwd(),
            'public',
            'language',
            lang,
            `${key}.txt`
        );
        const placeHolderFilePath = path.join(
            process.cwd(),
            'public',
            'language',
            lang,
            `${key}_PlaceHolder.txt`
        );
        const lines = fs.readFileSync(filePath, 'utf8').split(/\r?\n/);
        const placeHolderLines = fs
            .readFileSync(placeHolderFilePath, 'utf8')
            .split(/\r?\n/);
        const values: KeyValue = [...placeHolderLines, ...lines].reduce(
            (acc: KeyValue, line: string) => {
                const [key, value] = line.split(/,/);
                if (key === undefined || value === null) {
                    return acc;
                }
                acc[key.trim()] = value;
                return acc;
            },
            {}
        );
        return values;
    };
}

export function fetchAllByFile<T>(file: string): T {
    const filePath = path.join(process.cwd(), 'public', file);
    const jsonData = fs.readFileSync(filePath, 'utf8');
    return JSON.parse(jsonData) as T;
}
