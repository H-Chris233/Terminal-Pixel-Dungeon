// src/hero/src/bag.rs
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use items::{
    Item, ItemKind,
    armor::Armor,
    food::Food,
    herb::{Herb, HerbKind},
    misc::{MiscItem, MiscKind},
    potion::{Potion, PotionKind},
    ring::Ring,
    scroll::{Scroll, ScrollKind},
    seed::{Seed, SeedKind},
    stone::Stone,
    throwable::Throwable,
    wand::Wand,
    weapon::Weapon,
};
use items::ItemTrait;

// 子模块定义
pub mod equipment; // 装备管理
pub mod inventory; // 物品库存管理

use equipment::{EquipError, Equipment, EquipmentSlot};
use inventory::{Inventory, InventoryError, InventorySlot};

fn default_throwable_inventory() -> Inventory<Throwable> {
    Inventory::new(6)
}

fn default_herb_inventory() -> Inventory<Herb> {
    Inventory::new(6)
}

/// 完整的背包系统（遵循破碎的像素地牢机制）
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Bag {
    gold: u32,                      // 金币数量
    equipment: Equipment,           // 已装备的物品
    weapons: Inventory<Weapon>,     // 武器库存
    armors: Inventory<Armor>,       // 护甲库存
    potions: Inventory<Potion>,     // 药水库存
    scrolls: Inventory<Scroll>,     // 卷轴库存
    wands: Inventory<Wand>,         // 法杖库存
    rings: Inventory<Ring>,         // 戒指库存
    seeds: Inventory<Seed>,         // 种子库存
    stones: Inventory<Stone>,       // 宝石库存
    food: Inventory<Food>,          // 食物库存
    misc: Inventory<MiscItem>,      // 杂项物品
    #[serde(default = "default_throwable_inventory")]
    throwables: Inventory<Throwable>, // 投掷武器
    #[serde(default = "default_herb_inventory")]
    herbs: Inventory<Herb>,         // 药草库存
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
    #[error("无法合成物品")]
    CombinationFailed,
}

impl Bag {
    /// 创建具有默认容量的新背包
    pub fn new() -> Self {
        Self {
            gold: 0,
            equipment: Equipment::new(),
            weapons: Inventory::new(4),      // 武器容量4
            armors: Inventory::new(3),       // 护甲容量3
            potions: Inventory::new(8),      // 药水容量8
            scrolls: Inventory::new(8),      // 卷轴容量8
            wands: Inventory::new(4),        // 法杖容量4
            rings: Inventory::new(4),        // 戒指容量4
            seeds: Inventory::new(4),        // 种子容量4
            stones: Inventory::new(4),       // 宝石容量4
            food: Inventory::new(4),         // 食物容量4
            misc: Inventory::new(4),         // 杂项容量4
            throwables: Inventory::new(6),   // 投掷武器容量6
            herbs: Inventory::new(6),        // 药草容量6
        }
    }

}

impl Default for Bag {
    fn default() -> Self {
        Self::new()
    }
}

