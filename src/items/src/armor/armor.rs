// src/items/armor/armor.rs

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 护甲数据（精确还原游戏机制）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Armor {
    pub name: String,
    pub tier: usize,               // 品阶1-5
    pub defense: i32,              // 基础防御
    pub level: i32,                // 强化等级（可正可负）
    pub glyph: Option<ArmorGlyph>, // 护甲刻印
    pub cursed: bool,              // 是否被诅咒
}

/// 护甲刻印类型（全部10种）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum ArmorGlyph {
    Affection,   // 魅惑 - 受到攻击时概率魅惑敌人
    AntiEntropy, // 抗熵 - 免疫燃烧和冰冻
    Brimstone,   // 硫磺 - 免疫燃烧，受到火焰攻击时恢复生命
    Camouflage,  // 伪装 - 静止时获得隐身
    Flow,        // 流动 - 被击退距离翻倍但不受伤害
    Obfuscation, // 混淆 - 敌人命中率降低
    Potential,   // 潜能 - 充能速度提升
    Repulsion,   // 排斥 - 击退攻击者
    Stone,       // 石肤 - 免疫毒气和瘫痪
    Thorns,      // 荆棘 - 反弹部分近战伤害
}

impl Armor {
    /// 创建新护甲
    pub fn new(tier: usize, level: i32) -> Self {
        let defense = Self::base_defense(tier);
        let name = Self::tier_name(tier);

        Armor {
            name,
            tier,
            defense,
            level,
            glyph: None,
            cursed: level < 0,
        }
    }

    /// 获取基础防御值（根据游戏平衡数据）
    fn base_defense(tier: usize) -> i32 {
        match tier {
            1 => 2,  // 布甲
            2 => 5,  // 皮甲
            3 => 8,  // 锁甲
            4 => 11, // 鳞甲
            5 => 14, // 板甲
            _ => 0,  // 默认值
        }
    }

    /// 获取品阶名称
    fn tier_name(tier: usize) -> String {
        match tier {
            1 => "布甲".to_string(),
            2 => "皮甲".to_string(),
            3 => "锁甲".to_string(),
            4 => "鳞甲".to_string(),
            5 => "板甲".to_string(),
            _ => "未知护甲".to_string(),
        }
    }

    /// 计算实际防御值（考虑强化等级）
    pub fn effective_defense(&self) -> i32 {
        let mut defense = self.defense + self.level;
        if self.cursed {
            defense -= 2; // 诅咒惩罚
        }
        defense.max(0) // 防御值不低于0
    }

    /// 升级护甲（参考游戏升级机制）
    pub fn upgrade(&mut self) {
        self.level += 1;
        if self.level >= 0 {
            self.cursed = false;
        }
    }

    /// 添加/改变刻印（会解除诅咒）
    pub fn inscribe(&mut self, glyph: ArmorGlyph) {
        self.glyph = Some(glyph);
        if self.cursed {
            self.cursed = false;
        }
    }

    /// 触发刻印效果（简化版，实际游戏需要更复杂的实现）
    pub fn trigger_glyph(&self) -> Option<GlyphEffect> {
        self.glyph.as_ref().map(|glyph| match glyph {
            ArmorGlyph::Thorns => GlyphEffect::ReflectDamage(1 + self.level.max(0) / 3),
            ArmorGlyph::Repulsion => GlyphEffect::Knockback(2),
            // 其他刻印效果...
            _ => GlyphEffect::None,
        })
    }
}

/// 刻印触发效果
#[derive(Debug, Clone)]
pub enum GlyphEffect {
    None,
    ReflectDamage(i32), // 反弹伤害值
    Knockback(i32),     // 击退距离
                        // 其他效果...
}

impl fmt::Display for Armor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level_str = match self.level {
            0 => "".to_string(),
            l if l > 0 => format!("+{}", l),
            l => format!("{}", l), // 负数自带负号
        };

        let glyph_str = match &self.glyph {
            Some(g) => format!("[{}]", g),
            None => "".to_string(),
        };

        write!(f, "{}{} {}", self.name, level_str, glyph_str)
    }
}

impl fmt::Display for ArmorGlyph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ArmorGlyph::Affection => "魅惑",
            ArmorGlyph::AntiEntropy => "抗熵",
            ArmorGlyph::Brimstone => "硫磺",
            ArmorGlyph::Camouflage => "伪装",
            ArmorGlyph::Flow => "流动",
            ArmorGlyph::Obfuscation => "混淆",
            ArmorGlyph::Potential => "潜能",
            ArmorGlyph::Repulsion => "排斥",
            ArmorGlyph::Stone => "石肤",
            ArmorGlyph::Thorns => "荆棘",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_armor_creation() {
        let leather = Armor::new(2, 0);
        assert_eq!(leather.name, "皮甲");
        assert_eq!(leather.effective_defense(), 5);
        assert!(!leather.cursed);
    }

    #[test]
    fn test_upgrade_armor() {
        let mut plate = Armor::new(5, 0);
        plate.upgrade();
        assert_eq!(plate.level, 1);
        assert_eq!(plate.effective_defense(), 15); // 14 + 1
    }

    #[test]
    fn test_cursed_armor() {
        let mut cursed = Armor::new(3, -2);
        assert!(cursed.cursed);
        assert_eq!(cursed.effective_defense(), 6); // 8 - 2 - 2(诅咒惩罚)

        cursed.upgrade();
        assert_eq!(cursed.level, -1);
        assert!(cursed.cursed);

        cursed.upgrade();
        assert_eq!(cursed.level, 0);
        assert!(!cursed.cursed);
    }

    #[test]
    fn test_glyphs() {
        let mut armor = Armor::new(4, 3);
        armor.inscribe(ArmorGlyph::Thorns);

        if let Some(effect) = armor.trigger_glyph() {
            match effect {
                GlyphEffect::ReflectDamage(dmg) => assert_eq!(dmg, 2), // 1 + (3/3)
                _ => panic!("Wrong glyph effect"),
            }
        }
    }
}
