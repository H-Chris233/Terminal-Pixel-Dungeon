//src/items/src/lib.rs
#![allow(dead_code)]
#![allow(unused)]

const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::legacy().with_variable_int_encoding(); // 添加变长整数编码

use bincode::serde::encode_to_vec;
use bincode::{Decode, Encode};
use serde::de::DeserializeOwned;
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
    pub fn new(kind: ItemKind) -> Self {
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
            description: "...".to_string(),
            quantity: 1,
            x: -1,
            y: -1,
        }
    }
    /// 获取显示名称（还原游戏内命名规则）
    pub fn name(&self) -> String {
        match &self.kind {
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
        }
    }

    /// 是否为消耗品（精确匹配游戏机制）
    pub fn is_consumable(&self) -> bool {
        matches!(
            &self.kind,
            ItemKind::Potion(_) | ItemKind::Scroll(_) | ItemKind::Food(_)
        )
    }

    /// 判断物品是否需要鉴定（精确匹配游戏机制）
    pub fn needs_identify(&self) -> bool {
        match &self.kind {
            ItemKind::Potion(p) => !p.identified,
            ItemKind::Scroll(s) => !s.identified,
            ItemKind::Ring(r) => !r.identified,
            ItemKind::Wand(w) => !w.identified,
            ItemKind::Weapon(w) => !w.identified,
            ItemKind::Armor(a) => !a.identified,
            _ => false, // 其他物品默认不需要鉴定
        }
    }

    /// 获取物品价值（用于商店系统）
    pub fn value(&self) -> u32 {
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
    
    pub fn as_weapon(&self) -> &Weapon {
        if let ItemKind::Weapon(ref w) = self.kind { w } 
        else { panic!("Item is not Weapon: {:?}", self.kind) }
    }

    pub fn as_armor(&self) -> &Armor {
        if let ItemKind::Armor(ref a) = self.kind { a } 
        else { panic!("Item is not Armor: {:?}", self.kind) }
    }

    pub fn as_potion(&self) -> &Potion {
        if let ItemKind::Potion(ref p) = self.kind { p } 
        else { panic!("Item is not Potion: {:?}", self.kind) }
    }

    pub fn as_scroll(&self) -> &Scroll {
        if let ItemKind::Scroll(ref s) = self.kind { s } 
        else { panic!("Item is not Scroll: {:?}", self.kind) }
    }

    pub fn as_food(&self) -> &Food {
        if let ItemKind::Food(ref f) = self.kind { f } 
        else { panic!("Item is not Food: {:?}", self.kind) }
    }

    pub fn as_wand(&self) -> &Wand {
        if let ItemKind::Wand(ref w) = self.kind { w } 
        else { panic!("Item is not Wand: {:?}", self.kind) }
    }

    pub fn as_ring(&self) -> &Ring {
        if let ItemKind::Ring(ref r) = self.kind { r } 
        else { panic!("Item is not Ring: {:?}", self.kind) }
    }

    pub fn as_seed(&self) -> &Seed {
        if let ItemKind::Seed(ref s) = self.kind { s } 
        else { panic!("Item is not Seed: {:?}", self.kind) }
    }

    pub fn as_stone(&self) -> &Stone {
        if let ItemKind::Stone(ref s) = self.kind { s } 
        else { panic!("Item is not Stone: {:?}", self.kind) }
    }

    pub fn as_misc(&self) -> &MiscItem {
        if let ItemKind::Misc(ref m) = self.kind { m } 
        else { panic!("Item is not Misc: {:?}", self.kind) }
    }
}

impl Default for Item {
    fn default() -> Self {
        Self {
            kind: ItemKind::Misc(MiscItem::new(MiscKind::Torch)), // 默认使用火炬作为占位物品
            name: "Default Item".to_string(),
            description: "Default item description".to_string(),
            quantity: 1,
            x: -1,
            y: -1,
        }
    }
}

/// 物品特性约束
pub trait ItemTrait:
    PartialEq + Clone + Serialize + std::fmt::Debug + DeserializeOwned + Send + Sync
{
    /// 是否可堆叠（药水/卷轴等可堆叠，武器/护甲不可）
    fn is_stackable(&self) -> bool;

    fn max_stack(&self) -> u32 {
        u32::MAX
    }

    /// 显示名称（用于UI渲染）
    fn display_name(&self) -> String;

    /// 物品分类（用于自动整理）
    fn category(&self) -> ItemCategory;

    /// 排序权重（数值越大排序越前）
    fn sort_value(&self) -> u32;

    fn stacking_id(&self) -> u64;
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
            ItemKind::Weapon(w) => w.is_stackable(),
            ItemKind::Armor(a) => a.is_stackable(),
            ItemKind::Potion(p) => p.is_stackable(),
            ItemKind::Scroll(s) => s.is_stackable(),
            ItemKind::Food(f) => f.is_stackable(),
            ItemKind::Wand(w) => w.is_stackable(),
            ItemKind::Ring(r) => r.is_stackable(),
            ItemKind::Seed(s) => s.is_stackable(),
            ItemKind::Stone(s) => s.is_stackable(),
            ItemKind::Misc(m) => m.is_stackable(),
        }
    }

    fn max_stack(&self) -> u32 {
        match &self.kind {
            ItemKind::Weapon(w) => w.max_stack(),
            ItemKind::Armor(a) => a.max_stack(),
            ItemKind::Potion(p) => p.max_stack(),
            ItemKind::Scroll(s) => s.max_stack(),
            ItemKind::Food(f) => f.max_stack(),
            ItemKind::Wand(w) => w.max_stack(),
            ItemKind::Ring(r) => r.max_stack(),
            ItemKind::Seed(s) => s.max_stack(),
            ItemKind::Stone(s) => s.max_stack(),
            ItemKind::Misc(m) => m.max_stack(),
        }
    }

    fn stacking_id(&self) -> u64 {
        match &self.kind {
            ItemKind::Weapon(w) => w.stacking_id(),
            ItemKind::Armor(a) => a.stacking_id(),
            ItemKind::Potion(p) => p.stacking_id(),
            ItemKind::Scroll(s) => s.stacking_id(),
            ItemKind::Food(f) => f.stacking_id(),
            ItemKind::Wand(w) => w.stacking_id(),
            ItemKind::Ring(r) => r.stacking_id(),
            ItemKind::Seed(s) => s.stacking_id(),
            ItemKind::Stone(s) => s.stacking_id(),
            ItemKind::Misc(m) => m.stacking_id(),
        }
    }

    fn display_name(&self) -> String {
        match &self.kind {
            ItemKind::Weapon(w) => w.display_name(),
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
            ItemKind::Weapon(w) => w.sort_value(),
            ItemKind::Armor(a) => a.sort_value(),
            ItemKind::Potion(p) => p.sort_value(),
            ItemKind::Scroll(s) => s.sort_value(),
            ItemKind::Wand(w) => w.sort_value(),
            ItemKind::Ring(r) => r.sort_value(),
            ItemKind::Seed(s) => s.sort_value(),
            ItemKind::Stone(s) => s.sort_value(),
            ItemKind::Food(f) => f.sort_value(),
            ItemKind::Misc(m) => m.sort_value(),
        }
    }
}
