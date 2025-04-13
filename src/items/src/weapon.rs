//src/items/src/weapon.rs
use crate::weapon::kind::*;
use bincode::{Decode, Encode};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod kind;
pub mod tier;

/// 武器附魔效果（完全还原Shattered PD的附魔系统）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum WeaponEnhance {
    // 常规附魔
    Burning,    // 燃烧：25%概率点燃敌人3回合
    Stunning,   // 击晕：20%概率眩晕敌人2回合
    Vampiric,   // 吸血：恢复造成伤害的10%
    Lucky,      // 幸运：暴击率+15%
    Projecting, // 投射：攻击距离+1
    Grim,       // 致命：对生命值低于20%的敌人必杀
    Chilling,   // 冰冻：减速敌人
}

/// 武器改造方向（还原Shattered PD的武器改造系统）
#[derive(PartialEq, Debug, Encode, Decode, Serialize, Deserialize, Clone, Default)]
pub enum WeaponMod {
    Damage,   // 偏向伤害（+30%伤害，-15%攻速）
    Speed,    // 偏向速度（+25%攻速，-20%伤害）
    Accuracy, // 偏向精准（+20%命中率）

    #[default]
    Balanced, // 平衡改造（+10%伤害和攻速）
}

/// 武器数据结构（完全还原Shattered PD武器属性）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Weapon {
    pub name: String,                     // 武器名称
    pub tier: Tier,                       // 品阶1-5
    pub damage: (usize, usize),           // 基础伤害范围(min,max)
    pub hit_chance: f32,                  // 基础命中率(0.0-1.0)
    pub str_requirement: u8,              // 力量需求
    pub enchanted: Option<WeaponEnhance>, // 附魔效果
    pub modifier: WeaponMod,              // 改造方向
    pub upgrade_level: u8,                // 强化等级（+1,+2等）
    pub cursed: bool,                     // 是否被诅咒
    pub identified: bool,                 // 是否已鉴定
    pub kind: WeaponKind,
    pub base_value: usize,
}

impl Weapon {
    /// 创建新武器（根据品阶初始化基础属性）
    pub fn new(tier: usize, kind: WeaponKind) -> Self {
        let (damage, str_req, name) = match tier {
            1 => ((1, 6), 10, "短剑"),     // 一阶武器
            2 => ((3, 12), 13, "长剑"),    // 二阶武器
            3 => ((6, 18), 16, "巨剑"),    // 三阶武器
            4 => ((10, 25), 19, "符文剑"), // 四阶武器
            5 => ((15, 35), 22, "圣剑"),   // 五阶武器
            _ => panic!("Invalid weapon tier"),
        };

        Self {
            name: name.to_string(),
            tier: Tier::from_usize(tier),
            damage,
            hit_chance: 0.8,
            str_requirement: str_req,
            enchanted: None,
            modifier: WeaponMod::Accuracy,
            upgrade_level: 0,
            cursed: false,
            identified: false,
            kind,
            base_value: Self::base_value_for_tier(tier), // 根据品阶设置基础价值
        }
    }
    
    /// 随机生成新武器（5%概率为诅咒武器）
    pub fn random_new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        
        let tier = rng.random_range(1..=5);
        let kinds = [
            WeaponKind::Sword,
            WeaponKind::Dagger,
            WeaponKind::Greataxe,
            WeaponKind::Spear,
            WeaponKind::Mace,
            WeaponKind::Whip,
        ];
        let kind = kinds[rng.random_range(0..kinds.len())];
        
        let mut weapon = Weapon::new(tier, kind);
        
        // 10%概率有附魔
        if rng.random_bool(0.1) {
            weapon.add_random_enhancement();
        }
        
        // 随机改造方向
        let mods = [
            WeaponMod::Damage,
            WeaponMod::Speed,
            WeaponMod::Accuracy,
            WeaponMod::Balanced,
        ];
        weapon.modifier = mods[rng.random_range(0..mods.len())].clone();
        
        // 5%概率被诅咒
        if rng.random_bool(0.05) {
            weapon.cursed = true;
        }
        
