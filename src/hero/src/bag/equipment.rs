// src/hero/src/bag/equipment.rs
use bincode::{Decode, Encode};
use items::{armor::Armor, ring::Ring, scroll::Scroll, weapon::Weapon};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use thiserror::Error;

/// 装备系统错误类型（完全匹配游戏机制）
#[derive(Debug, Error, PartialEq)]
pub enum EquipError {
    #[error("装备槽已满")]
    SlotFull,
    #[error("已达最大强化等级(+15)")]
    MaxUpgrade,
    #[error("不兼容的装备类型")]
    IncompatibleType,
    #[error("需要先解除诅咒")]
    CursedItem,
    #[error("需要升级卷轴")]
    UpgradeScrollRequired,
    #[error("力量不足")]
    StrengthRequirement,
}

/// 装备位枚举（支持完整迭代）
#[derive(Debug, Display, Clone, Copy, EnumIter, PartialEq, Serialize, Deserialize)]
pub enum EquipmentSlot {
    #[strum(serialize = "主手")]
    Weapon,
    #[strum(serialize = "护甲")]
    Armor,
    #[strum(serialize = "戒指1")]
    Ring1,
    #[strum(serialize = "戒指2")]
    Ring2,
}

/// 装备系统（完整实现游戏机制）
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: Option<Weapon>,
    pub armor: Option<Armor>,
    pub rings: [Option<Ring>; 2],
    pub cursed: bool, // 全局诅咒状态
}

impl Default for Equipment {
    fn default() -> Self {
        Self::new()
    }
}

impl Equipment {
    /// 创建空装备栏（初始无诅咒状态）
    pub fn new() -> Self {
        Self {
            weapon: None,
            armor: None,
            rings: [None, None],
            cursed: false,
        }
    }

    /// 装备武器（完整游戏逻辑）
    pub fn equip_weapon(
        &mut self,
        weapon: Weapon,
        user_str: u8,
    ) -> Result<Option<Weapon>, EquipError> {
        // 检查力量需求
        if user_str < weapon.str_requirement {
            return Err(EquipError::StrengthRequirement);
        }

        // 检查诅咒状态
        if weapon.cursed && self.cursed {
            return Err(EquipError::CursedItem);
        }

        let old = self.weapon.take();
        self.weapon = Some(weapon);
        self.update_cursed_status();
        Ok(old)
    }

    /// 装备护甲（完整游戏逻辑）
    pub fn equip_armor(&mut self, armor: Armor, user_str: u8) -> Result<Option<Armor>, EquipError> {
        // 检查力量需求
        if user_str < armor.str_requirement {
            return Err(EquipError::StrengthRequirement);
        }

        // 检查诅咒状态
        if armor.cursed && self.cursed {
            return Err(EquipError::CursedItem);
        }

        let old = self.armor.take();
        self.armor = Some(armor);
        self.update_cursed_status();
        Ok(old)
    }

    /// 装备戒指（支持双戒指槽）
    pub fn equip_ring(&mut self, ring: Ring, slot: usize) -> Result<Option<Ring>, EquipError> {
        if slot >= 2 {
            return Err(EquipError::IncompatibleType);
        }

        // 检查诅咒状态
        if ring.cursed && self.cursed {
            return Err(EquipError::CursedItem);
        }

        let old = self.rings[slot].take();
        self.rings[slot] = Some(ring);
        self.update_cursed_status();
        Ok(old)
    }

    /// 卸下武器（考虑诅咒状态）
    pub fn unequip_weapon(&mut self) -> Result<Option<Weapon>, EquipError> {
        if self.cursed {
            return Err(EquipError::CursedItem);
        }
        let weapon = self.weapon.take();
        self.update_cursed_status();
        Ok(weapon)
    }

    /// 卸下护甲（考虑诅咒状态）
    pub fn unequip_armor(&mut self) -> Result<Option<Armor>, EquipError> {
        if self.cursed {
            return Err(EquipError::CursedItem);
        }
        let armor = self.armor.take();
        self.update_cursed_status();
        Ok(armor)
    }

    /// 卸下指定槽位的戒指（考虑诅咒状态）
    pub fn unequip_ring(&mut self, slot: usize) -> Result<Option<Ring>, EquipError> {
        if slot >= 2 {
            return Err(EquipError::IncompatibleType);
        }
        if self.cursed {
            return Err(EquipError::CursedItem);
        }
        let ring = self.rings[slot].take();
        self.update_cursed_status();
        Ok(ring)
    }

