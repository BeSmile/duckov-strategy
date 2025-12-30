'use client';

import { type Language } from '../i18n/config';
import { languages, LOCALES } from '@/app/constants';
import {useRouter, usePathname} from '@/app/i18n/navigation';
import {
    DropdownMenu,
    DropdownMenuContent, DropdownMenuItem,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Globe } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useLocale } from 'use-intl';

export default function LanguageSwitcher() {
    const locale = useLocale() as Language;
    const router = useRouter();
    const pathname = usePathname();

    const handleLanguageChange = (newLocale: string) => {
        router.replace(pathname, { locale: newLocale });
    };

    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" className="h-9 w-9">
                    <Globe className="h-4 w-4" />
                    <span className="sr-only">Switch language</span>
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
                {LOCALES.map((loc) => (
                    <DropdownMenuItem
                        key={loc}
                        onClick={() => handleLanguageChange(loc)}
                        className={locale === loc ? 'bg-accent' : ''}
                    >
                        {languages[loc as keyof typeof languages]?.name}
                    </DropdownMenuItem>
                ))}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}