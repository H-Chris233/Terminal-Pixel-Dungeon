//src/hero/bag/bag.rs
use serde::Serialize;
use serde::Deserialize;

mod bag;
mod equipment;
mod inventory;

use super::{equipment::Equipment, inventory::Inventory};
use crate::hero::bag::BagError;
use crate::items::{
    armor::Armor, food::Food, misc::MiscItem, potion::Potion, ring::Ring, scroll::Scroll,
    seed::Seed, wand::Wand, weapon::Weapon,
};
use crate::items::items::Item;

/// 符合游戏机制的背包系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bag {
    equipment: Equipment,
    general: Inventory<MiscItem>,
    consumables: Inventory<Potion>,
    // ...其他分类
}

impl Bag {
    /// 创建新背包（参考游戏默认容量）
    pub fn new() -> Self {
        Self {
            equipment: Equipment::with_capacity(4), // 武器/护甲/2饰品
            general: Inventory::new(16),            // 常规物品
            consumables: Inventory::new(8),         // 消耗品专用
        }
    }

    /// 智能添加物品（自动分类）
    pub fn add_item(&mut self, item: impl Into<BagItem>) -> Result<(), BagError> {
        match item.into() {
            BagItem::Weapon(w) => self.equipment.equip_weapon(w),
            BagItem::Potion(p) => self.consumables.add(p),
            // ...其他类型处理
        }
    }

    /// 升级装备中的武器（带材料检查）
    pub fn upgrade_weapon(&mut self, material: &Item) -> Result<(), BagError> {
        if !material.is_upgrade_material() {
            return Err(BagError::InvalidMaterial);
        }
        self.equipment.upgrade_weapon()
    }
}

/// 统一背包物品枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BagItem {
    Weapon(Weapon),
    Armor(Armor),
    Potion(Potion),
    // ...其他类型
}
