import type { NextConfig } from 'next';
import createNextIntlPlugin from 'next-intl/plugin';

const nextConfig: NextConfig = {
    /* config options here */
    // trailingSlash: true,
    webpack: (config, { isServer }) => {
        if (!isServer) {
            config.optimization.splitChunks = {
                chunks: 'all',
                cacheGroups: {
                    default: false,
                    vendors: false,
                    // 合并更多代码到主包
                    commons: {
                        name: 'commons',
                        chunks: 'all',
                        minChunks: 2,
                    },
                },
            };
        }
        return config;
    },
    compress: false, // 禁用 gzip 压缩，由 Vercel Edge 处理
    images: {
        unoptimized: true, // 完全禁用 Vercel 优化
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
    output: 'standalone',
};

const withNextIntl = createNextIntlPlugin();
export default withNextIntl(nextConfig);