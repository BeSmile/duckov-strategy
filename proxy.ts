import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'
import { getLocale, setCookie } from '@/app/actions/cookies';
import { defaultLanguage, Language } from '@/app/i18n/config';
import { LANG_KEY } from '@/app/constants';
import { parseLang } from '@/app/utils/lang';
import createMiddleware from 'next-intl/middleware';
import { routing } from '@/app/i18n/routing';

const intlMiddleware = createMiddleware(routing);

// This function can be marked `async` if using `await` inside
export async function proxy(request: NextRequest) {
    const langs = request.headers.get('accept-language') || defaultLanguage;
    const language = langs?.split(",")?.[0] as Language || defaultLanguage;
    let cookieLocale = await getLocale();
    if (decodeURIComponent(cookieLocale).indexOf(",") >= 0 || !cookieLocale) {
        cookieLocale = parseLang(language);
    }
    await setCookie(LANG_KEY, cookieLocale);

    const pathname = request.nextUrl.pathname;

    const response = NextResponse.next();

    // 将当前路径存入 header
    response.headers.set('x-current-pathname', pathname);
    response.headers.set('x-current-url', request.url);

    return intlMiddleware(request);
}

// See "Matching Paths" below to learn more
export const config = {
    matcher: [
        '/((?!api|_next/static|_next/image|locales|favicon.ico|.*\\.png$|.*\\.jpg$|.*\\.jpeg$|.*\\.gif$|.*\\.svg$|.*\\.ico$).*)',
    ]
}