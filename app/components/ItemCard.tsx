import Image from 'next/image';
import { Item } from '../types/item';
import { getQualityConfig } from '@/app/constants/quality';
import { Language } from '@/app/i18n/config';
import { getTranslations } from 'next-intl/server';
import { LinkProps } from 'next/link';
import { Link } from '@/app/i18n/navigation';
import { PREFETCH } from '@/app/constants';
import { getItemKey, getItemName, TrFn } from '@/app/utils/lang';

interface ItemCardProps {
    item: Item;
    locale: Language;
}

export type ItemLinkProps =  Omit<LinkProps, 'href'>& {
    locale: Language;
    item: Item;
    t: TrFn;
    extra?: React.ReactNode;
    qualityBorder?: boolean;
    disabled?: boolean;
    className?: string;
    border?: string;
    badge?: React.ReactNode;
};

export function ItemLink({ locale, item, extra, t, qualityBorder, disabled, badge, className, border = "border-gray-200 dark:border-gray-600" }: ItemLinkProps){
    const qualityConfig = getQualityConfig(item.quality);
    const borderColor =  qualityBorder ? qualityConfig.borderColor: '';
    const color = disabled ? 'grayscale' : borderColor;
    return <Link
        locale={locale}
        href={`/inventory/${item!.id}`}
        className={`${color} relative flex items-center  justify-center gap-2  p-2 bg-gray-50 dark:bg-gray-700 rounded border ${border} h-14 hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors ${className}`}
    >
        {badge && badge}
        <div className='relative w-8 h-8 flex-shrink-0 bg-gray-100 dark:bg-gray-600 rounded'>
            <Image
                src={`/images/${item!.icon}`}
                alt={item!.displayName}
                fill
                loading="lazy"
                className="object-contain"
                sizes="32px"
            />
        </div>
        <div className="flex-1 min-w-0">
            <p className="text-xs font-medium text-gray-900 dark:text-gray-100 truncate">
                {getItemName(
                    t,
                    item as Item,
                )}
            </p>
            {extra}
        </div>
    </Link>
}

export default async function ItemCard({ item, locale }: ItemCardProps)
{
    const t = await getTranslations({ locale });
    const qualityConfig = getQualityConfig(item.quality);
    const qualityName = t.has(qualityConfig.nameKey) ? t(qualityConfig.nameKey) : qualityConfig.name;

    return (
        <Link prefetch={PREFETCH} locale={locale} href={`/inventory/${item.id}`} className="block">
            <div className={`border flex-1/6 rounded-lg p-4 hover:shadow-lg transition-shadow h-[300px] flex flex-col ${qualityConfig.borderColor}`}>
                <div className="flex flex-col items-center gap-3 flex-1">
                    <div className="relative w-16 h-16 flex-shrink-0 bg-gray-100 dark:bg-gray-700 rounded">
                        <Image
                            src={`/images/${item.icon}`}
                            alt={item.displayName}
                            fill
                            loading="lazy"
                            className="object-contain"
                            sizes="64px"
                            // onError={(e) => {
                            //   const target = e.target as HTMLImageElement;
                            //   target.src = '/placeholder.png';
                            // }}
                        />
                    </div>
                    <div className="text-center w-full flex-shrink-0">
                        <h3 className="font-semibold text-sm text-gray-900 dark:text-gray-100 truncate">
                            {getItemName(t, item)}
                        </h3>
                        <span className={`inline-block px-2 py-0.5 rounded text-xs font-medium ${qualityConfig.bgColor} ${qualityConfig.textColor} mt-1`}>
                            {qualityName}: {item.quality}
                        </span>
                        <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">
                            ID: {item.id}
                        </p>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1 line-clamp-2 h-8">
                            {getItemKey(t, item, 'description')}
                        </p>
                    </div>
                    <div className="flex justify-between w-full text-xs text-gray-600 dark:text-gray-300 border-t border-gray-100 dark:border-gray-700 pt-2 flex-shrink-0">
                        <span>Price: {item.priceEach}</span>
                        <span>Stack: {item.maxStackCount}</span>
                    </div>
                    <div className="w-full flex-1 overflow-y-auto min-h-0">
                        <div className="flex flex-wrap gap-1">
                            {item?.tags.map((tag) => (
                                <span
                                    key={tag}
                                    className="px-2 py-0.5 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded text-xs font-medium whitespace-nowrap"
                                >
                                    {t(`tags.Tag_${tag}`)}
                                </span>
                            ))}
                        </div>
                    </div>
                </div>
            </div>
        </Link>
    );
}

