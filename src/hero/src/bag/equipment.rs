// src/hero/src/bag/equipment.rs
use bincode::{Decode, Encode};
use items::{armor::Armor, ring::Ring, weapon::Weapon};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use thiserror::Error;

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
        }
    }

    /// 装备武器（完整游戏逻辑）
    pub fn equip_weapon(
        &mut self,
        weapon: Weapon,
        user_str: u8,
    ) -> Result<Option<Weapon>, EquipError> {
        if user_str < weapon.str_requirement {
            return Err(EquipError::StrengthRequirement);
        }

        let old = self.weapon.take();
        self.weapon = Some(weapon);
        Ok(old)
    }

    pub fn equip_armor(&mut self, armor: Armor, user_str: u8) -> Result<Option<Armor>, EquipError> {
        if user_str < armor.str_requirement {
            return Err(EquipError::StrengthRequirement);
        }

        let old = self.armor.take();
        self.armor = Some(armor);
        Ok(old)
    }

    /// 装备戒指（支持双戒指槽）
    pub fn equip_ring(&mut self, ring: Ring, slot: usize) -> Result<Option<Ring>, EquipError> {
        if slot >= 2 {
            return Err(EquipError::IncompatibleType);
        }

        let old = self.rings[slot].take();
        self.rings[slot] = Some(ring);
        Ok(old)
    }

    /// 卸下武器（考虑诅咒状态）
    pub fn unequip_weapon(&mut self) -> Result<Option<Weapon>, EquipError> {
        if self.weapon.as_ref().is_some_and(|w| w.cursed) {
            return Err(EquipError::CursedItem);
        }
        Ok(self.weapon.take())
    }

    /// 卸下护甲（考虑诅咒状态）
    pub fn unequip_armor(&mut self) -> Result<Option<Armor>, EquipError> {
        if self.armor.as_ref().is_some_and(|a| a.cursed) {
            return Err(EquipError::CursedItem);
        }
        Ok(self.armor.take())
    }

    /// 卸下指定槽位的戒指（考虑诅咒状态）
    pub fn unequip_ring(&mut self, slot: usize) -> Result<Option<Ring>, EquipError> {
        if slot >= 2 {
            return Err(EquipError::IncompatibleType);
        }
        if self.rings[slot].as_ref().is_some_and(|r| r.cursed) {
            return Err(EquipError::CursedItem);
        }
        Ok(self.rings[slot].take())
    }

    /// 强制卸下所有装备（无视诅咒状态，用于特殊场景）
    pub fn force_unequip_all(&mut self) -> (Option<Weapon>, Option<Armor>, [Option<Ring>; 2]) {
        (
            self.weapon.take(),
            self.armor.take(),
            std::mem::take(&mut self.rings),
        )
    }

    /// 武器强化（解除自身诅咒）
    pub fn upgrade_weapon(&mut self) -> Result<(), EquipError> {
        if let Some(weapon) = &mut self.weapon {
            if weapon.upgrade_level >= 15 {
                return Err(EquipError::MaxUpgrade);
            }
            weapon.cursed = false;
            weapon.upgrade();
            Ok(())
        } else {
            Err(EquipError::IncompatibleType)
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

    /// 计算装备总防御力（护甲+戒指加成）
    pub fn total_defense(&self) -> i32 {
        let armor_def = self.armor.as_ref().map(|a| a.defense).unwrap_or(0);
        self.rings
            .iter()
            .filter_map(|r| r.as_ref())
            .map(|r| r.defense_bonus() as i32)
            .sum::<i32>()
            + armor_def as i32
    }

    pub fn remove_curse(&mut self, slot: EquipmentSlot) -> Result<(), EquipError> {
        match slot {
            EquipmentSlot::Weapon => {
                if let Some(w) = &mut self.weapon {
                    w.cursed = false;
                    Ok(())
                } else {
                    Err(EquipError::IncompatibleType)
                }
            }
            EquipmentSlot::Armor => {
                if let Some(a) = &mut self.armor {
                    a.cursed = false;
                    Ok(())
                } else {
                    Err(EquipError::IncompatibleType)
                }
            }
            EquipmentSlot::Ring1 => {
                if let Some(r) = &mut self.rings[0] {
                    r.cursed = false;
                    Ok(())
                } else {
                    Err(EquipError::IncompatibleType)
                }
            }
            EquipmentSlot::Ring2 => {
                if let Some(r) = &mut self.rings[1] {
                    r.cursed = false;
                    Ok(())
                } else {
                    Err(EquipError::IncompatibleType)
                }
            }
        }
    }

    /// 解除所有装备的诅咒状态
    pub fn remove_curse_all(&mut self) {
        // 解除武器诅咒
        if let Some(weapon) = &mut self.weapon {
            weapon.cursed = false;
        }

        // 解除护甲诅咒
        if let Some(armor) = &mut self.armor {
            armor.cursed = false;
        }

        // 解除所有戒指诅咒
        for ring in self.rings.iter_mut().flatten() {
            ring.cursed = false;
        }
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
