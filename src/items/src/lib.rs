//src/items/src/lib.rs
#![allow(dead_code)]
#![allow(unused)]

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub use crate::armor::Armor;
pub use crate::food::Food;
pub use crate::misc::MiscItem;
pub use crate::potion::Potion;
pub use crate::ring::Ring;
pub use crate::scroll::Scroll;
pub use crate::seed::Seed;
pub use crate::stone::Stone;
pub use crate::wand::Wand;
pub use crate::weapon::Weapon;



use crate::food::FoodKind;
use crate::misc::MiscKind;
use crate::potion::PotionKind;
use crate::ring::RingKind;
use crate::scroll::ScrollKind;
use crate::seed::SeedKind;
use crate::stone::StoneKind;
use crate::wand::WandKind;
use crate::weapon::WeaponKind;

pub mod armor;
pub mod food;
pub mod misc;
pub mod potion;
pub mod ring;
pub mod scroll;
pub mod seed;
pub mod stone;
pub mod wand;
pub mod weapon;

/// 基础物品结构（还原游戏内属性）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub name: String,
    pub description: String,
    pub quantity: u32, // 堆叠数量
    pub x: i32,
    pub y: i32,
}

/// 物品类型枚举（与Shattered PD完全一致）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum ItemKind {
    Weapon(Weapon), // 近战武器
    Armor(Armor),   // 护甲
    Potion(Potion), // 药水（12种）
    Scroll(Scroll), // 卷轴（10种）
    Food(Food),     // 食物（3种）
    Wand(Wand),     // 法杖（8种）
    Ring(Ring),     // 戒指（10种）
    Seed(Seed),     // 种子（8种）
    Stone(Stone),   // 魔法石（6种）
    Misc(MiscItem), // 杂项（钥匙等）
}

impl Item {
    pub fn new(kind: ItemKind, description: &str) -> Self {
        let name = match &kind {
            ItemKind::Weapon(w) => w.name.clone(),
            ItemKind::Armor(a) => a.name().clone(),
            ItemKind::Potion(p) => p.name(),
            ItemKind::Scroll(s) => s.name(),
            ItemKind::Food(f) => f.name(),
            ItemKind::Wand(w) => w.name(),
            ItemKind::Ring(r) => r.name(),
            ItemKind::Seed(s) => s.name(),
            ItemKind::Stone(s) => s.name(),
            ItemKind::Misc(m) => m.name().clone(),
        };

        Self {
            kind,
            name,
            description: description.to_string(),
            quantity: 1,
            x: -1,
            y: -1,
        }
    }
    /// 获取显示名称（还原游戏内命名规则）
    pub fn name(&self) -> String {
        let name = match &self.kind {
            ItemKind::Weapon(w) => w.name.clone(),
            ItemKind::Armor(a) => a.name().clone(),
            ItemKind::Potion(p) => p.name(),
            ItemKind::Scroll(s) => s.name(),
            ItemKind::Food(f) => f.name(),
            ItemKind::Wand(w) => w.name(),
            ItemKind::Ring(r) => r.name(),
            ItemKind::Seed(s) => s.name(),
            ItemKind::Stone(s) => s.name(),
            ItemKind::Misc(m) => m.name().clone(),
        };
        name
    }

    /// 是否为消耗品（精确匹配游戏机制）
    pub fn is_consumable(&self) -> bool {
        matches!(
            &self.kind,
            ItemKind::Potion(_) | ItemKind::Scroll(_) | ItemKind::Food(_)
        )
    }

    /// 物品是否已鉴定
    pub fn is_identified(&self) -> bool {
        match &self.kind {
            ItemKind::Potion(p) => p.identified,
            ItemKind::Scroll(s) => s.identified,
            ItemKind::Ring(_) => true, // 戒指需要装备才知效果
            _ => todo!(),              // 其他物品默认已鉴定
        }
    }

