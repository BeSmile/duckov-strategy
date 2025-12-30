import { NextResponse } from 'next/server'
import type { NextRequest } from 'next/server'
import createMiddleware from 'next-intl/middleware';
import { routing } from '@/app/i18n/routing';

const intlMiddleware = createMiddleware(routing);

// This function can be marked `async` if using `await` inside
export async function proxy(request: NextRequest) {
    const pathname = request.nextUrl.pathname;

    // 排除 sitemap, robots, api 等
    if (
        pathname.startsWith('/sitemap') ||
        pathname.startsWith('/robots') ||
        pathname.startsWith('/api') ||
        pathname.startsWith('/_next') ||
        pathname.includes('.')  // 所有带文件扩展名的请求
    ) {
        return NextResponse.next();
    }

    return intlMiddleware(request);
}

// See "Matching Paths" below to learn more
export const config = {
    matcher: [
        '/((?!api/|_next/static|_next/image|favicon.ico|sitemap|robots|.*\\.(?:png|jpg|jpeg|gif|svg|ico|webp|woff2?|ttf|eot)).*)',
    ]

}