export interface Item {
    id: number;
    name: string;
    //2: 🟦 藍色
    //3: 🟪 紫色 - 中上選擇
    //4: 🟧 金色 - 稀有價值
    //5: 🟥 紅色 - 超高價值
    quality: number;
    displayName: string;
    description: string;
    maxStackCount: number;
    icon: string;
    priceEach: string;
    tags: string[];
}

export type KeyValue = {
    [key: string]: string;
};

export type UnityReference = {
    fileID: number;
    guid: string;
    type: number;
}