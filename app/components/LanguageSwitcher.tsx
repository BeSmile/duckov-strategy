'use client';

import { useState, useEffect, useRef, useCallback } from 'react';
import { type Language, languageKeys } from '../i18n/config';
import { getStoredLanguage, changeLanguage } from '../i18n/client';
import { languages } from '@/app/constants';
import {useRouter, usePathname} from '@/app/i18n/navigation';
import { useLocale } from 'use-intl';

export default function LanguageSwitcher() {
    const [isOpen, setIsOpen] = useState(false);
    const locale = useLocale() as Language;
    const pathname = usePathname().replace(locale, '');
    const pathWithoutLocale = pathname.replace(`/${locale}`, '') || '/';

    const router = useRouter();
    const [currentLang, setCurrentLang] = useState<Language>(locale);
    const dropdownRef = useRef<HTMLDivElement>(null);

    // 初始化设置一次
    const handleLanguages = useCallback(async () => {
        const stored = await getStoredLanguage();
        setCurrentLang(stored);
        await changeLanguage(stored);
    }, []);

    useEffect(() => {
        // eslint-disable-next-line react-hooks/set-state-in-effect
        void handleLanguages();
    }, [handleLanguages]);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (
                dropdownRef.current &&
                !dropdownRef.current.contains(event.target as Node)
            ) {
                setIsOpen(false);
            }
        };

        if (isOpen) {
            document.addEventListener('mousedown', handleClickOutside);
        }

        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, [isOpen]);

    const handleLanguageChange = async (lang: Language) => {
        setCurrentLang(lang);
        await changeLanguage(lang);
        setIsOpen(false);
        router.replace(pathWithoutLocale, {locale: lang as Language});
    };

    return (
        <div className="relative" ref={dropdownRef}>
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="flex items-center gap-2 px-3 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
            >
                <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129"
                    />
                </svg>
                <span>{languages[currentLang].name}</span>
                <svg
                    className={`w-4 h-4 transition-transform ${
                        isOpen ? 'rotate-180' : ''
                    }`}
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M19 9l-7 7-7-7"
                    />
                </svg>
            </button>

            {isOpen && (
                <div className="absolute right-0 mt-2 w-48 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg z-50">
                    {languageKeys.map((lang) => (
                        <button
                            key={lang}
                            onClick={() => handleLanguageChange(lang)}
                            className={`w-full text-left px-4 py-2 text-sm transition-colors first:rounded-t-lg last:rounded-b-lg ${
                                currentLang === lang
                                    ? 'bg-blue-50 dark:bg-blue-900 text-blue-700 dark:text-blue-200 font-medium'
                                    : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700'
                            }`}
                        >
                            {languages[lang].name}
                        </button>
                    ))}
                </div>
            )}
        </div>
    );
}