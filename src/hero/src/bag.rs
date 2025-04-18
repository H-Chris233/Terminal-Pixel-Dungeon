
// src/hero/src/bag.rs
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

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

// 子模块定义
pub mod equipment; // 装备管理
pub mod inventory; // 物品库存管理

use equipment::{EquipError, Equipment};
use inventory::{Inventory, InventoryError};

/// 完整的背包系统（遵循破碎的像素地牢机制）
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Bag {
    gold: u32,                  // 金币数量
    equipment: Equipment,       // 已装备的物品
    weapons: Inventory<Weapon>, // 武器库存
    armors: Inventory<Armor>,   // 护甲库存
    potions: Inventory<Potion>, // 药水库存
    scrolls: Inventory<Scroll>, // 卷轴库存
    wands: Inventory<Wand>,     // 法杖库存
    rings: Inventory<Ring>,     // 戒指库存
    seeds: Inventory<Seed>,     // 种子库存
    stones: Inventory<Stone>,   // 宝石库存
    food: Inventory<Food>,      // 食物库存
    misc: Inventory<MiscItem>,  // 杂项物品
}

/// 背包特定的错误类型
#[derive(Debug, Error)]
pub enum BagError {
    #[error(transparent)]
    Inventory(#[from] InventoryError), // 库存错误
    #[error(transparent)]
    Equipment(#[from] EquipError), // 装备错误
    #[error("金币不足")]
    NotEnoughGold, // 金币不足
    #[error("没有装备武器")]
    NoWeaponEquipped, // 无武器
    #[error("没有升级卷轴")]
    NoUpgradeScroll, // 无升级卷轴
    #[error("背包已满")]
    InventoryFull, // 背包满
    #[error("无效的物品索引")]
    InvalidIndex, // 无效索引
    #[error("物品无法使用")]
    CannotUseItem, // 不可使用
    #[error("物品无法装备")]
    CannotEquipItem, // 不可装备
}

impl Bag {
    /// 创建具有默认容量的新背包
    pub fn new() -> Self {
        Self {
            gold: 0,
            equipment: Equipment::new(),
            weapons: Inventory::new(4), // 武器容量4
            armors: Inventory::new(3),  // 护甲容量3
            potions: Inventory::new(8), // 药水容量8
            scrolls: Inventory::new(8), // 卷轴容量8
            wands: Inventory::new(4),   // 法杖容量4
            rings: Inventory::new(4),   // 戒指容量4
            seeds: Inventory::new(4),   // 种子容量4
            stones: Inventory::new(4),  // 宝石容量4
            food: Inventory::new(4),    // 食物容量4
            misc: Inventory::new(4),    // 杂项容量4
        }
    }

    /* ================== 金币管理 ================== */
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

    /* ================== 物品使用 ================== */
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

    /* ================== 装备系统 ================== */
    pub fn equip_item(&mut self, index: usize, strength: u8) -> Result<Option<Item>, BagError> {
        let item = self.get_item_by_index(index)?;

        match &item.kind {
            ItemKind::Weapon(weapon) => {
                let old_weapon = self.equipment.equip_weapon(weapon.clone(), strength)?;
                self.remove_item(index)?;
                if let Some(w) = old_weapon {
                    self.add_item(w.into())?;
                }
                Ok(old_weapon.map(Item::from))
            }
            ItemKind::Armor(armor) => {
                let old_armor = self.equipment.equip_armor(armor.clone(), strength)?;
                self.remove_item(index)?;
                if let Some(ref a) = old_armor {
                    self.add_item(a.into())?;
                }
                Ok(old_armor.map(Item::from))
            }
            ItemKind::Ring(ring) => {
                let slot = self.find_available_ring_slot();
                let old_ring = self.equipment.equip_ring(ring.clone(), slot)?;
                self.remove_item(index)?;
                if let Some(ref r) = old_ring {
                    self.add_item(r.into())?;
                }
                Ok(old_ring.map(Item::from))
            }
            _ => Err(BagError::CannotEquipItem),
        }
    }

    pub fn upgrade_weapon(&mut self) -> Result<(), BagError> {
        let scroll_idx = self.find_upgrade_scroll()?;
        self.scrolls.remove(scroll_idx)?;
        self.equipment.upgrade_weapon()?;
        Ok(())
    }

    /* ================== 物品管理 ================== */
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

    /* ================== 查询方法 ================== */
    pub fn get_item_by_index(&self, index: usize) -> Result<Item, BagError> {
        let mut idx = index;

        // 检查武器
        if idx < self.weapons.len() {
            return self
                .weapons
                .get(idx)
                .map(|w| Item::from(w.clone()))
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.weapons.len();

        // 检查护甲
        if idx < self.armors.len() {
            return self
                .armors
                .get(idx)
                .map(|a| Item::from(a.clone()))
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.armors.len();

        // 检查药水
        if idx < self.potions.len() {
            return self
                .potions
                .get(idx)
                .map(|p| Item::from(p.clone()))
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.potions.len();

        // 检查卷轴
        if idx < self.scrolls.len() {
            return self
                .scrolls
                .get(idx)
                .map(|s| Item::from(s.clone()))
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.scrolls.len();

        // 检查戒指
        if idx < self.rings.len() {
            return self
                .rings
                .get(idx)
                .map(|r| Item::from(r.clone()))
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.rings.len();

        // 检查食物
        if idx < self.food.len() {
            return self
                .food
                .get(idx)
                .map(|f| Item::from(f.clone()))
                .ok_or(BagError::InvalidIndex);
        }

        Err(BagError::InvalidIndex)
    }

    fn find_upgrade_scroll(&self) -> Result<usize, BagError> {
        self.scrolls
            .find(|s| matches!(s.kind, ScrollKind::Upgrade))
            .ok_or(BagError::NoUpgradeScroll)
    }

    fn find_available_ring_slot(&self) -> usize {
        if self.equipment.rings[0].is_none() {
            0
        } else {
            1
        }
    }

    /* ================== 排序功能 ================== */
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

    /* ================== 获取方法 ================== */
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

    /* ================== 装备属性计算 ================== */
    pub fn armor_defense(&self) -> u32 {
        self.equipment.armor.as_ref().map_or(0, |a| a.defense)
    }

    pub fn crit_bonus(&self) -> f32 {
        self.equipment.weapon.as_ref().map_or(1.0, |w| w.crit_modifier)
    }

    pub fn evasion_penalty(&self) -> u32 {
        self.equipment.armor.as_ref().map_or(0, |a| a.evasion_penalty)
    }
}