    /// 获取物品价值（用于商店系统）
    pub fn value(&self) -> usize {
        match &self.kind {
            ItemKind::Weapon(w) => w.value(),
            ItemKind::Armor(a) => a.value(),
            ItemKind::Potion(p) => p.value(),
            ItemKind::Scroll(s) => s.value(),
            ItemKind::Food(f) => f.value(),
            ItemKind::Wand(w) => w.value(),
            ItemKind::Ring(r) => r.value(),
            ItemKind::Seed(s) => s.value(),
            ItemKind::Stone(s) => s.value(),
            ItemKind::Misc(m) => m.value(),
        }
    }
}

/// 物品特性约束
pub trait ItemTrait: PartialEq + Clone + Serialize + std::fmt::Debug {
    /// 是否可堆叠（药水/卷轴等可堆叠，武器/护甲不可）
    fn is_stackable(&self) -> bool;

    /// 显示名称（用于UI渲染）
    fn display_name(&self) -> String;

    /// 物品分类（用于自动整理）
    fn category(&self) -> ItemCategory;

    /// 排序权重（数值越大排序越前）
    fn sort_value(&self) -> u32;
}

/// 物品分类（完全匹配游戏内分类）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ItemCategory {
    Weapon, // 武器
    Armor,  // 护甲
    Potion, // 药水
    Scroll, // 卷轴
    Wand,   // 法杖
    Ring,   // 戒指
    Seed,   // 种子
    Stone,  // 魔法石
    Food,   // 食物
    Misc,   // 杂项
}

impl ItemTrait for Item {
    fn is_stackable(&self) -> bool {
        match &self.kind {
            ItemKind::Potion(_) | ItemKind::Scroll(_) | ItemKind::Food(_) | ItemKind::Seed(_) => {
                true
            }
            _ => false,
        }
    }

    fn display_name(&self) -> String {
        match &self.kind {
            ItemKind::Weapon(w) => w.name.clone(),
            ItemKind::Armor(a) => a.name(),
            ItemKind::Potion(p) => p.name(),
            ItemKind::Scroll(s) => s.name(),
            ItemKind::Food(f) => f.name(),
            ItemKind::Wand(w) => w.name(),
            ItemKind::Ring(r) => r.name(),
            ItemKind::Seed(s) => s.name(),
            ItemKind::Stone(s) => s.name(),
            ItemKind::Misc(m) => m.name(),
        }
    }

    fn category(&self) -> ItemCategory {
        match &self.kind {
            ItemKind::Weapon(_) => ItemCategory::Weapon,
            ItemKind::Armor(_) => ItemCategory::Armor,
            ItemKind::Potion(_) => ItemCategory::Potion,
            ItemKind::Scroll(_) => ItemCategory::Scroll,
            ItemKind::Wand(_) => ItemCategory::Wand,
            ItemKind::Ring(_) => ItemCategory::Ring,
            ItemKind::Seed(_) => ItemCategory::Seed,
            ItemKind::Stone(_) => ItemCategory::Stone,
            ItemKind::Food(_) => ItemCategory::Food,
            ItemKind::Misc(_) => ItemCategory::Misc,
        }
    }

    fn sort_value(&self) -> u32 {
        match &self.kind {
            // Weapons sorted by tier then level
            ItemKind::Weapon(w) => (w.tier as u32 * 100) + w.upgrade_level as u32,
            // Armor sorted by tier then level
            ItemKind::Armor(a) => (a.tier as u32 * 100) + a.upgrade_level as u32,
            // Potions sorted by type
            ItemKind::Potion(p) => p.sort_value(),
            // Scrolls sorted by type
            ItemKind::Scroll(s) => s.sort_value(),
            // Wands sorted by level then type
            ItemKind::Wand(w) => (w.level as u32 * 100) + w.sort_value(),
            // Rings sorted by type
            ItemKind::Ring(r) => r.sort_value(),
            // Seeds sorted by type
            ItemKind::Seed(s) => s.sort_value(),
            // Stones sorted by type
            ItemKind::Stone(s) => s.sort_value(),
            // Food sorted by type
            ItemKind::Food(f) => f.sort_value(),
            // Misc items have fixed order
            ItemKind::Misc(m) => m.sort_value(),
        }
    }
}

