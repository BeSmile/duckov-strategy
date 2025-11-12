import { UnityReference } from '@/app/types/item';

export type Buff = {
    id: number;
    displayName: string;
    description: string;
    icon: UnityReference;
    limitedLifeTime?: number;
    totalLifeTime?: number;
    // effects?: Effect[]; // Commented out based on provided Rust struct
    // hide?: number;      // Commented out based on provided Rust struct
    fromWeaponID: number;
}

export type BuffEntry = {
    raw: Buff,
    guid: string,
}