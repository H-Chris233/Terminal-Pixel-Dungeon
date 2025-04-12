// src/hero/src/class/class.rs

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub mod huntress;
pub mod mage;
pub mod rogue;
pub mod warrior;

use crate::*;

use items::{armor::Armor, weapon::Weapon, weapon::WeaponKind};
use items::{Item, ItemKind};

/// 英雄职业枚举
#[derive(
    Default, Clone, Debug, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize, EnumIter,
)]
pub enum Class {
    #[default]
    Warrior, // 战士（高生命值，中等攻击，擅长近战）

    Mage,     // 法师（低生命值，高魔法伤害）
    Rogue,    // 盗贼（中等生命值，高暴击率）
    Huntress, // 女猎手（远程攻击，中等属性）
}

impl Class {
    /// 获取职业的基础生命值
    pub fn base_hp(&self) -> u32 {
        match self {
            Class::Warrior => 30,
            Class::Mage => 20,
            Class::Rogue => 25,
            Class::Huntress => 22,
        }
    }

    /// 获取职业的攻击修正系数
    pub fn attack_mod(&self) -> f32 {
        match self {
            Class::Warrior => 1.1,
            Class::Mage => 0.9,
            Class::Rogue => 1.0,
            Class::Huntress => 1.05,
        }
    }

    /// 获取职业的暴击率修正
    pub fn crit_mod(&self) -> f32 {
        match self {
            Class::Warrior => 1.0,
            Class::Mage => 0.8,
            Class::Rogue => 1.3,
            Class::Huntress => 1.1,
        }
    }

    /// 获取职业的防御修正
    pub fn defense_mod(&self) -> f32 {
        match self {
            Class::Warrior => 1.2,
            Class::Mage => 0.7,
            Class::Rogue => 0.9,
            Class::Huntress => 1.0,
        }
    }

    /// 获取职业的初始装备（使用tier系统）
    pub fn starting_kit(&self) -> Vec<Item> {
        match self {
            Class::Warrior => vec![
                Item::new(
                    ItemKind::Weapon(Weapon::new(1, WeaponKind::Sword)), // 第1级武器
                    "战士的初始武器",
                ),
                Item::new(
                    ItemKind::Armor(Armor::new(3)), // 链甲(第3级)
                    "战士的初始护甲",
                ),
            ],
            Class::Mage => vec![
                Item::new(
                    ItemKind::Weapon(Weapon::new(1, WeaponKind::Sword)), // 第1级武器
                    "法师的初始武器",
                ),
                Item::new(
                    ItemKind::Armor(Armor::new(1)), // 布甲(第1级)
                    "法师的初始护甲",
                ),
            ],
            Class::Rogue => vec![
                Item::new(
                    ItemKind::Weapon(Weapon::new(1, WeaponKind::Dagger)), // 第1级武器
                    "盗贼的初始武器",
                ),
                Item::new(
                    ItemKind::Armor(Armor::new(2)), // 皮甲(第2级)
                    "盗贼的初始护甲",
                ),
            ],
            Class::Huntress => vec![
                Item::new(
                    ItemKind::Weapon(Weapon::new(1, WeaponKind::Sword)), // 第1级武器
                    "女猎手的初始武器",
                ),
                Item::new(
                    ItemKind::Armor(Armor::new(2)), // 皮甲(第2级)
                    "女猎手的初始护甲",
                ),
            ],
        }
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Class::Warrior => "战士",
                Class::Mage => "法师",
                Class::Rogue => "盗贼",
                Class::Huntress => "女猎手",
            }
        )
    }
}
