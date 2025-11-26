'use client';

import { usePathname } from 'next/navigation';
import { useTranslations } from 'next-intl';
import LanguageSwitcher from './LanguageSwitcher';
import ThemeSwitcher from './ThemeSwitcher';
import { LocaleLink } from '@/app/components/ClientProxy';
import { Language } from '@/app/i18n/config';

type HeaderProps = {
    locale: Language;
}

export default function Header({ locale }: HeaderProps) {
    const pathname = usePathname();

    const t =  useTranslations();

    const navLinks = [
        { href: '/', label: t('nav.home') },
        { href: '/inventory', label: t('nav.inventory') },
        { href: '/monsters', label: t('nav.monsters') },
        { href: '/quests', label: t('nav.quests') },
        { href: '/archived', label: t('nav.archived') },
    ];

    const isActive = (href: string) => {
        const langPath = `/${locale}${href}`;
        if (href === '/') {
            return `${pathname}/` === langPath;
        }
        return pathname.startsWith(langPath);
    };

    return (
        <header className="sticky top-0 z-50 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 shadow-sm">
            <nav className="max-w-7xl mx-auto px-4">
                <div className="flex items-center justify-between h-16">
                    {/* Logo/Brand */}
                    <LocaleLink locale={locale} href="/" className="flex items-center gap-3 group">
                        <div className="text-base sm:text-lg md:text-xl lg:text-2xl font-bold text-gray-900 dark:text-gray-100 group-hover:text-blue-500 dark:group-hover:text-blue-400 transition-colors whitespace-nowrap">
                            逃离鸭科夫
                        </div>
                    </LocaleLink>

                    {/* Navigation Links, Theme Switcher & Language Switcher */}
                    <div className="flex items-center gap-4">
                        {/* gap-1 会导致 lg:flex失效 */}
                        <div className="hidden lg:flex items-center">
                            {navLinks.map((link, idx) => (
                                <LocaleLink
                                    locale={locale}
                                    key={idx}
                                    href={link.href}
                                    className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                                        isActive(link.href)
                                            ? 'bg-blue-500 text-white'
                                            : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'
                                    }`}
                                >
                                    {link.label}
                                </LocaleLink>
                            ))}
                        </div>
                        <ThemeSwitcher />
                        <LanguageSwitcher locale={locale} />
                    </div>
                </div>
            </nav>
        </header>
    );
}
