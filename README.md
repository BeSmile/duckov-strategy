# 逃离鸭科夫 - 游戏攻略网站

这是一个为游戏《逃离鸭科夫》打造的全功能攻略网站，提供物品查询、怪物图鉴、任务系统、存档解析等功能。

## 功能特性

### 核心功能

- **物品库** (`/inventory`)
  - 1200+ 游戏物品完整数据库
  - 支持按 ID、名称搜索
  - 多标签组合筛选
  - 品质颜色标识
  - 分页展示（每页 10 项）
  - 物品详情页，包含掉落来源、属性、插槽信息

- **怪物图鉴** (`/monsters`)
  - 怪物基础属性（HP、经验值）
  - 详细掉落物品列表
  - 掉落概率显示

- **任务系统** (`/quests`)
  - 任务列表与搜索
  - 可视化任务关系图谱
  - 任务详情（NPC、奖励等）
  - 支持列表/图谱双视图切换

- **Buff 效果** (`/buffs`)
  - 游戏 Buff 效果查询
  - ID、名称、描述展示

- **存档解析** (`/archived`)
  - 上传并解析游戏存档文件
  - 使用 Rust WASM 模块在客户端解析
  - 展示存档中的物品、任务等信息
  - 数据不上传服务器，保护隐私

### 用户体验

- **多语言支持**: 简体中文、繁体中文、日语、英语
- **主题切换**: Light、Dark、Cassette（特色磁带主题）
- **SEO 优化**: 完整的元数据、Sitemap、结构化数据
- **性能优化**: 静态生成、缓存策略、WebP 图片优化

## 技术栈

### 前端

- **框架**: Next.js 16.0.7 (App Router)
- **UI**: React 19.2.1 + Tailwind CSS 4
- **组件库**: Radix UI、Lucide React、Vaul
- **可视化**: @xyflow/react（任务图谱）
- **国际化**: next-intl 4.5.0 + i18next
- **主题**: next-themes
- **语言**: TypeScript 5.x（严格模式）

### 后端 / WebAssembly

- **语言**: Rust
- **WASM 模块**:
  - `wgpu-renderer`: 基于 WebGPU 的游戏场景渲染引擎
  - `savefile-parse`: 游戏存档解析工具

## 项目结构

### 目录说明

```
duckov-strategy/
├── app/[locale]/              # 国际化页面路由（核心内容）
│   ├── page.tsx              # 首页
│   ├── layout.tsx            # 根布局
│   ├── inventory/            # 物品库页面
│   ├── monsters/             # 怪物图鉴页面
│   ├── quests/               # 任务系统页面
│   ├── buffs/                # Buff 效果页面
│   ├── archived/             # 存档解析页面
│   └── maps/                 # 地图功能（暂时排除）
├── app/components/           # React 组件
├── app/types/                # TypeScript 类型定义
├── app/utils/                # 工具函数
├── app/constants/            # 常量配置
├── public/                   # 静态资源
│   ├── images/               # 物品图标（613 张）
│   ├── prefabs/              # 物品数据（1294 个文件）
│   ├── language/             # 游戏多语言文件
│   ├── items.json            # 物品数据（24,972 行）
│   ├── loot.json             # 掉落数据（24,959 行）
│   └── quest.json            # 任务数据（7,021 行）
├── src/locales/              # 网站界面翻译
├── wgpu-renderer/            # WebGPU 渲染引擎（Rust）
├── savefile-parse/           # 存档解析器（Rust）
└── components/               # shadcn/ui 组件库
```

## 快速开始

### 环境要求

- Node.js 18+
- Rust 和 wasm-pack（用于构建 WASM 模块）

### 安装

```bash
# 安装依赖
npm install
# 或
pnpm install
```

### 开发

```bash
# 启动开发服务器
npm run dev

# 访问 http://localhost:3000
```

### 构建

```bash
# 构建生产版本
npm run build

# 构建游戏渲染器（WASM）
npm run build:game

# 构建存档解析器（WASM）
npm run build:save
```

### 代码规范

```bash
# ESLint 检查
npm run lint

# Prettier 格式化
npm run format
```

## 国际化

### 支持语言

- 简体中文 (zh-CN) - 默认
- 繁体中文 (zh-TW)
- 日语 (ja)
- 英语 (en)

### 实现方式

- **URL 路径**: `/{locale}/inventory` 格式
- **服务端**: next-intl 提供 SSR 翻译
- **客户端**: i18next + react-i18next
- **游戏内容**: 从 `/public/language/` 读取游戏原始翻译文件

### 添加新语言

1. 在 `src/locales/` 添加对应语言文件夹
2. 在 `app/constants/common.ts` 中配置语言信息
3. 在 `app/i18n/routing.ts` 添加语言代码

## 开发指南

### WASM 模块开发

#### 渲染引擎（wgpu-renderer）

```bash
cd wgpu-renderer
wasm-pack build --target web
```

功能模块：
- Unity 场景解析
- WebGPU 3D 渲染
- 相机控制系统
- 光照与材质

#### 存档解析器（savefile-parse）

```bash
cd savefile-parse
wasm-pack build --target web
```

功能：
- 读取二进制存档
- 提取游戏数据
- JSON 格式化

### 数据更新流程

1. 从游戏文件提取最新数据
2. 更新 `public/` 下的 JSON 文件
3. 更新 `public/language/` 翻译文件
4. 重新构建并测试

### 性能优化

- **静态生成**: 所有语言版本预生成
- **图片优化**: WebP 格式，31 天缓存
- **代码分割**: 按路由自动分割
- **数据加载**: 零网络请求（本地 JSON）

## 部署

### Vercel（推荐）

```bash
# 自动部署
vercel

# 或连接 GitHub 自动部署
```

### 其他平台

确保支持：
- Node.js 18+
- 静态文件服务
- 环境变量配置

## SEO 配置

- **动态元数据**: 每个页面独立的 title、description
- **Open Graph**: 社交媒体分享优化
- **Sitemap**: `/sitemap.ts` 自动生成
- **Robots.txt**: `/robots.ts` 配置

## 项目亮点

1. **混合架构**: Next.js + Rust WASM，结合 Web 和系统级性能
2. **完整国际化**: 4 种语言全面支持
3. **丰富数据**: 1200+ 物品、完整掉落表、任务关系图谱
4. **隐私保护**: 客户端存档解析，数据不上传
5. **用户体验**: 搜索、筛选、分页、主题切换
6. **SEO 友好**: 完整元数据和结构化数据

## 技术特色

- **App Router**: 使用 Next.js 最新路由系统
- **TypeScript 严格模式**: 类型安全
- **Tailwind CSS 4**: 现代化样式方案
- **WebAssembly**: 高性能数据处理
- **React Server Components**: 优化性能

## 许可证

本项目基于游戏《逃离鸭科夫》的数据构建，仅供学习和参考使用。

## 相关链接

- [Next.js 文档](https://nextjs.org/docs)
- [Tailwind CSS](https://tailwindcss.com)
- [Rust WASM](https://rustwasm.github.io/)