use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::items::armor::armor::*;
use crate::items::food::food::*;
use crate::items::misc::misc::*;
use crate::items::potion::potion::*;
use crate::items::ring::ring::*;
use crate::items::scoll::scoll::*;
use crate::items::seed::seed::*;
use crate::items::stone::stone::*;
use crate::items::wand::wand::*;
use crate::items::weapon::weapon::*;

/// 基础物品结构（还原游戏内属性）
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub name: String,
    pub quantity: usize,    // 堆叠数量
    pub cursed: bool,       // 诅咒状态
    pub cursed_known: bool, // 是否已知被诅咒
    pub level: i32,         // 强化等级（+1,+2等）
}

/// 物品类型枚举（与Shattered PD完全一致）
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum ItemKind {
    Weapon(Weapon), // 近战武器
    Armor(Armor),   // 护甲
    Potion(Potion), // 药水（12种）
    Scroll(Scroll), // 卷轴（10种）
    Food(Food),     // 食物（3种）
    Wand(Wand),     // 法杖（8种）
    Ring(Ring),     // 戒指（10种）
    Seed(Seed),     // 种子（8种）
    Stone(Stone),   // 魔法石（6种）
    Misc(Misc),     // 杂项（钥匙等）
}

impl Item {
    /// 获取显示名称（还原游戏内命名规则）
    pub fn name(&self) -> String {
        if self.is_identified() {
            // 已鉴定物品显示全名
            match &self.kind {
                ItemKind::Weapon(w) => format!("{}+{} {}", w.tier, self.level, self.name),
                ItemKind::Armor(a) => format!("{}+{} {}", a.tier, self.level, self.name),
                _ => self.name.clone(),
            }
        } else {
            // 未鉴定物品显示类型
            match &self.kind {
                ItemKind::Potion(_) => "未知药水".to_string(),
                ItemKind::Scroll(_) => "未知卷轴".to_string(),
                ItemKind::Ring(_) => "未知戒指".to_string(),
                _ => self.name.clone(),
            }
        }
    }

    /// 是否为消耗品（精确匹配游戏机制）
    pub fn is_consumable(&self) -> bool {
        matches!(
            &self.kind,
            ItemKind::Potion(_) | ItemKind::Scroll(_) | ItemKind::Food(_)
        )
    }

    /// 物品是否已鉴定
    pub fn is_identified(&self) -> bool {
        match &self.kind {
            ItemKind::Potion(p) => p.identified,
            ItemKind::Scroll(s) => s.identified,
            ItemKind::Ring(_) => true, // 戒指需要装备才知效果
            _ => todo!(),              // 其他物品默认已鉴定
        }
    }

    /// 获取物品价值（用于商店系统）
    pub fn value(&self) -> usize {
        let base_value = match &self.kind {
            ItemKind::Potion(_) => 100,
            ItemKind::Scroll(_) => 150,
            // ...其他匹配项
            _ => todo!(),
        };
        base_value * (self.level.max(0) as usize + 1)
    }
}
