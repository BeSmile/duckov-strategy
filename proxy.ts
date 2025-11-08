import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'
import { getLocale, removeCookie, setCookie } from '@/app/actions/cookies';
import { defaultLanguage } from '@/app/i18n/config';
import { LANG_KEY } from '@/app/constants';

// This function can be marked `async` if using `await` inside
export async function proxy(request: NextRequest) {
    const langs = request.headers.get('accept-language') || defaultLanguage;
    const language = langs?.split(",")?.[0] || defaultLanguage;
    const cookieLocale = await getLocale();
    if (cookieLocale.indexOf(",") >= 0) {
        await removeCookie(LANG_KEY);
    }
    await setCookie(LANG_KEY, cookieLocale || language);

    return NextResponse.next();
}

// See "Matching Paths" below to learn more
export const config = {
    matcher: [
        '/((?!api|_next/static|_next/image|favicon.ico).*)',
    ]
}