impl Bag {
    /* ================== 金币管理 ================== */
    pub fn add_gold(&mut self, amount: u32) -> Result<(), BagError> {
        self.gold = self
            .gold
            .checked_add(amount)
            .ok_or(BagError::InventoryFull)?;
        Ok(())
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
                Ok(Item::from((*potion).clone()))
            }
            ItemKind::Scroll(_) => {
                let scroll = self.scrolls.remove(index)?;
                Ok(Item::from((*scroll).clone()))
            }
            ItemKind::Food(_) => {
                let food = self.food.remove(index)?;
                Ok(Item::from((*food).clone()))
            }
            ItemKind::Seed(_) => {
                let seed = self.seeds.remove(index)?;
                Ok(Item::from((*seed).clone()))
            }
            ItemKind::Wand(_) => {
                let wand = self.wands.remove(index)?;
                Ok(Item::from((*wand).clone()))
            }
            ItemKind::Stone(_) => {
                let stone = self.stones.remove(index)?;
                Ok(Item::from((*stone).clone()))
            }
            ItemKind::Throwable(_) => {
                let throwable = self.throwables.remove(index)?;
                Ok(Item::from((*throwable).clone()))
            }
            ItemKind::Herb(_) => {
                let herb = self.herbs.remove(index)?;
                Ok(Item::from((*herb).clone()))
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
                if let Some(ref w) = old_weapon {
                    self.add_item(Item::from(w.clone()))?;
                }
                Ok(old_weapon.map(Item::from))
            }
            ItemKind::Armor(armor) => {
                let old_armor = self.equipment.equip_armor(armor.clone(), strength)?;
                self.remove_item(index)?;
                if let Some(ref a) = old_armor {
                    self.add_item(Item::from(a.clone()))?;
                }
                Ok(old_armor.map(Item::from))
            }
            ItemKind::Ring(ring) => {
                let slot = self.find_available_ring_slot();
                let old_ring = self.equipment.equip_ring(ring.clone(), slot)?;
                self.remove_item(index)?;
                if let Some(ref r) = old_ring {
                    self.add_item(Item::from(r.clone()))?;
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
        let quantity = item.quantity.max(1);

        match item.kind {
            ItemKind::Weapon(w) => self
                .weapons
                .add_sorted(w, |a, b| b.upgrade_level.cmp(&a.upgrade_level)),
            ItemKind::Armor(a) => self
                .armors
                .add_sorted(a, |a, b| b.upgrade_level.cmp(&a.upgrade_level)),
            ItemKind::Potion(p) => {
                if quantity > 1 {
                    self.potions.add_multiple(p, quantity)
                } else {
                    self.potions.add(p)
                }
            }
            ItemKind::Scroll(s) => {
                if quantity > 1 {
                    self.scrolls.add_multiple(s, quantity)
                } else {
                    self.scrolls.add(s)
                }
            }
            ItemKind::Wand(w) => self
                .wands
                .add_sorted(w, |a, b| b.level.cmp(&a.level)),
            ItemKind::Ring(r) => self.rings.add(r),
            ItemKind::Seed(s) => {
                if quantity > 1 {
                    self.seeds.add_multiple(s, quantity)
                } else {
                    self.seeds.add(s)
                }
            }
            ItemKind::Stone(s) => {
                if quantity > 1 {
                    self.stones.add_multiple(s, quantity)
                } else {
                    self.stones.add(s)
                }
            }
            ItemKind::Food(f) => {
                if quantity > 1 {
                    self.food.add_multiple(f, quantity)
                } else {
                    self.food.add(f)
                }
            }
            ItemKind::Throwable(t) => {
                let mut result = Ok(());
                for _ in 0..quantity {
                    result = self.throwables.add_sorted(
                        t.clone(),
                        |a, b| b.range.cmp(&a.range).then_with(|| b.damage.1.cmp(&a.damage.1)),
                    );
                    if result.is_err() {
                        break;
                    }
                }
                result
            }
            ItemKind::Herb(h) => {
                if quantity > 1 {
                    self.herbs.add_multiple(h, quantity)
                } else {
                    self.herbs.add(h)
                }
            }
            ItemKind::Misc(m) => match m.kind {
                MiscKind::Gold(amount) => {
                    let total = amount.saturating_mul(quantity);
                    let _ = self.add_gold(total);
                    Ok(())
                }
                _ => {
                    if quantity > 1 && m.is_stackable() {
                        self.misc.add_multiple(m, quantity)
                    } else {
                        self.misc.add(m)
                    }
                }
            },
        }
        .map_err(Into::into)
    }

    /// 合成药草与种子，生成特定药水
    pub fn combine_reagents(
        &mut self,
        herb_kind: HerbKind,
        seed_kind: SeedKind,
    ) -> Result<Item, BagError> {
        let recipe = match (herb_kind, seed_kind) {
            (HerbKind::Sungrass, SeedKind::Earthroot) => PotionKind::Healing,
            (HerbKind::Moonleaf, SeedKind::Fadeleaf) => PotionKind::Invisibility,
            (HerbKind::Nightshade, SeedKind::Sorrowmoss) => PotionKind::ToxicGas,
            (HerbKind::SpiritMoss, SeedKind::Dreamfoil) => PotionKind::Purity,
            (HerbKind::Dragonthorn, SeedKind::Stormvine) => PotionKind::Strength,
            (HerbKind::Glowcap, SeedKind::Icecap) => PotionKind::MindVision,
            _ => return Err(BagError::CombinationFailed),
        };

        let herb_index = self
            .herbs
            .find(|h| h.kind == herb_kind)
            .ok_or(BagError::CombinationFailed)?;
        let seed_index = self
            .seeds
            .find(|s| s.kind == seed_kind)
            .ok_or(BagError::CombinationFailed)?;

        self.herbs.remove(herb_index)?;
        self.seeds.remove(seed_index)?;

        let potion_item = Item::new(ItemKind::Potion(Potion::new_alchemy(recipe)));
        self.add_item(potion_item.clone())?;
        Ok(potion_item)
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
        idx -= self.food.len();

        // 添加缺失的检查
        if idx < self.wands.len() {
            return self.wands.remove(idx).map(|_| ()).map_err(Into::into);
        }
        idx -= self.wands.len();

        if idx < self.seeds.len() {
            return self.seeds.remove(idx).map(|_| ()).map_err(Into::into);
        }
        idx -= self.seeds.len();

        if idx < self.stones.len() {
            return self.stones.remove(idx).map(|_| ()).map_err(Into::into);
        }
        idx -= self.stones.len();

        if idx < self.misc.len() {
            return self.misc.remove(idx).map(|_| ()).map_err(Into::into);
        }
        idx -= self.misc.len();

        if idx < self.throwables.len() {
            return self
                .throwables
                .remove(idx)
                .map(|_| ())
                .map_err(Into::into);
        }
        idx -= self.throwables.len();

        if idx < self.herbs.len() {
            return self.herbs.remove(idx).map(|_| ()).map_err(Into::into);
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
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.weapons.len();

        // 检查护甲
        if idx < self.armors.len() {
            return self
                .armors
                .get(idx)
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.armors.len();

        // 检查药水
        if idx < self.potions.len() {
            return self
                .potions
                .get(idx)
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.potions.len();

        // 检查卷轴
        if idx < self.scrolls.len() {
            return self
                .scrolls
                .get(idx)
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.scrolls.len();

        // 检查戒指
        if idx < self.rings.len() {
            return self
                .rings
                .get(idx)
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.rings.len();

        // 检查食物
        if idx < self.food.len() {
            return self
                .food
                .get(idx)
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.food.len();

        // 检查法杖
        if idx < self.wands.len() {
            return self
                .wands
                .get(idx)
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.wands.len();

        // 检查种子
        if idx < self.seeds.len() {
            return self
                .seeds
                .get(idx)
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.seeds.len();

        // 检查宝石
        if idx < self.stones.len() {
            return self
                .stones
                .get(idx)
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.stones.len();

        // 检查杂项
        if idx < self.misc.len() {
            return self
                .misc
                .get(idx)
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.misc.len();

        if idx < self.throwables.len() {
            return self
                .throwables
                .get(idx)
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
                .ok_or(BagError::InvalidIndex);
        }
        idx -= self.throwables.len();

        if idx < self.herbs.len() {
            return self
                .herbs
                .get(idx)
                .map(|slot| match slot {
                    InventorySlot::Single(item) | InventorySlot::Stackable(item, _) => {
                        Item::from(item.as_ref().clone())
                    }
                })
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

    pub fn remove_curse(&mut self, slot: EquipmentSlot) {
        let _ = self.equipment.remove_curse(slot);
    }

    pub fn remove_curse_all(&mut self) {
        self.equipment.remove_curse_all()
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

    pub fn wands(&self) -> &Inventory<Wand> {
        &self.wands
    }

    pub fn equipment(&self) -> &Equipment {
        &self.equipment
    }

    pub fn rings(&self) -> &Inventory<Ring> {
        &self.rings
    }

    pub fn seeds(&self) -> &Inventory<Seed> {
        &self.seeds
    }

    pub fn stones(&self) -> &Inventory<Stone> {
        &self.stones
    }

    pub fn food(&self) -> &Inventory<Food> {
        &self.food
    }

    pub fn misc(&self) -> &Inventory<MiscItem> {
        &self.misc
    }

    pub fn throwables(&self) -> &Inventory<Throwable> {
        &self.throwables
    }

    pub fn herbs(&self) -> &Inventory<Herb> {
        &self.herbs
    }

    /* ================== 装备属性计算 ================== */

    pub fn evasion_penalty(&self) -> u32 {
        self.equipment
            .armor
            .as_ref()
            .map_or(0, |a| a.evasion_penalty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_reagents_creates_identified_potion() {
        let mut bag = Bag::new();

        let mut herb_item = Item::new(ItemKind::Herb(Herb::new(HerbKind::Sungrass)));
        if let ItemKind::Herb(ref mut herb) = herb_item.kind {
            herb.identified = true;
        }
        bag.add_item(herb_item).expect("failed to add herb");

        let seed_item = Item::new(ItemKind::Seed(Seed::new(SeedKind::Earthroot)));
        bag.add_item(seed_item).expect("failed to add seed");

        let result = bag
            .combine_reagents(HerbKind::Sungrass, SeedKind::Earthroot)
            .expect("combination should succeed");

        match result.kind {
            ItemKind::Potion(ref potion) => {
                assert!(matches!(potion.kind, PotionKind::Healing));
                assert!(potion.identified);
            }
            _ => panic!("expected potion from combination"),
        }

        assert!(bag.herbs().items().is_empty());
        assert!(bag.seeds().items().is_empty());
    }

    #[test]
    fn combine_reagents_requires_matching_seed() {
        let mut bag = Bag::new();

        bag.add_item(Item::new(ItemKind::Herb(Herb::new(HerbKind::Sungrass))))
            .expect("failed to add herb");

        let result = bag.combine_reagents(HerbKind::Sungrass, SeedKind::Fadeleaf);
        assert!(matches!(result, Err(BagError::CombinationFailed)));
    }
}