// Implement ItemTrait for each specific item type

impl ItemTrait for Weapon {
    fn is_stackable(&self) -> bool {
        false
    }
    fn display_name(&self) -> String {
        self.name.clone()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Weapon
    }
    fn sort_value(&self) -> u32 {
        (self.tier as u32 * 100) + self.upgrade_level as u32
    }
}

impl ItemTrait for Armor {
    fn is_stackable(&self) -> bool {
        false
    }
    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Armor
    }
    fn sort_value(&self) -> u32 {
        (self.tier as u32 * 100) + self.upgrade_level as u32
    }
}

impl ItemTrait for Potion {
    fn is_stackable(&self) -> bool {
        true
    }
    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Potion
    }
    fn sort_value(&self) -> u32 {
        match self.kind {
            PotionKind::Healing => 12,
            PotionKind::Experience => 11,
            PotionKind::ToxicGas => 10,
            PotionKind::ParalyticGas => 9,
            PotionKind::LiquidFlame => 8,
            PotionKind::Levitation => 7,
            PotionKind::Invisibility => 6,
            PotionKind::Purity => 5,
            PotionKind::Frost => 4,
            PotionKind::Strength => 3,
            PotionKind::MindVision => 2,
            PotionKind::Haste => 1,
            _ => 0,
        }
    }
}

impl ItemTrait for Scroll {
    fn is_stackable(&self) -> bool {
        true
    }
    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Scroll
    }
    fn sort_value(&self) -> u32 {
        match self.kind {
            ScrollKind::Upgrade => 100,
            ScrollKind::Identify => 90,
            ScrollKind::RemoveCurse => 80,
            // ... other scroll types
            _ => 0,
        }
    }
}

impl ItemTrait for Wand {
    fn is_stackable(&self) -> bool {
        false
    }
    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Wand
    }
    fn sort_value(&self) -> u32 {
        (self.level as u32 * 100)
            + match self.kind {
                WandKind::Disintegration => 100,
                WandKind::Lightning => 90,
                WandKind::Fireblast => 80,
                // ... other wand types
                _ => 0,
            }
    }
}

impl ItemTrait for Ring {
    fn is_stackable(&self) -> bool {
        false
    }
    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Ring
    }
    fn sort_value(&self) -> u32 {
        match self.kind {
            _ => 100,
        }
    }
}

impl ItemTrait for Seed {
    fn is_stackable(&self) -> bool {
        true
    }
    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Seed
    }
    fn sort_value(&self) -> u32 {
        match self.kind {
            _ => 20,
        }
    }
}

impl ItemTrait for Stone {
    fn is_stackable(&self) -> bool {
        false
    }
    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Stone
    }
    fn sort_value(&self) -> u32 {
        match self.kind {
            _ => 30,
        }
    }
}

impl ItemTrait for Food {
    fn is_stackable(&self) -> bool {
        true
    }
    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Food
    }
    fn sort_value(&self) -> u32 {
        match self.kind {
            FoodKind::Ration => 100,
            FoodKind::Pasty => 90,
            FoodKind::MysteryMeat => 80,
            _ => 0,
        }
    }
}

impl ItemTrait for MiscItem {
    fn is_stackable(&self) -> bool {
        match self.kind {
            MiscKind::Gold(_) => true,
            MiscKind::Key => false,
            MiscKind::Torch => true,
            // ... other misc items
            _ => false,
        }
    }

    fn display_name(&self) -> String {
        match self.kind {
            MiscKind::Gold(_) => "金币".to_string(),
            MiscKind::Key => "钥匙".to_string(),
            MiscKind::Torch => "火把".to_string(),
            // ... other misc items
            _ => "杂项".to_string(),
        }
    }

    fn category(&self) -> ItemCategory {
        ItemCategory::Misc
    }

    fn sort_value(&self) -> u32 {
        match self.kind {
            MiscKind::Gold(_) => 100,
            MiscKind::Key => 90,
            MiscKind::Torch => 80,
            // ... other misc items
            _ => 0,
        }
    }
}
