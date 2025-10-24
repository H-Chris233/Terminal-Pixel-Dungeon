// src/hero/src/class/class.rs
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

pub mod huntress;
pub mod mage;
pub mod rogue;
pub mod warrior;

use items::{
    Item, ItemKind,
    armor::Armor,
    potion::PotionKind,
    scroll::ScrollKind,
    seed::Seed,
    weapon::{Weapon, WeaponKind},
};

/// 表示职业技能的持久化状态（技能解锁、正在激活的技能等）
#[derive(
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Encode,
    Decode,
    Serialize,
    Deserialize,
)]
pub struct SkillState {
    /// 已解锁的职业技能标识列表（用于未来的职业技能树）
    pub unlocked_talents: Vec<String>,
    /// 当前激活的职业技能（如果有）
    pub active_skill: Option<String>,
}

/// 英雄职业枚举（SPD核心四职业）
#[derive(
    Default,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Encode,
    Decode,
    Serialize,
    Deserialize,
    EnumIter,
    EnumString,
    Display,
)]
pub enum Class {
    #[default]
    #[strum(serialize = "战士")]
    Warrior, // 高生命值，平衡攻防

    #[strum(serialize = "法师")]
    Mage, // 低生命值，魔法特化

    #[strum(serialize = "盗贼")]
    Rogue, // 高暴击，擅长速攻

    #[strum(serialize = "女猎手")]
    Huntress, // 远程专家，自然亲和
}

impl Class {
    // === 基础属性 ===

    /// 初始生命值（SPD标准值）
    pub fn base_hp(&self) -> u32 {
        match self {
            Class::Warrior => 30,
            Class::Mage => 20,
            Class::Rogue => 25,
            Class::Huntress => 22,
        }
    }

    /// 每级生命值成长
    pub fn hp_per_level(&self) -> u32 {
        match self {
            Class::Warrior => 5,
            Class::Mage => 3,
            Class::Rogue => 4,
            Class::Huntress => 4,
        }
    }

    /// 基础攻击修正
    pub fn attack_mod(&self) -> f32 {
        match self {
            Class::Warrior => 1.0,
            Class::Mage => 0.9,
            Class::Rogue => 1.0,
            Class::Huntress => 1.0,
        }
    }

    /// 暴击率加成（SPD核心机制）
    pub fn crit_mod(&self) -> f32 {
        match self {
            Class::Warrior => 0.05,  // 5%基础
            Class::Mage => 0.0,      // 无加成
            Class::Rogue => 0.15,    // 15%加成
            Class::Huntress => 0.07, // 7%加成
        }
    }

    /// 基础防御修正
    pub fn defense_mod(&self) -> f32 {
        match self {
            Class::Warrior => 1.0,
            Class::Mage => 0.8,
            Class::Rogue => 0.9,
            Class::Huntress => 1.0,
        }
    }

    // === 成长系统 ===

    /// 每级攻击力成长
    pub fn attack_per_level(&self) -> u32 {
        match self {
            Class::Warrior => 1,
            Class::Mage => 1,
            Class::Rogue => 1,
            Class::Huntress => 1,
        }
    }

    /// 每级防御力成长
    pub fn defense_per_level(&self) -> u32 {
        match self {
            Class::Warrior => 1,
            Class::Mage => 0,
            Class::Rogue => 1,
            Class::Huntress => 1,
        }
    }

    // === 初始装备 ===

    /// 职业初始装备（SPD标准配置）
    pub fn starting_kit(&self) -> Vec<Item> {
        match self {
            Class::Warrior => vec![
                Item::new(ItemKind::Weapon(Weapon::new(1, WeaponKind::Sword))),
                Item::new(ItemKind::Armor(Armor::new(2))),
                Item::new(ItemKind::Potion(PotionKind::Healing.into())), // 治疗药水
            ],
            Class::Mage => vec![
                Item::new(ItemKind::Weapon(Weapon::new(1, WeaponKind::Sword))),
                Item::new(ItemKind::Armor(Armor::new(1))),
                Item::new(ItemKind::Scroll(ScrollKind::Upgrade.into())), // 魔法卷轴
            ],
            Class::Rogue => vec![
                Item::new(ItemKind::Weapon(Weapon::new(1, WeaponKind::Dagger))),
                Item::new(ItemKind::Armor(Armor::new(1))),
                Item::new(ItemKind::Potion(PotionKind::Invisibility.into())), // 隐身药水
            ],
            Class::Huntress => vec![
                Item::new(ItemKind::Weapon(Weapon::new(1, WeaponKind::Sword))),
                Item::new(ItemKind::Armor(Armor::new(1))),
                Item::new(ItemKind::Seed(Seed::random_new())), // 自然种子
            ],
        }
    }

    // === 职业特性 ===

    /// 获取职业描述（SPD特色）
    pub fn description(&self) -> &'static str {
        match self {
            Class::Warrior => "坚韧的战士，攻守平衡",
            Class::Mage => "智慧的法师，魔法大师",
            Class::Rogue => "敏捷的盗贼，暴击专家",
            Class::Huntress => "精准的猎手，远程王者",
        }
    }
}

// 为每个职业保留子模块（供未来扩展）
mod warrior_impl {
    use super::*;

    /// 战士特有逻辑
    impl Class {
        pub fn warrior_rage_bonus(&self) -> f32 {
            if *self == Class::Warrior { 1.1 } else { 1.0 }
        }
    }
}
