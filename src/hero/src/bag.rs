// src/hero/src/bag.rs
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

pub mod equipment;
pub mod inventory;

use crate::bag::{
    equipment::{EquipError, Equipment},
    inventory::{Inventory, InventoryError},
};
use items::{
    armor::Armor, food::Food, misc::{MiscItem, MiscKind}, potion::Potion, ring::Ring, scroll::{Scroll, ScrollKind},
    seed::Seed, stone::Stone, wand::Wand, weapon::Weapon, Item, ItemKind,
};

/// Complete bag system matching Shattered PD mechanics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bag {
    gold: u32,
    equipment: Equipment,
    weapons: Inventory<Weapon>,
    armors: Inventory<Armor>,
    potions: Inventory<Potion>,
    scrolls: Inventory<Scroll>,
    wands: Inventory<Wand>,
    rings: Inventory<Ring>,
    seeds: Inventory<Seed>,
    stones: Inventory<Stone>,
    food: Inventory<Food>,
    misc: Inventory<MiscItem>,
}

/// Bag-specific error types
#[derive(Debug, Error)]
pub enum BagError {
    #[error(transparent)]
    Inventory(#[from] InventoryError),
    #[error(transparent)]
    Equipment(#[from] EquipError),
    #[error("Not enough gold")]
    NotEnoughGold,
    #[error("No weapon equipped")]
    NoWeaponEquipped,
    #[error("No upgrade scroll available")]
    NoUpgradeScroll,
    #[error("Invalid item index")]
    InvalidIndex,
}

impl Bag {
    /// Create new bag with default capacities
    pub fn new() -> Self {
        Self {
            gold: 0,
            equipment: Equipment::new(),
            weapons: Inventory::new(4), // Weapons
            armors: Inventory::new(3),  // Armor
            potions: Inventory::new(8), // Potions
            scrolls: Inventory::new(8), // Scrolls
            wands: Inventory::new(4),   // Wands
            rings: Inventory::new(4),   // Rings
            seeds: Inventory::new(4),   // Seeds
            stones: Inventory::new(4),  // Stones
            food: Inventory::new(4),    // Food
            misc: Inventory::new(4),    // Miscellaneous
        }
    }

    /// Gold management
    pub fn add_gold(&mut self, amount: u32) {
        self.gold += amount;
    }

    pub fn spend_gold(&mut self, amount: u32) -> Result<(), BagError> {
        if self.gold >= amount {
            self.gold -= amount;
            Ok(())
        } else {
            Err(BagError::NotEnoughGold)
        }
    }
    
    /// 通用装备方法 - 如果槽位已有装备则自动交换
    pub fn equip_item(&mut self, item_index: usize, strength: u8) -> Result<(), BagError> {
        let item = self.get_item_by_index(item_index)?;
        
        match &item.kind {
            ItemKind::Weapon(weapon) => {
                let old_weapon = self.equipment.equip_weapon(weapon.clone(), strength)?;
                self.remove_item(item_index)?; // 移除新装备
                if let Some(old) = old_weapon {
                    self.add_item(Item::from(old))?; // 添加旧装备回背包
                }
            }
            ItemKind::Armor(armor) => {
                let old_armor = self.equipment.equip_armor(armor.clone(), strength)?;
                self.remove_item(item_index)?;
                if let Some(old) = old_armor {
                    self.add_item(Item::from(old))?;
                }
            }
            ItemKind::Ring(ring) => {
                // 自动选择第一个可用槽位
                let slot = self.find_available_ring_slot();
                let old_ring = self.equipment.equip_ring(ring.clone(), slot)?;
                self.remove_item(item_index)?;
                if let Some(old) = old_ring {
                    self.add_item(Item::from(old))?;
                }
            }
            _ => return Err(BagError::Equipment(EquipError::IncompatibleType)),
        }
        
        Ok(())
    }

    /// 装备武器 - 自动交换
    pub fn equip_weapon(&mut self, index: usize, strength: u8) -> Result<(), BagError> {
        let weapon = self.weapons.remove(index)?;
        if let Some(old) = self.equipment.equip_weapon(weapon, strength)? {
            self.weapons.add(old)?;
        }
        Ok(())
    }

    /// 装备护甲 - 自动交换
    pub fn equip_armor(&mut self, index: usize, strength: u8) -> Result<(), BagError> {
        let armor = self.armors.remove(index)?;
        if let Some(old) = self.equipment.equip_armor(armor, strength)? {
            self.armors.add(old)?;
        }
        Ok(())
    }

    /// 装备戒指 - 自动交换
    pub fn equip_ring(&mut self, index: usize, slot: Option<usize>) -> Result<(), BagError> {
        let ring = self.rings.remove(index)?;
        let slot = slot.unwrap_or_else(|| self.find_available_ring_slot());
        
        if let Some(old) = self.equipment.equip_ring(ring, slot)? {
            self.rings.add(old)?;
        }
        Ok(())
    }

    /// 查找可用的戒指槽位
    fn find_available_ring_slot(&self) -> usize {
        if self.equipment.rings[0].is_none() {
            0
        } else if self.equipment.rings[1].is_none() {
            1
        } else {
            0 // 默认替换第一个槽位
        }
    }

    /// 根据索引获取物品
    fn get_item_by_index(&self, index: usize) -> Result<&Item, BagError> {
        // 实现根据索引查找物品的逻辑
        // 这里需要你根据实际的数据结构来实现
        unimplemented!()
    }

    /// 根据索引移除物品
    fn remove_item(&mut self, index: usize) -> Result<(), BagError> {
        // 实现根据索引移除物品的逻辑
        // 这里需要你根据实际的数据结构来实现
        unimplemented!()
    }

    /// Add item with automatic categorization
    pub fn add_item(&mut self, item: Item) -> Result<(), BagError> {
        match item.kind {
            ItemKind::Weapon(w) => self
                .weapons
                .add_sorted(w, |a, b| b.upgrade_level.cmp(&a.upgrade_level)),
            ItemKind::Armor(a) => self
                .armors
                .add_sorted(a, |a, b| b.upgrade_level.cmp(&a.upgrade_level)),
            ItemKind::Potion(p) => self.potions.add(p),
            ItemKind::Scroll(s) => self.scrolls.add(s),
            ItemKind::Wand(w) => self.wands.add_sorted(w, |a, b| b.level.cmp(&a.level)),
            ItemKind::Ring(r) => self.rings.add(r),
            ItemKind::Seed(s) => self.seeds.add(s),
            ItemKind::Stone(s) => self.stones.add(s),
            ItemKind::Food(f) => self.food.add(f),
            ItemKind::Misc(m) => match m.kind {
                MiscKind::Gold(amount) => {
                    self.add_gold(amount);
                    Ok(())
                }
                _ => self.misc.add(m),
            },
        }
        .map_err(Into::into)
    }

    
    /// Item usage
    pub fn use_potion(&mut self, index: usize) -> Result<Potion, BagError> {
        self.potions.remove(index).map_err(Into::into)
    }

    pub fn use_scroll(&mut self, index: usize) -> Result<Scroll, BagError> {
        self.scrolls.remove(index).map_err(Into::into)
    }
    
    fn handle_equip_error(&mut self, error: EquipError, item: Item) -> BagError {
        match error {
            EquipError::SlotFull => {
                // 自动尝试将当前装备放回背包
                BagError::Equipment(error)
            }
            EquipError::StrengthRequirement => {
                // 可以记录力量不足的具体数值
                BagError::Equipment(error)
            }
            EquipError::CursedItem => {
                // 记录诅咒装备信息
                BagError::Equipment(error)
            }
            _ => BagError::Equipment(error),
        }
    }

    /// Weapon upgrade system
    pub fn upgrade_weapon(&mut self) -> Result<(), BagError> {
        let scroll_idx = self
            .scrolls
            .find(|s| matches!(s.kind, ScrollKind::Upgrade))
            .ok_or(BagError::NoUpgradeScroll)?;
        self.scrolls.remove(scroll_idx)?;
        self.equipment.upgrade_weapon()?;
        Ok(())
    }
    
    pub fn sort_weapons(&mut self) {
        self.weapons.sort_by(|a, b| {
            b.upgrade_level
                .cmp(&a.upgrade_level)
                .then_with(|| b.damage.0.cmp(&a.damage.0))
        });
    }

    pub fn sort_armors(&mut self) {
        self.armors.sort_by(|a, b| {
            b.upgrade_level
                .cmp(&a.upgrade_level)
                .then_with(|| b.defense.cmp(&a.defense))
        });
    }

    pub fn sort_rings(&mut self) {
        self.rings.sort_by(|a, b| b.level.cmp(&a.level));
    }

    /// Getters for UI
    pub fn gold(&self) -> u32 {
        self.gold
    }
    pub fn weapons(&self) -> &Inventory<Weapon> {
        &self.weapons
    }
    pub fn armors(&self) -> &Inventory<Armor> {
        &self.armors
    }
    pub fn potions(&self) -> &Inventory<Potion> {
        &self.potions
    }
    pub fn scrolls(&self) -> &Inventory<Scroll> {
        &self.scrolls
    }
    pub fn equipment(&self) -> &Equipment {
        &self.equipment
    }
    // ... other getters ...
}
