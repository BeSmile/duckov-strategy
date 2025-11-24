import type { Metadata } from 'next';
import Header from '../components/Header';
import '../globals.css';
import { NextIntlClientProvider } from 'next-intl';
import { getMessages, getTranslations } from 'next-intl/server';
import { Language } from '@/app/i18n/config';
import { generateStaticParams } from '@/lib/getStatic';
import { Analytics } from '@vercel/analytics/next';
import { setRequestLocale } from 'next-intl/server';
import { ThemeProvider } from '@/components/theme-provider';

import { Geist, Geist_Mono } from 'next/font/google';
import { LocaleRouter } from '@/app/types/global';
import { PageParamsProps } from '@/app/types/router';

const geistSans = Geist({
    variable: '--font-geist-sans',
    subsets: ['latin'],
});

const geistMono = Geist_Mono({
    variable: '--font-geist-mono',
    subsets: ['latin'],
});

const localeMap: Record<string, string> = {
    'zh-CN': 'zh_CN',
    'zh-TW': 'zh_TW',
    ja: 'ja_JP',
    en: 'en_US',
};

export async function generateMetadata({
    params,
}: PageParamsProps): Promise<Metadata> {
    const { locale } = await params;
    const t = await getTranslations({ locale });
    const ogLocale = localeMap[locale] || 'zh_CN';
    const siteName = t('seo.site_name');
    const siteDescription = t('seo.site_description');
    const keywords = t('seo.keywords').split(',');

    return {
        title: {
            default: siteName,
            template: `%s | ${siteName}`,
        },
        description: siteDescription,
        keywords,
        authors: [{ name: 'BeSmile' }],
        creator: 'https://github.com/BeSmile/',
        publisher: 'https://github.com/BeSmile/',
        metadataBase: new URL(
            process.env.NEXT_PUBLIC_BASE_URL || 'http://localhost:3000'
        ),
        alternates: {
            canonical: '/',
            languages: {
                'zh-CN': '/zh-CN',
                'zh-TW': '/zh-TW',
                ja: '/ja',
                en: '/en',
            },
        },
        openGraph: {
            type: 'website',
            locale: ogLocale,
            alternateLocale: Object.values(localeMap).filter(
                (l) => l !== ogLocale
            ),
            url: '/',
            siteName,
            title: siteName,
            description: siteDescription,
            images: [
                {
                    url: '/icon.png',
                    width: 1200,
                    height: 630,
                    alt: siteName,
                },
            ],
        },
        // twitter: {
        //     card: 'summary_large_image',
        //     title: siteName,
        //     description: siteDescription,
        //     images: ['/twitter-image.jpg'],
        // },
        robots: {
            index: true,
            follow: true,
            googleBot: {
                index: true,
                follow: true,
                'max-video-preview': -1,
                'max-image-preview': 'large',
                'max-snippet': -1,
            },
        },
        verification: {
            // google: 'your-google-verification-code', // 需要时添加
            // yandex: 'your-yandex-verification-code',
            // bing: 'your-bing-verification-code',
        },
        // icons: {
        //     icon: "/icon.png"
        // }
    };
}

export default async function RootLayout({
    children,
    params,
}: Readonly<LocaleRouter>) {
    const isDev = process.env.NODE_ENV === 'development'

    const { locale } = await params;

    const messages = await getMessages({ locale });
    setRequestLocale(locale);

    return (
        <html lang={locale} suppressHydrationWarning>
            <body
                className={`${geistSans.variable} ${geistMono.variable} antialiased`}
            >
                <ThemeProvider
                    attribute="class"
                    defaultTheme="system"
                    enableSystem
                    disableTransitionOnChange
                >
                    <NextIntlClientProvider locale={locale} messages={messages}>
                        <Header locale={locale as Language} />
                        {children}
                        {!isDev && (
                            <Analytics />
                        )}
                    </NextIntlClientProvider>
                </ThemeProvider>
            </body>
        </html>
    );
}

export { generateStaticParams };
// 控制页面的渲染模式 // auto force-dynamic force-static error
// export const dynamic = 'force-static';  // 强制静态渲染
// export const revalidate = 86400; // 60秒后重新验证

// 动态参数的行为
// export const dynamicParams = false; // 添加这个