'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import LanguageSwitcher from './LanguageSwitcher';

export default function Header() {
    const pathname = usePathname();
    const { t } = useTranslation();

    const navLinks = [
        { href: '/', label: t('nav.home') },
        { href: '/inventory', label: t('nav.inventory') },
        { href: '/monsters', label: t('nav.monsters') },
        { href: '/quests', label: t('nav.quests') },
    ];

    const isActive = (href: string) => {
        if (href === '/') {
            return pathname === '/';
        }
        return pathname.startsWith(href);
    };

    return (
        <header className="sticky top-0 z-50 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 shadow-sm">
            <nav className="max-w-7xl mx-auto px-4">
                <div className="flex items-center justify-between h-16">
                    {/* Logo/Brand */}
                    <Link href="/" className="flex items-center gap-3 group">
                        <div className="text-2xl font-bold text-gray-900 dark:text-gray-100 group-hover:text-blue-500 dark:group-hover:text-blue-400 transition-colors">
                            逃离鸭科夫
                        </div>
                    </Link>

                    {/* Navigation Links & Language Switcher */}
                    <div className="flex items-center gap-4">
                        <div className="flex items-center gap-1">
                            {navLinks.map((link) => (
                                <Link
                                    key={link.href}
                                    href={link.href}
                                    className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                                        isActive(link.href)
                                            ? 'bg-blue-500 text-white'
                                            : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'
                                    }`}
                                >
                                    {link.label}
                                </Link>
                            ))}
                        </div>
                        <LanguageSwitcher />
                    </div>
                </div>
            </nav>
        </header>
    );
}
