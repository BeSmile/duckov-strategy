import { UnityReference } from '@/app/types/item';

export interface ItemPoolEntry {
    value: {
        itemTypeID: number;
    };
    weight: number;
    percent: string | null;
}

export interface QualityEntry {
    value: number;
    weight: number;
    percent: string | null;
}

export interface ItemDrop {
    comment: string;
    chance: number;
    randomCount: {
        x: number;
        y: number;
    };
    controlDurability: number;
    durability: {
        x: number;
        y: number;
    };
    durabilityIntegrity: {
        x: number;
        y: number;
    };
    randomFromPool: number;
    itemPool: {
        entries: ItemPoolEntry[];
    };
    // addtionalRequireTags: any[];
    // excludeTags: any[];
    qualities: {
        entries: QualityEntry[];
    };
}

export interface Monster {
    m_Name: string;
    nameKey: string;
    health: number;
    exp: number;
    characterModel?: UnityReference;
    shootCanMove?: number;
    sightDistance?: number;
    sightAngle?: number;
    reactionTime?: number;
    nightReactionTimeFactor?: number;
    shootDelay?: number;
    hearingAbility?: number;
    patrolRange?: number;
    combatMoveRange?: number;
    canDash?: number;
    minTraceTargetChance?: number;
    maxTraceTargetChance?: number;
    forgetTime?: number;
    defaultWeaponOut?: number;
    canTalk?: number;
    patrolTurnSpeed?: number;
    combatTurnSpeed?: number;
    wantItem?: number;
    moveSpeedFactor?: number;
    bulletSpeedMultiplier?: number;
    gunDistanceMultiplier?: number;
    nightVisionAbility?: number;
    gunScatterMultiplier?: number;
    scatterMultiIfTargetRunning?: number;
    scatterMultiIfOffScreen?: number;
    damageMultiplier?: number;
    gunCritRateGain?: number;
    aiCombatFactor?: number;
    hasSkill?: number;
    hasSkillChance?: number;
    skillSuccessChance?: number;
    itemSkillChance?: number;
    itemSkillCoolTime?: number;
    elementFactor_Physics?: number;
    elementFactor_Fire?: number;
    elementFactor_Poison?: number;
    elementFactor_Electricity?: number;
    elementFactor_Space?: number;
    elementFactor_Ghost?: number;
    itemsToGenerate: ItemDrop[];
}

export interface MonsterData {
    [key: string]: Monster;
}
