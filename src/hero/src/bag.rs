// src/hero/src/bag.rs
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

use crate::HeroError;

pub mod equipment;
pub mod inventory;

use crate::bag::{
    equipment::{EquipError, Equipment},
    inventory::{Inventory, InventoryError},
};
use items::{
    armor::Armor,
    food::Food,
    misc::{MiscItem, MiscKind},
    potion::Potion,
    ring::Ring,
    scroll::{Scroll, ScrollKind},
    seed::Seed,
    stone::Stone,
    wand::Wand,
    weapon::Weapon,
    Item, ItemKind,
};

/// Complete bag system matching Shattered PD mechanics
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
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
    #[error("背包已满")]
    InventoryFull,
    #[error("Invalid item index")]
    InvalidIndex,
    #[error("Item cannot be used")]
    CannotUseItem,
    #[error("Item cannot be equipped")]
    CannotEquipItem,
}

impl Bag {
    /// Create new bag with default capacities
    pub fn new() -> Self {
        Self {
            gold: 0,
            equipment: Equipment::new(),
            weapons: Inventory::new(4),
            armors: Inventory::new(3),
            potions: Inventory::new(8),
            scrolls: Inventory::new(8),
            wands: Inventory::new(4),
            rings: Inventory::new(4),
            seeds: Inventory::new(4),
            stones: Inventory::new(4),
            food: Inventory::new(4),
            misc: Inventory::new(4),
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

    /// Unified item usage interface
    pub fn use_item(&mut self, index: usize) -> Result<Item, BagError> {
        let item = self.get_item_by_index(index)?;

        match &item.kind {
            ItemKind::Potion(_) => {
                let potion = self.potions.remove(index)?;
                Ok(Item::from(potion))
            }
            ItemKind::Scroll(_) => {
                let scroll = self.scrolls.remove(index)?;
                Ok(Item::from(scroll))
            }
            ItemKind::Food(_) => {
                let food = self.food.remove(index)?;
                Ok(Item::from(food))
            }
            ItemKind::Seed(_) => {
                let seed = self.seeds.remove(index)?;
                Ok(Item::from(seed))
            }
            _ => Err(BagError::CannotUseItem),
        }
    }

    /// Unified equipment interface
    pub fn equip_item(&mut self, index: usize, strength: u8) -> Result<Option<Item>, BagError> {
        let item = self.get_item_by_index(index)?;

        match &item.kind {
            ItemKind::Weapon(weapon) => {
                let old_weapon = self.equipment.equip_weapon(weapon.clone(), strength)?;
                self.remove_item(index)?;
                if let Some(w) = old_weapon {
                    self.add_item(w.into())
                        .map_err(|_| HeroError::InventoryFull)?;
                }
                Ok(old_weapon.map(Item::from))
            }
            ItemKind::Armor(armor) => {
                let old_armor = self.equipment.equip_armor(armor.clone(), strength)?;
                self.remove_item(index)?;
                Ok(old_armor.map(Item::from))
            }
            ItemKind::Ring(ring) => {
                let slot = self.find_available_ring_slot();
                let old_ring = self.equipment.equip_ring(ring.clone(), slot)?;
                self.remove_item(index)?;
                Ok(old_ring.map(Item::from))
            }
            _ => Err(BagError::CannotEquipItem),
        }
    }

    /// Weapon upgrade system
    pub fn upgrade_weapon(&mut self) -> Result<(), BagError> {
        let scroll_idx = self.find_upgrade_scroll()?;
        self.scrolls.remove(scroll_idx)?;
        self.equipment.upgrade_weapon()?;
        Ok(())
    }

    /// Find first available upgrade scroll
    fn find_upgrade_scroll(&self) -> Result<usize, BagError> {
        self.scrolls
            .find(|s| matches!(s.kind, ScrollKind::Upgrade))
            .ok_or(BagError::NoUpgradeScroll)
    }

    /// Find available ring slot (0 or 1)
    fn find_available_ring_slot(&self) -> usize {
        if self.equipment.rings[0].is_none() {
            0
        } else {
            1
        }
    }

    /// Get item by global index across all inventories
    fn get_item_by_index(&self, index: usize) -> Result<Item, BagError> {
        let mut idx = index;

        // Check weapons
        if idx < self.weapons.len() {
            return self
                .weapons
                .get(idx)
                .map(|w| Item::from(w.clone()))
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.weapons.len();

        // Check armors
        if idx < self.armors.len() {
            return self
                .armors
                .get(idx)
                .map(|a| Item::from(a.clone()))
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.armors.len();

        // Continue with other inventories...
        if idx < self.potions.len() {
            return self
                .potions
                .get(idx)
                .map(|p| Item::from(p.clone()))
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.potions.len();

        if idx < self.scrolls.len() {
            return self
                .scrolls
                .get(idx)
                .map(|s| Item::from(s.clone()))
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.scrolls.len();

        if idx < self.rings.len() {
            return self
                .rings
                .get(idx)
                .map(|r| Item::from(r.clone()))
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.rings.len();

        if idx < self.food.len() {
            return self
                .food
                .get(idx)
                .map(|f| Item::from(f.clone()))
                .ok_or(BagError::InvalidIndex);
        }

        Err(BagError::InvalidIndex)
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

    pub fn remove_item(&mut self, index: usize) -> Result<(), BagError> {
        let mut idx = index;

        if idx < self.weapons.len() {
            return self.weapons.remove(idx).map(|_| ()).map_err(Into::into);
        }
        idx -= self.weapons.len();

        if idx < self.armors.len() {
            return self.armors.remove(idx).map(|_| ()).map_err(Into::into);
        }
        idx -= self.armors.len();

        if idx < self.potions.len() {
            return self.potions.remove(idx).map(|_| ()).map_err(Into::into);
        }
        idx -= self.potions.len();

        if idx < self.scrolls.len() {
            return self.scrolls.remove(idx).map(|_| ()).map_err(Into::into);
        }
        idx -= self.scrolls.len();

        if idx < self.rings.len() {
            return self.rings.remove(idx).map(|_| ()).map_err(Into::into);
        }
        idx -= self.rings.len();

        if idx < self.food.len() {
            return self.food.remove(idx).map(|_| ()).map_err(Into::into);
        }

        Err(BagError::InvalidIndex)
    }

    /// Sorting functions
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

    pub fn rings(&self) -> &Inventory<Ring> {
        &self.rings
    }

    pub fn food(&self) -> &Inventory<Food> {
        &self.food
    }
}

/* === STRUCTS === */

/*
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
*/

/* === ENUMS === */

/*
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
    #[error("Item cannot be used")]
    CannotUseItem,
    #[error("Item cannot be equipped")]
    CannotEquipItem,
}
*/

/* === IMPL BLOCK FOR Bag === */

/*
impl Bag {
    pub fn new() -> Self
    pub fn add_gold(&mut self, amount: u32)
    pub fn spend_gold(&mut self, amount: u32) -> Result<(), BagError>
    pub fn use_item(&mut self, index: usize) -> Result<Item, BagError>
    pub fn equip_item(&mut self, index: usize, strength: u8) -> Result<Option<Item>, BagError>
    pub fn upgrade_weapon(&mut self) -> Result<(), BagError>
    fn find_upgrade_scroll(&self) -> Result<usize, BagError>
    fn find_available_ring_slot(&self) -> usize
    fn get_item_by_index(&self, index: usize) -> Result<Item, BagError>
    pub fn add_item(&mut self, item: Item) -> Result<(), BagError>
    fn remove_item(&mut self, index: usize) -> Result<(), BagError>
    pub fn sort_weapons(&mut self)
    pub fn sort_armors(&mut self)
    pub fn sort_rings(&mut self)
    pub fn gold(&self) -> u32
    pub fn weapons(&self) -> &Inventory<Weapon>
    pub fn armors(&self) -> &Inventory<Armor>
    pub fn potions(&self) -> &Inventory<Potion>
    pub fn scrolls(&self) -> &Inventory<Scroll>
    pub fn equipment(&self) -> &Equipment
    pub fn rings(&self) -> &Inventory<Ring>
    pub fn food(&self) -> &Inventory<Food>
}
*/

/* === TRAIT IMPLEMENTATIONS === */

/*
impl Clone for Bag
impl Debug for Bag
impl Encode for Bag
impl Decode for Bag
impl Serialize for Bag
impl Deserialize for Bag
*/
