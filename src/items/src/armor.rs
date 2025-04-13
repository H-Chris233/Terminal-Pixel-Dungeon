//src/items/src/armor.rs
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt;
use tui::style::Color;

/// 护甲数据（精确还原游戏机制）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Armor {
    pub tier: usize,               // 品阶1-5
    pub defense: i32,              // 基础防御
    pub upgrade_level: u8,         // 强化等级（非负）
    pub glyph: Option<ArmorGlyph>, // 护甲刻印
    pub cursed: bool,              // 是否被诅咒
    pub cursed_known: bool,        // 是否已鉴定出诅咒状态
    pub str_requirement: i32,      // 力量需求
    pub base_value: usize,
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
    pub fn new(tier: usize) -> Self {
        let defense = Self::base_defense(tier);
        let str_requirement = Self::base_str_requirement(tier);
        let base_value = Self::base_value(tier); // 新增基础价值计算

        Armor {
            tier,
            defense,
            upgrade_level: 0,
            glyph: None,
            cursed: false,
            cursed_known: false,
            str_requirement,
            base_value, // 初始化基础价值
        }
    }
    
    /// 随机生成新护甲（随机品阶、刻印和诅咒状态）
    pub fn random_new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        
        let tier = rng.random_range(1..=5);
        let mut armor = Armor::new(tier);
        
        // 20%概率有刻印
        if rng.random_bool(0.2) {
            let glyphs = [
                ArmorGlyph::Affection,
                ArmorGlyph::AntiEntropy,
                ArmorGlyph::Brimstone,
                ArmorGlyph::Camouflage,
                ArmorGlyph::Flow,
                ArmorGlyph::Obfuscation,
                ArmorGlyph::Potential,
                ArmorGlyph::Repulsion,
                ArmorGlyph::Stone,
                ArmorGlyph::Thorns,
            ];
            let glyph = glyphs[rng.random_range(0..glyphs.len())].clone();
            armor.inscribe(glyph);
        }
        
        // 10%概率被诅咒
        if rng.random_bool(0.1) {
            armor.curse();
        }
        
        armor
    }

    /// 获取基础价值（根据品阶）
    fn base_value(tier: usize) -> usize {
        match tier {
            1 => 150,  // 布甲
            2 => 300,  // 皮甲
            3 => 600,  // 锁甲
            4 => 1200, // 鳞甲
            5 => 2400, // 板甲
            _ => 0,
        }
    }

    /// 计算护甲完整价值（考虑品阶、等级、刻印和诅咒状态）
    pub fn value(&self) -> usize {
        let mut value = self.base_value;

        // 等级加成（每级+20%基础价值）
        if self.upgrade_level > 0 {
            value += (self.base_value as f32 * 0.2 * self.upgrade_level as f32) as usize;
        }

        // 刻印加成（不同类型有不同加成）
        if let Some(glyph) = &self.glyph {
            value = match glyph {
                ArmorGlyph::Thorns => (value as f32 * 1.3) as usize, // 荆棘 +30%
                ArmorGlyph::Repulsion => (value as f32 * 1.25) as usize, // 排斥 +25%
                ArmorGlyph::Affection => (value as f32 * 1.2) as usize, // 魅惑 +20%
                ArmorGlyph::Potential => (value as f32 * 1.15) as usize, // 潜能 +15%
                _ => (value as f32 * 1.1) as usize,                  // 其他刻印 +10%
            };
        }

        // 诅咒惩罚（价值减半）
        if self.cursed {
            value /= 2;
        }

        value
    }

    /// 获取基础防御值（根据游戏平衡数据）
    fn base_defense(tier: usize) -> i32 {
        match tier {
            1 => 2,  // 布甲
            2 => 5,  // 皮甲
            3 => 8,  // 锁甲
            4 => 11, // 鳞甲
            5 => 14, // 板甲
            _ => 0,
        }
    }

    fn base_str_requirement(tier: usize) -> i32 {
        match tier {
            1 => 10, // 布甲
            2 => 12, // 皮甲
            3 => 14, // 锁甲
            4 => 16, // 鳞甲
            5 => 18, // 板甲
            _ => 0,
        }
    }

    /// 获取品阶名称
    pub fn tier_name(&self) -> String {
        match self.tier {
            1 => "布甲".to_string(),
            2 => "皮甲".to_string(),
            3 => "锁甲".to_string(),
            4 => "鳞甲".to_string(),
            5 => "板甲".to_string(),
            _ => "未知护甲".to_string(),
        }
    }

    /// 获取完整名称（包含等级和刻印）
    pub fn name(&self) -> String {
        let level_str = if self.upgrade_level > 0 {
            format!("+{}", self.upgrade_level)
        } else {
            String::new()
        };

        let glyph_str = match &self.glyph {
            Some(g) => format!(" [{}]", g),
            None => String::new(),
        };

        let cursed_str = if self.cursed_known && self.cursed {
            " (诅咒)"
        } else {
            ""
        };

        format!(
            "{}{}{}{}",
            self.tier_name(),
            level_str,
            glyph_str,
            cursed_str
        )
    }

    /// 获取护甲颜色（用于TUI显示）
    pub fn color(&self) -> Color {
        if self.cursed_known && self.cursed {
            Color::LightRed
        } else {
            match self.tier {
                1 => Color::White,
                2 => Color::LightYellow,
                3 => Color::LightBlue,
                4 => Color::LightGreen,
                5 => Color::LightMagenta,
                _ => Color::Gray,
            }
        }
    }

    /// 计算实际防御值（考虑强化等级和诅咒）
    pub fn effective_defense(&self) -> i32 {
        let mut defense = self.defense + self.upgrade_level as i32;
        if self.cursed {
            defense = (defense as f32 * 0.67).floor() as i32; // 诅咒减少33%防御
        }
        defense.max(1) // 防御值至少为1
    }

    /// 升级护甲（参考游戏升级机制）
    pub fn upgrade(&mut self) {
        self.upgrade_level += 1;
        // 升级不会自动解除诅咒
    }

    /// 添加/改变刻印（不会自动解除诅咒）
    pub fn inscribe(&mut self, glyph: ArmorGlyph) {
        self.glyph = Some(glyph);
    }

    /// 添加诅咒
    pub fn curse(&mut self) {
        self.cursed = true;
        // 诅咒状态默认为未知
        self.cursed_known = false;
    }

    /// 鉴定诅咒状态
    pub fn identify_curse(&mut self) {
        self.cursed_known = true;
    }

    /// 解除诅咒
    pub fn remove_curse(&mut self) {
        self.cursed = false;
        self.cursed_known = true;
    }

    /// 触发刻印效果（简化版）
    pub fn trigger_glyph(&self) -> Option<GlyphEffect> {
        self.glyph.as_ref().map(|glyph| match glyph {
            ArmorGlyph::Thorns => GlyphEffect::ReflectDamage(1 + self.upgrade_level as i32 / 3),
            ArmorGlyph::Repulsion => GlyphEffect::Knockback(2),
            ArmorGlyph::Affection => GlyphEffect::Charm(10 + self.upgrade_level as i32 * 5),
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
    Charm(i32),         // 魅惑概率(百分比)
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


impl Default for Armor {
    fn default() -> Self {
        Armor {
            tier: 1,                // Default to lowest tier
            defense: Self::base_defense(1),
            upgrade_level: 0,
            glyph: None,
            cursed: false,
            cursed_known: false,
            str_requirement: Self::base_str_requirement(1),
            base_value: Self::base_value(1),
        }
    }
}