        weapon
    }

    /// 根据品阶获取基础价值
    fn base_value_for_tier(tier: usize) -> usize {
        match tier {
            1 => 300,  // 一阶武器
            2 => 600,  // 二阶武器
            3 => 1200, // 三阶武器
            4 => 2400, // 四阶武器
            5 => 4800, // 五阶武器
            _ => 0,
        }
    }

    /// 获取武器完整价值（考虑品阶、等级、附魔和改造）
    pub fn value(&self) -> usize {
        let mut value = self.base_value;

        // 等级加成（每级+25%基础价值）
        if self.upgrade_level > 0 {
            value += (self.base_value as f32 * 0.25 * self.upgrade_level as f32) as usize;
        }

        // 附魔加成（不同类型有不同加成）
        if let Some(enhance) = &self.enchanted {
            value = match enhance {
                WeaponEnhance::Burning => (value as f32 * 1.2) as usize,
                WeaponEnhance::Stunning => (value as f32 * 1.3) as usize,
                WeaponEnhance::Vampiric => (value as f32 * 1.4) as usize,
                WeaponEnhance::Lucky => (value as f32 * 1.25) as usize,
                WeaponEnhance::Projecting => (value as f32 * 1.15) as usize,
                WeaponEnhance::Grim => (value as f32 * 1.5) as usize,
                WeaponEnhance::Chilling => (value as f32 * 1.1) as usize,
            };
        }

        // 改造加成
        value = match self.modifier {
            WeaponMod::Damage => (value as f32 * 1.1) as usize,
            WeaponMod::Speed => (value as f32 * 1.05) as usize,
            WeaponMod::Accuracy => (value as f32 * 1.15) as usize,
            WeaponMod::Balanced => (value as f32 * 1.2) as usize,
        };

        // 诅咒惩罚（价值减半）
        if self.cursed {
            value /= 2;
        }

        value
    }

    /// 强化武器（还原Shattered PD的强化系统）
    pub fn upgrade(&mut self) {
        self.upgrade_level += 1;

        // 每3级降低1点力量需求（最低1）
        if self.upgrade_level % 3 == 0 {
            self.str_requirement = (self.str_requirement - 1).max(1);
        }

        // 伤害成长公式：每级最小伤害+1，最大伤害+2
        self.damage.0 += 1;
        self.damage.1 += 2;

        // 命中率小幅提升
        self.hit_chance = (self.hit_chance + 0.01).min(0.95);
    }

    /// 计算实际伤害（还原力量加成和诅咒惩罚）
    pub fn calculate_damage(&self, user_str: u8) -> usize {
        let str_diff = user_str as i32 - self.str_requirement as i32;
        let mut damage = rand::rng().random_range(self.damage.0..=self.damage.1);

        // 力量修正（每点差异影响5%伤害）
        if str_diff > 0 {
            damage += (damage as f32 * 0.05 * str_diff as f32) as usize;
        } else if str_diff < 0 {
            damage = (damage as f32 * (0.9f32).powi(-str_diff)) as usize;
        }

        // 改造修正
        damage = match self.modifier {
            WeaponMod::Damage => (damage as f32 * 1.3) as usize,
            WeaponMod::Speed => (damage as f32 * 0.8) as usize,
            WeaponMod::Accuracy => damage,
            WeaponMod::Balanced => (damage as f32 * 1.1) as usize,
        };

        // 诅咒惩罚（降低30%伤害）
        if self.cursed {
            damage = (damage as f32 * 0.7) as usize;
        }

        damage.max(1)
    }

    /// 添加随机附魔（还原Shattered PD的附魔概率）
    pub fn add_random_enhancement(&mut self) {
        let enhancements = vec![
            WeaponEnhance::Burning,    // 25%概率点燃敌人3回合
            WeaponEnhance::Stunning,   // 20%概率眩晕敌人2回合
            WeaponEnhance::Vampiric,   // 恢复造成伤害的10%
            WeaponEnhance::Lucky,      // 暴击率+15%
            WeaponEnhance::Projecting, // 攻击距离+1
            WeaponEnhance::Grim,       // 对生命值低于20%的敌人必杀
            WeaponEnhance::Chilling,   // 减速敌人
        ];

        // 确保100%获得一个随机附魔
        self.enchanted =
            Some(enhancements[rand::rng().random_range(0..enhancements.len())].clone());
    }

    /// 武器鉴定逻辑（还原Shattered PD的鉴定机制）
    pub fn identify(&mut self) {
        self.identified = true;
    }

    /// 改造武器（还原Shattered PD的改造系统）
    pub fn modify(&mut self, direction: WeaponMod) {
        self.modifier = direction;
    }
}

/// 武器类型枚举（还原Shattered PD的武器分类）
#[derive(Copy, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum WeaponKind {
    Sword,    // 剑类：平衡型
    Dagger,   // 匕首：高速低伤（+25%攻速）
    Greataxe, // 巨斧：低速高伤（+30%伤害）
    Spear,    // 长矛：中距攻击（攻击范围+1）
    Mace,     // 钉锤：破甲效果
    Whip,     // 长鞭：高命中
}

/// 武器品阶（还原Shattered PD的5阶系统）
#[derive(Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub enum Tier {
    One,   // 普通（白色）
    Two,   // 优秀（绿色）
    Three, // 稀有（蓝色）
    Four,  // 史诗（紫色）
    Five,  // 传奇（橙色）
}

impl Tier {
    pub fn from_usize(value: usize) -> Self {
        match value {
            1 => Tier::One,
            2 => Tier::Two,
            3 => Tier::Three,
            4 => Tier::Four,
            5 => Tier::Five,
            _ => panic!("Invalid tier value"),
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            Tier::One => 1,
            Tier::Two => 2,
            Tier::Three => 3,
            Tier::Four => 4,
            Tier::Five => 5,
        }
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Tier::One => "I",
                Tier::Two => "II",
                Tier::Three => "III",
                Tier::Four => "IV",
                Tier::Five => "V",
            }
        )
    }
}

impl Default for Weapon {
    fn default() -> Self {
        Weapon {
            name: "短剑".to_string(),
            tier: Tier::One,              // 一阶武器
            damage: (1, 6),              // 基础伤害1-6
            hit_chance: 0.8,             // 80%命中率
            str_requirement: 10,         // 力量需求10
            enchanted: None,             // 默认无附魔
            modifier: WeaponMod::Balanced, // 平衡改造
            upgrade_level: 0,            // 未强化
            cursed: false,               // 未诅咒
            identified: false,           // 未鉴定
            kind: WeaponKind::Sword,     // 剑类武器
            base_value: 300,             // 一阶武器基础价值
        }
    }
}

impl Default for WeaponKind {
    fn default() -> Self {
        WeaponKind::Sword  // 默认剑类武器
    }
}

impl Default for Tier {
    fn default() -> Self {
        Tier::One  // 默认一阶
    }
}
