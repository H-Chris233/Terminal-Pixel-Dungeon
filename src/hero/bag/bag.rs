// src/hero/bag/bag.rs

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::hero::hero::*;
use crate::items::{
    armor::Armor, food::Food, misc::MiscItem, potion::Potion, ring::Ring, scroll::Scroll,
    seed::Seed, wand::Wand, weapon::Weapon,
};

/// 背包结构体，参考破碎的像素地牢游戏逻辑
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct Bag {
    /// 当前装备的武器（可为空）
    pub equipped_weapon: Option<Weapon>,
    /// 当前装备的护甲（可为空）
    pub equipped_armor: Option<Armor>,
    /// 武器库存（包括未装备的）
    pub weapons: Vec<Weapon>,
    /// 护甲库存（包括未装备的）
    pub armors: Vec<Armor>,
    /// 药水，使用HashMap存储数量和类型
    pub potions: HashMap<Potion, u32>,
    /// 卷轴，使用HashMap存储数量和类型
    pub scrolls: HashMap<Scroll, u32>,
    /// 种子，使用HashMap存储数量和类型
    pub seeds: HashMap<Seed, u32>,
    /// 食物，使用HashMap存储数量和类型
    pub foods: HashMap<Food, u32>,
    /// 法杖
    pub wands: Vec<Wand>,
    /// 戒指
    pub rings: Vec<Ring>,
    /// 杂项物品
    pub misc_items: Vec<MiscItem>,
    /// 背包容量限制（参考游戏默认设置）
    pub capacity: usize,
}

impl Bag {
    /// 创建一个新的空背包
    pub fn new() -> Self {
        Bag {
            equipped_weapon: None,
            equipped_armor: None,
            weapons: Vec::new(),
            armors: Vec::new(),
            potions: HashMap::new(),
            scrolls: HashMap::new(),
            seeds: HashMap::new(),
            foods: HashMap::new(),
            wands: Vec::new(),
            rings: Vec::new(),
            misc_items: Vec::new(),
            capacity: 20, // 默认背包容量
        }
    }

    /// 装备武器
    pub fn equip_weapon(&mut self, weapon: Weapon) -> Option<Weapon> {
        let old_weapon = self.equipped_weapon.take();
        self.equipped_weapon = Some(weapon);
        old_weapon
    }

    /// 装备护甲
    pub fn equip_armor(&mut self, armor: Armor) -> Option<Armor> {
        let old_armor = self.equipped_armor.take();
        self.equipped_armor = Some(armor);
        old_armor
    }

    /// 添加武器到背包
    pub fn add_weapon(&mut self, weapon: Weapon) -> bool {
        if self.weapons.len() + self.armors.len() + 1 > self.capacity {
            return false;
        }
        self.weapons.push(weapon);
        true
    }

    /// 添加护甲到背包
    pub fn add_armor(&mut self, armor: Armor) -> bool {
        if self.weapons.len() + self.armors.len() + 1 > self.capacity {
            return false;
        }
        self.armors.push(armor);
        true
    }

    /// 添加药水到背包
    pub fn add_potion(&mut self, potion: Potion) -> bool {
        let count = self.potions.entry(potion).or_insert(0);
        *count += 1;
        true
    }

    /// 添加卷轴到背包
    pub fn add_scroll(&mut self, scroll: Scroll) -> bool {
        let count = self.scrolls.entry(scroll).or_insert(0);
        *count += 1;
        true
    }

    /// 添加种子到背包
    pub fn add_seed(&mut self, seed: Seed) -> bool {
        let count = self.seeds.entry(seed).or_insert(0);
        *count += 1;
        true
    }

    /// 添加食物到背包
    pub fn add_food(&mut self, food: Food) -> bool {
        let count = self.foods.entry(food).or_insert(0);
        *count += 1;
        true
    }

    /// 检查背包是否已满
    pub fn is_full(&self) -> bool {
        self.weapons.len() + self.armors.len() >= self.capacity
    }

    /// 获取背包当前物品数量
    pub fn item_count(&self) -> usize {
        self.weapons.len()
            + self.armors.len()
            + self.potions.len()
            + self.scrolls.len()
            + self.seeds.len()
            + self.foods.len()
            + self.wands.len()
            + self.rings.len()
            + self.misc_items.len()
    }

    /// 升级装备中的武器（参考游戏升级机制）
    pub fn upgrade_equipped_weapon(&mut self) -> bool {
        if let Some(ref mut weapon) = self.equipped_weapon {
            weapon.upgrade();
            return true;
        }
        false
    }

    /// 升级装备中的护甲
    pub fn upgrade_equipped_armor(&mut self) -> bool {
        if let Some(ref mut armor) = self.equipped_armor {
            armor.upgrade();
            return true;
        }
        false
    }

    /// 使用药水
    pub fn use_potion(&mut self, potion_type: Potion) -> Option<Potion> {
        if let Some(count) = self.potions.get_mut(&potion_type) {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    self.potions.remove(&potion_type);
                }
                return Some(potion_type);
            }
        }
        None
    }

    /// 使用卷轴
    pub fn use_scroll(&mut self, scroll_type: Scroll) -> Option<Scroll> {
        if let Some(count) = self.scrolls.get_mut(&scroll_type) {
            if *count > 0 {
                *count -= 1;
                if *count == 0 {
                    self.scrolls.remove(&scroll_type);
                }
                return Some(scroll_type);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{armor::ArmorType, weapon::WeaponType};

    #[test]
    fn test_bag_operations() {
        let mut bag = Bag::new();

        // 测试武器装备
        let sword = Weapon::new(WeaponType::Sword, 1);
        assert!(bag.add_weapon(sword.clone()));
        assert_eq!(bag.equip_weapon(sword.clone()), None);
        assert_eq!(bag.equipped_weapon.as_ref(), Some(&sword));

        // 测试护甲装备
        let plate = Armor::new(ArmorType::Plate, 1);
        assert!(bag.add_armor(plate.clone()));
        assert_eq!(bag.equip_armor(plate.clone()), None);
        assert_eq!(bag.equipped_armor.as_ref(), Some(&plate));

        // 测试药水添加和使用
        let healing_potion = Potion::Healing;
        assert!(bag.add_potion(healing_potion));
        assert_eq!(bag.use_potion(healing_potion), Some(healing_potion));
        assert_eq!(bag.potions.contains_key(&healing_potion), false);
    }
}