    /// 强制卸下所有装备（无视诅咒状态，用于特殊场景）
    pub fn force_unequip_all(&mut self) -> (Option<Weapon>, Option<Armor>, [Option<Ring>; 2]) {
        let weapon = self.weapon.take();
        let armor = self.armor.take();
        let rings = std::mem::take(&mut self.rings);
        self.cursed = false;
        (weapon, armor, rings)
    }

    /// 武器强化（无需升级卷轴）
    pub fn upgrade_weapon(&mut self) -> Result<(), EquipError> {
        match &mut self.weapon {
            Some(weapon) => {
                if weapon.upgrade_level >= 15 {
                    return Err(EquipError::MaxUpgrade);
                }
                if weapon.cursed {
                    self.cursed = false;
                }
                weapon.upgrade();
                Ok(())
            }
            None => Err(EquipError::IncompatibleType),
        }
    }

    /// 获取当前装备（供UI渲染）
    pub fn get_equipment(&self) -> Vec<(EquipmentSlot, Option<&dyn std::fmt::Display>)> {
        use EquipmentSlot::*;
        vec![
            (Weapon, self.weapon.as_ref().map(|w| w as _)),
            (Armor, self.armor.as_ref().map(|a| a as _)),
            (Ring1, self.rings[0].as_ref().map(|r| r as _)),
            (Ring2, self.rings[1].as_ref().map(|r| r as _)),
        ]
    }

    /// 检查全局诅咒状态
    pub fn is_cursed(&self) -> bool {
        self.cursed
    }

    /// 计算装备总防御力（护甲+戒指加成）
    pub fn total_defense(&self) -> i32 {
        let armor_def = self.armor.as_ref().map(|a| a.defense).unwrap_or(0);
        let ring_bonus: i32 = self
            .rings
            .iter()
            .filter_map(|r| r.as_ref())
            .map(|r| r.defense_bonus() as i32)
            .sum();

        armor_def + ring_bonus
    }

    /// 更新全局诅咒状态
    fn update_cursed_status(&mut self) {
        self.cursed = self.weapon.as_ref().map_or(false, |w| w.cursed)
            || self.armor.as_ref().map_or(false, |a| a.cursed)
            || self
                .rings
                .iter()
                .any(|r| r.as_ref().map_or(false, |r| r.cursed));
    }
    /// 获取当前装备的武器（如果有）
    pub fn weapon(&self) -> Option<&Weapon> {
        self.weapon.as_ref()
    }

    /// 获取当前装备的护甲（如果有）
    pub fn armor(&self) -> Option<&Armor> {
        self.armor.as_ref()
    }

    /// 获取当前装备的所有戒指（过滤掉None值）
    pub fn rings(&self) -> Vec<&Ring> {
        self.rings.iter().filter_map(|opt| opt.as_ref()).collect()
    }

