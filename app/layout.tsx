import type { Metadata } from 'next';
import Header from './components/Header';
import I18nProvider from './components/I18nProvider';
import './globals.css';
import { ReactNode } from 'react';

// import {Geist, Geist_Mono} from "next/font/google";
// const geistSans = Geist({
//     variable: "--font-geist-sans",
//     subsets: ["latin"],
// });

// const geistMono = Geist_Mono({
//     variable: "--font-geist-mono",
//     subsets: ["latin"],
// });

export const metadata: Metadata = {
    title: '逃离鸭科夫: 指南',
    description: '你好鸭',
    // icons: {
    //     icon: "/icon.png"
    // }
};

export default function RootLayout({
    children,
}: Readonly<{
    children: ReactNode;
}>) {
    return (
        <html lang="en">
            <body className="antialiased">
                <I18nProvider>
                    <Header />
                    {children}
                </I18nProvider>
            </body>
        </html>
    );
}
