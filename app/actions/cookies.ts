'use server';

import { cookies } from 'next/headers';
import { KeyValue } from '@/app/types/global';
import { Language } from '@/app/i18n/config';

export async function getCookies(name: string) {
    const cookieStore = await cookies();
    return cookieStore.get(name)?.value;
}

export async function getLocale(): Promise<Language> {
    return await getCookies('locale') as Language;
}

export async function getAllCookies() {
    const cookieStore = await cookies();
    return cookieStore.getAll().reduce((acc, cookie) => {
        acc[cookie.name] = cookie.value;

        return acc;
    }, {} as KeyValue)
}

export async function removeCookie(name: string) {
    const cookieStore = await cookies();
    cookieStore.delete(name);
}

/**
 * 设置cookie
 * @param name
 * @param value
 * @param options
 */
export async function setCookie(name: string, value: string, options?: {
    maxAge?: number;
    httpOnly?: boolean;
    secure?: boolean;
}) {
    const cookieStore = await cookies();
    cookieStore.set(name, value, {
        httpOnly: options?.httpOnly ?? false, // 默认允许客户端访问
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: options?.maxAge ?? 60 * 60 * 24 * 7,
        path: '/',
        ...options,
    })
}