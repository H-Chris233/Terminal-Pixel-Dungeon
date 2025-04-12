// src/hero/src/bag/equipment.rs
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    weapon: Option<Weapon>,
    armor: Option<Armor>,
    rings: [Option<Ring>; 2],
    cursed: bool, // 全局诅咒状态
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
        user_str: i32,
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
    pub fn equip_armor(
        &mut self,
        armor: Armor,
        user_str: i32,
    ) -> Result<Option<Armor>, EquipError> {
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
            .map(|r| r.defense_bonus())
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
}
