import type { NextConfig } from 'next';
import createNextIntlPlugin from 'next-intl/plugin';

const nextConfig: NextConfig = {
    /* config options here */
    // trailingSlash: true,
    images: {
        minimumCacheTTL: 2678400,
        formats: ['image/webp'],
        qualities: [25, 50, 75, 100],
    },
    async headers() {
        return [
            {
                source: '/images/:path*',  // 匹配 public/images 下的所有文件
                headers: [
                    {
                        key: 'Cache-Control',
                        value: 'public, max-age=2678400, immutable',
                    },
                ],
            },
        ];
    },
};

const withNextIntl = createNextIntlPlugin();
export default withNextIntl(nextConfig);