    /// 获取指定槽位的戒指（0或1）
    pub fn ring(&self, slot: usize) -> Option<&Ring> {
        if slot < 2 {
            self.rings[slot].as_ref()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use items::{ArmorType, RingType, WeaponType};

    #[test]
    fn test_equipment_flow() {
        let mut eq = Equipment::new();
        let sword = Weapon::new(WeaponType::Sword, 1);
        let plate = Armor::new(ArmorType::Plate, 1);
        let ring = Ring::new(RingType::Power);

        assert!(eq.equip_weapon(sword.clone(), 10).unwrap().is_none());
        assert!(eq.equip_armor(plate.clone(), 10).unwrap().is_none());
        assert!(eq.equip_ring(ring.clone(), 0).unwrap().is_none());

        // 测试替换装备
        let hammer = Weapon::new(WeaponType::Hammer, 1);
        assert_eq!(eq.equip_weapon(hammer, 10).unwrap(), Some(sword));
    }

    #[test]
    fn test_cursed_equipment() {
        let mut eq = Equipment::new();
        let mut cursed_sword = Weapon::new(WeaponType::Sword, 1);
        cursed_sword.cursed = true;

        assert!(eq.equip_weapon(cursed_sword, 10).is_ok());
        assert!(eq.is_cursed());

        // 尝试装备第二件诅咒物品
        let mut cursed_ring = Ring::new(RingType::Power);
        cursed_ring.cursed = true;
        assert_eq!(eq.equip_ring(cursed_ring, 1), Err(EquipError::CursedItem));
    }

    #[test]
    fn test_strength_requirement() {
        let mut eq = Equipment::new();
        let heavy_armor = Armor::new(ArmorType::Plate, 1);
        assert_eq!(
            eq.equip_armor(heavy_armor, 5),
            Err(EquipError::StrengthRequirement)
        );
    }

    #[test]
    fn test_unequip() {
        let mut eq = Equipment::new();
        let sword = Weapon::new(WeaponType::Sword, 1);
        let plate = Armor::new(ArmorType::Plate, 1);
        let ring = Ring::new(RingType::Power);

        eq.equip_weapon(sword.clone(), 10).unwrap();
        eq.equip_armor(plate.clone(), 10).unwrap();
        eq.equip_ring(ring.clone(), 0).unwrap();

        // 正常卸下
        assert_eq!(eq.unequip_weapon().unwrap(), Some(sword));
        assert_eq!(eq.unequip_armor().unwrap(), Some(plate));
        assert_eq!(eq.unequip_ring(0).unwrap(), Some(ring));
        assert!(!eq.is_cursed());
    }

    #[test]
    fn test_cursed_unequip() {
        let mut eq = Equipment::new();
        let mut cursed_sword = Weapon::new(WeaponType::Sword, 1);
        cursed_sword.cursed = true;

        eq.equip_weapon(cursed_sword, 10).unwrap();
        assert!(eq.is_cursed());

        // 尝试卸下诅咒装备
        assert_eq!(eq.unequip_weapon(), Err(EquipError::CursedItem));

        // 强制卸下
        let (weapon, _, _) = eq.force_unequip_all();
        assert!(weapon.is_some());
        assert!(!eq.is_cursed());
    }

    #[test]
    fn test_force_unequip_all() {
        let mut eq = Equipment::new();
        let sword = Weapon::new(WeaponType::Sword, 1);
        let plate = Armor::new(ArmorType::Plate, 1);
        let ring1 = Ring::new(RingType::Power);
        let ring2 = Ring::new(RingType::Defense);

        eq.equip_weapon(sword.clone(), 10).unwrap();
        eq.equip_armor(plate.clone(), 10).unwrap();
        eq.equip_ring(ring1.clone(), 0).unwrap();
        eq.equip_ring(ring2.clone(), 1).unwrap();

        let (weapon, armor, rings) = eq.force_unequip_all();
        assert_eq!(weapon, Some(sword));
        assert_eq!(armor, Some(plate));
        assert_eq!(rings, [Some(ring1), Some(ring2)]);
        assert!(eq.weapon.is_none());
        assert!(eq.armor.is_none());
        assert!(eq.rings.iter().all(|r| r.is_none()));
    }
}

/*
// 错误类型
enum EquipError {
    SlotFull,
    MaxUpgrade,
    IncompatibleType,
    CursedItem,
    UpgradeScrollRequired,
    StrengthRequirement,
}

// 装备槽枚举
enum EquipmentSlot {
    Weapon,
    Armor,
    Ring1,
    Ring2,
}

// 装备系统结构体
struct Equipment {
    weapon: Option<Weapon>,
    armor: Option<Armor>,
    rings: [Option<Ring>; 2],
    cursed: bool,
}

// 实现块
impl Default for Equipment {
    fn default() -> Self;
}

impl Equipment {
    // 方法签名
    pub fn new() -> Self;
    pub fn equip_weapon(&mut self, weapon: Weapon, user_str: u8) -> Result<Option<Weapon>, EquipError>;
    pub fn equip_armor(&mut self, armor: Armor, user_str: u8) -> Result<Option<Armor>, EquipError>;
    pub fn equip_ring(&mut self, ring: Ring, slot: usize) -> Result<Option<Ring>, EquipError>;
    pub fn unequip_weapon(&mut self) -> Result<Option<Weapon>, EquipError>;
    pub fn unequip_armor(&mut self) -> Result<Option<Armor>, EquipError>;
    pub fn unequip_ring(&mut self, slot: usize) -> Result<Option<Ring>, EquipError>;
    pub fn force_unequip_all(&mut self) -> (Option<Weapon>, Option<Armor>, [Option<Ring>; 2]);
    pub fn upgrade_weapon(&mut self) -> Result<(), EquipError>;
    pub fn get_equipment(&self) -> Vec<(EquipmentSlot, Option<&dyn std::fmt::Display>)>;
    pub fn is_cursed(&self) -> bool;
    pub fn total_defense(&self) -> i32;
    fn update_cursed_status(&mut self);
}

// 测试模块
mod tests {
    fn test_equipment_flow();
    fn test_cursed_equipment();
    fn test_strength_requirement();
    fn test_unequip();
    fn test_cursed_unequip();
    fn test_force_unequip_all();
}
*/
