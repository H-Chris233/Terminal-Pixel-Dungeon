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
    armor::Armor, food::Food, misc::MiscItem, potion::Potion, ring::Ring, scroll::Scroll,
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
            ItemKind::Misc(m) => match m {
                MiscItem::Gold(amount) => {
                    self.add_gold(amount);
                    Ok(())
                }
                _ => self.misc.add(m),
            },
        }
        .map_err(Into::into)
    }

    /// Equipment management
    pub fn equip_weapon(&mut self, index: usize, strength: i32) -> Result<(), BagError> {
        let weapon = self.weapons.remove(index)?;
        if let Some(old) = self.equipment.equip_weapon(weapon, strength)? {
            self.weapons.add(old)?;
        }
        Ok(())
    }

    pub fn equip_armor(&mut self, index: usize, strength: i32) -> Result<(), BagError> {
        let armor = self.armors.remove(index)?;
        if let Some(old) = self.equipment.equip_armor(armor, strength)? {
            self.armors.add(old)?;
        }
        Ok(())
    }

    pub fn equip_ring(&mut self, index: usize, slot: usize) -> Result<(), BagError> {
        let ring = self.rings.remove(index)?;
        if let Some(old) = self.equipment.equip_ring(ring, slot)? {
            self.rings.add(old)?;
        }
        Ok(())
    }

    /// Item usage
    pub fn use_potion(&mut self, index: usize) -> Result<Potion, BagError> {
        self.potions.remove(index).map_err(Into::into)
    }

    pub fn use_scroll(&mut self, index: usize) -> Result<Scroll, BagError> {
        self.scrolls.remove(index).map_err(Into::into)
    }

    /// Weapon upgrade system
    pub fn upgrade_weapon(&mut self) -> Result<(), BagError> {
        let scroll_idx = self
            .scrolls
            .find(|s| matches!(s, Scroll::Upgrade))
            .ok_or(BagError::NoUpgradeScroll)?;
        self.scrolls.remove(scroll_idx)?;
        self.equipment.upgrade_weapon()?;
        Ok(())
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
