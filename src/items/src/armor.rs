//src/items/src/armor.rs
use bincode::{Decode, Encode};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use tui::style::Color;

/// 护甲数据（精确还原游戏机制）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Armor {
    pub tier: u32,               // 品阶1-5
    pub defense: u32,              // 基础防御
    pub upgrade_level: u8,         // 强化等级（非负）
    pub glyph: Option<ArmorGlyph>, // 护甲刻印
    pub cursed: bool,              // 是否被诅咒
    pub cursed_known: bool,        // 是否已鉴定出诅咒状态
    pub str_requirement: u8,      // 力量需求
    pub base_value: u32,         // 基础价值
    pub identified: bool,          // 是否已鉴定
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
    pub fn new(tier: u32) -> Self {
        let defense = Self::base_defense(tier);
        let str_requirement = Self::base_str_requirement(tier);
        let base_value = Self::base_value(tier);

        Armor {
            tier,
            defense,
            upgrade_level: 0,
            glyph: None,
            cursed: false,
            cursed_known: false,
            str_requirement,
            base_value,
            identified: false,
        }
    }
    
    pub fn identify(&mut self) {
        self.identified = true;
        self.cursed_known = true; // 鉴定同时会揭示诅咒状态
    }
    
    /// 随机生成新护甲（随机品阶、刻印和诅咒状态）
    pub fn random_new() -> Self {
        let mut rng = rand::rng();
        let tier = rng.random_range(1..=5);
        let mut armor = Armor::new(tier);
        
        // 15%概率有刻印（原版概率）
        if rng.random_bool(0.15) {
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
        
        // 15%概率被诅咒（原版概率）
        if rng.random_bool(0.15) {
            armor.curse();
        }
        
        armor
    }

    /// 获取基础价值（根据品阶）
    fn base_value(tier: u32) -> u32 {
        match tier {
            1 => 100,  // 布甲
            2 => 250,  // 皮甲
            3 => 500,  // 锁甲
            4 => 1000, // 鳞甲
            5 => 2000, // 板甲
            _ => 0,
        }
    }

    /// 计算护甲完整价值（考虑品阶、等级、刻印和诅咒状态）
    pub fn value(&self) -> u32 {
        let mut value = self.base_value;

        // 等级加成（每级+50%基础价值，原版机制）
        if self.upgrade_level > 0 {
            value += (self.base_value as f32 * 0.5 * self.upgrade_level as f32) as u32;
        }

        // 刻印加成（不同类型有不同加成）
        if let Some(glyph) = &self.glyph {
            value = match glyph {
                ArmorGlyph::Thorns => (value as f32 * 1.5) as u32,    // 荆棘 +50%
                ArmorGlyph::Repulsion => (value as f32 * 1.4) as u32, // 排斥 +40%
                ArmorGlyph::Affection => (value as f32 * 1.3) as u32, // 魅惑 +30%
                ArmorGlyph::Potential => (value as f32 * 1.25) as u32,// 潜能 +25%
                _ => (value as f32 * 1.2) as u32,                     // 其他刻印 +20%
            };
        }

        // 诅咒惩罚（价值减半）
        if self.cursed {
            value /= 2;
        }

        value
    }

    /// 获取基础防御值（根据游戏平衡数据）
    fn base_defense(tier: u32) -> u32 {
        match tier {
            1 => 2,  // 布甲
            2 => 5,  // 皮甲
            3 => 8,  // 锁甲
            4 => 11, // 鳞甲
            5 => 14, // 板甲
            _ => 0,
        }
    }

    fn base_str_requirement(tier: u32) -> u8 {
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
        let level_str = if self.identified && self.upgrade_level > 0 {
            format!("+{}", self.upgrade_level)
        } else {
            String::new()
        };

        let glyph_str = if self.identified {
            match &self.glyph {
                Some(g) => format!(" [{}]", g),
                None => String::new(),
            }
        } else {
            String::new()
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
    
    /// 获取护甲闪避惩罚（基于品阶和强化等级）
    pub fn evasion_penalty(&self) -> u32 {
        // 基础闪避惩罚（品阶越高惩罚越大）
        let base_penalty = match self.tier {
            1 => 0,   // 布甲无惩罚
            2 => 1,   // 皮甲
            3 => 2,   // 锁甲
            4 => 3,   // 鳞甲
            5 => 4,   // 板甲
            _ => 0,
        };

        // 每3级强化减少1点惩罚（最低0）
        let upgrade_reduction = self.upgrade_level as u32 / 3;
        let mut final_penalty = base_penalty - upgrade_reduction;

        // 特殊刻印效果：流动刻印完全消除惩罚
        if let Some(ArmorGlyph::Flow) = self.glyph {
            final_penalty = 0;
        }

        // 诅咒增加50%惩罚（向上取整）
        if self.cursed {
            final_penalty = (final_penalty as f32 * 1.5).ceil() as u32;
        }

        final_penalty // 确保不会返回负数
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
    pub fn defense(&self) -> u32 {
        let mut defense = self.defense + self.upgrade_level as u32;
        if self.cursed {
            defense = (defense as f32 * 0.67).floor() as u32; // 诅咒减少33%防御
        }
        defense.max(1) // 防御值至少为1
    }

    /// 升级护甲（参考游戏升级机制）
    pub fn upgrade(&mut self) {
        self.upgrade_level += 1;
        self.cursed = false;
        self.cursed_known = false;
        // 升级会自动解除诅咒
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
        if !self.identified {
            return None;
        }
        
        self.glyph.as_ref().map(|glyph| match glyph {
            ArmorGlyph::Thorns => GlyphEffect::ReflectDamage(1 + self.upgrade_level as u32 / 3),
            ArmorGlyph::Repulsion => GlyphEffect::Knockback(2),
            ArmorGlyph::Affection => GlyphEffect::Charm(10 + self.upgrade_level as u32 * 5),
            _ => GlyphEffect::None,
        })
    }
}

/// 刻印触发效果
#[derive(Debug, Clone)]
pub enum GlyphEffect {
    None,
    ReflectDamage(u32), // 反弹伤害值
    Knockback(u32),     // 击退距离
    Charm(u32),         // 魅惑概率(百分比)
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
            tier: 1,
            defense: Self::base_defense(1),
            upgrade_level: 0,
            glyph: None,
            cursed: false,
            cursed_known: false,
            str_requirement: Self::base_str_requirement(1),
            base_value: Self::base_value(1),
            identified: false,
        }
    }
}

impl fmt::Display for Armor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 基础信息
        let mut info = format!(
            "{} - 防御: {}",
            self.name(),
            self.defense()
        );

        // 添加力量需求
        info.push_str(&format!("\n力量需求: {}", self.str_requirement));

        // 添加价值信息
        info.push_str(&format!("\n价值: {} 金币", self.value()));

        // 添加鉴定状态
        info.push_str(&format!(
            "\n鉴定状态: {}",
            if self.identified { "已鉴定" } else { "未鉴定" }
        ));

        // 添加诅咒状态（如果已知）
        if self.cursed_known {
            info.push_str(&format!(
                "\n诅咒状态: {}",
                if self.cursed { "已诅咒" } else { "未诅咒" }
            ));
        }

        // 添加刻印描述（如果已鉴定且有刻印）
        if self.identified {
            if let Some(glyph) = &self.glyph {
                info.push_str(&format!("\n刻印效果: {}", glyph_description(glyph)));
            }
        }

        write!(f, "{}", info)
    }
}

/// 辅助函数：生成刻印效果的详细描述
fn glyph_description(glyph: &ArmorGlyph) -> String {
    match glyph {
        ArmorGlyph::Affection => "受到攻击时有概率魅惑敌人".to_string(),
        ArmorGlyph::AntiEntropy => "免疫燃烧和冰冻效果".to_string(),
        ArmorGlyph::Brimstone => "免疫燃烧，受到火焰攻击时恢复生命".to_string(),
        ArmorGlyph::Camouflage => "静止时获得隐身效果".to_string(),
        ArmorGlyph::Flow => "被击退距离翻倍但不受伤害".to_string(),
        ArmorGlyph::Obfuscation => "降低敌人的命中率".to_string(),
        ArmorGlyph::Potential => "充能速度提升".to_string(),
        ArmorGlyph::Repulsion => "击退攻击者".to_string(),
        ArmorGlyph::Stone => "免疫毒气和瘫痪效果".to_string(),
        ArmorGlyph::Thorns => "反弹部分近战伤害".to_string(),
    }
}
