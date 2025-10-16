//src/items/src/weapon.rs
use bincode::serde::encode_to_vec;
use bincode::{Decode, Encode};
use rand::Rng;
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::hash::Hasher;

use crate::BINCODE_CONFIG;
use crate::Item;
use crate::ItemCategory;
use crate::ItemKind;
use crate::ItemTrait;

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
    pub damage: (u32, u32),               // 基础伤害范围(min,max)
    pub hit_chance: f32,                  // 基础命中率(0.0-1.0)
    pub str_requirement: u8,              // 力量需求
    pub enchanted: Option<WeaponEnhance>, // 附魔效果
    pub modifier: WeaponMod,              // 改造方向
    pub upgrade_level: u8,                // 强化等级（+1,+2等）
    pub cursed: bool,                     // 是否被诅咒
    pub identified: bool,                 // 是否已鉴定
    pub kind: WeaponKind,
    pub base_value: u32,
}

impl Weapon {
    /// 创建新武器（根据品阶初始化基础属性）
    pub fn new(tier: u32, kind: WeaponKind) -> Self {
        let (damage, str_req, name) = match tier {
            1 => ((1, 6), 10, "短剑"),     // 一阶武器
            2 => ((3, 12), 13, "长剑"),    // 二阶武器
            3 => ((6, 18), 16, "巨剑"),    // 三阶武器
            4 => ((10, 25), 19, "符文剑"), // 四阶武器
            5 => ((15, 35), 22, "圣剑"),   // 五阶武器
            _ => {
                return Weapon {
                    name: "Unknown".to_string(),
                    tier: Tier::One,
                    damage: (1, 1),
                    hit_chance: 0.5,
                    str_requirement: 0,
                    enchanted: None,
                    modifier: WeaponMod::Balanced,
                    upgrade_level: 0,
                    cursed: false,
                    identified: false,
                    kind: WeaponKind::Sword,
                    base_value: 0,
                };
            }
        };

        Self {
            name: name.to_string(),
            tier: Tier::from_u32(tier),
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
    fn base_value_for_tier(tier: u32) -> u32 {
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
    pub fn value(&self) -> u32 {
        let mut value = self.base_value;

        // 等级加成（每级+25%基础价值）
        if self.upgrade_level > 0 {
            value += (self.base_value as f32 * 0.25 * self.upgrade_level as f32) as u32;
        }

        // 附魔加成（不同类型有不同加成）
        if let Some(enhance) = &self.enchanted {
            value = match enhance {
                WeaponEnhance::Burning => (value as f32 * 1.2) as u32,
                WeaponEnhance::Stunning => (value as f32 * 1.3) as u32,
                WeaponEnhance::Vampiric => (value as f32 * 1.4) as u32,
                WeaponEnhance::Lucky => (value as f32 * 1.25) as u32,
                WeaponEnhance::Projecting => (value as f32 * 1.15) as u32,
                WeaponEnhance::Grim => (value as f32 * 1.5) as u32,
                WeaponEnhance::Chilling => (value as f32 * 1.1) as u32,
            };
        }

        // 改造加成
        value = match self.modifier {
            WeaponMod::Damage => (value as f32 * 1.1) as u32,
            WeaponMod::Speed => (value as f32 * 1.05) as u32,
            WeaponMod::Accuracy => (value as f32 * 1.15) as u32,
            WeaponMod::Balanced => (value as f32 * 1.2) as u32,
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
        if self.upgrade_level.is_multiple_of(3) {
            self.str_requirement = (self.str_requirement - 1).max(1);
        }

        // 伤害成长公式：每级最小伤害+1，最大伤害+2
        self.damage.0 += 1;
        self.damage.1 += 2;

        // 命中率小幅提升
        self.hit_chance = (self.hit_chance + 0.01).min(0.95);
    }

    /// 计算实际伤害（还原力量加成和诅咒惩罚）
    pub fn calculate_damage(&self, user_str: u8) -> u32 {
        let str_diff = user_str as i32 - self.str_requirement as i32;
        let mut damage = rand::rng().random_range(self.damage.0..=self.damage.1);

        // 力量修正（每点差异影响5%伤害）
        match str_diff.cmp(&0) {
            Ordering::Less => {
                damage = (damage as f32 * (0.9f32).powi(-str_diff)) as u32;
            }
            Ordering::Greater => {
                damage += (damage as f32 * 0.05 * str_diff as f32) as u32;
            }
            _ => {}
        }

        // 改造修正
        damage = match self.modifier {
            WeaponMod::Damage => (damage as f32 * 1.3) as u32,
            WeaponMod::Speed => (damage as f32 * 0.8) as u32,
            WeaponMod::Accuracy => damage,
            WeaponMod::Balanced => (damage as f32 * 1.1) as u32,
        };

        // 诅咒惩罚（降低30%伤害）
        if self.cursed {
            damage = (damage as f32 * 0.7) as u32;
        }

        damage.max(1)
    }

    /// 获取武器暴击加成（基于武器类型和附魔效果）
    pub fn crit_bonus(&self) -> f32 {
        let base_bonus = match self.kind {
            WeaponKind::Dagger => 0.15, // 匕首有更高暴击率
            WeaponKind::Sword => 0.05,
            WeaponKind::Greataxe => 0.10, // 巨斧有较高暴击伤害
            WeaponKind::Spear => 0.03,
            WeaponKind::Mace => 0.0,
            WeaponKind::Whip => 0.07,
        };

        // 幸运附魔增加暴击率
        if let Some(WeaponEnhance::Lucky) = self.enchanted {
            base_bonus + 0.15
        } else {
            base_bonus
        }
    }

    /// 添加随机附魔（还原Shattered PD的附魔概率）
    pub fn add_random_enhancement(&mut self) {
        let enhancements = [
            WeaponEnhance::Burning,    // 25%概率点燃敌人3回合
            WeaponEnhance::Stunning,   // 20%概率眩晕敌人2回合
            WeaponEnhance::Vampiric,   // 恢复造成伤害的10%
            WeaponEnhance::Lucky,      // 暴击率+15%
            WeaponEnhance::Projecting, // 攻击距离+1
            WeaponEnhance::Grim,       // 对生命值低于20%的敌人必杀
            WeaponEnhance::Chilling,
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

    /// 获取武器伤害加成（基于强化等级和改造方向）
    pub fn damage_bonus(&self) -> u32 {
        let base_bonus = self.upgrade_level as u32;

        match self.modifier {
            WeaponMod::Damage => (base_bonus as f32 * 1.3).round() as u32,
            WeaponMod::Speed => (base_bonus as f32 * 0.8).round() as u32,
            WeaponMod::Accuracy => base_bonus,
            WeaponMod::Balanced => (base_bonus as f32 * 1.1).round() as u32,
        }
    }

    /// 获取武器命中加成（基于武器类型和改造方向）
    pub fn accuracy_bonus(&self) -> i32 {
        let base_bonus = match self.kind {
            WeaponKind::Sword => 0,
            WeaponKind::Dagger => 1,
            WeaponKind::Greataxe => -1,
            WeaponKind::Spear => 2,
            WeaponKind::Mace => -2,
            WeaponKind::Whip => 3,
        };

        base_bonus
            + match self.modifier {
                WeaponMod::Damage => 0,
                WeaponMod::Speed => 1,
                WeaponMod::Accuracy => 3,
                WeaponMod::Balanced => 1,
            }
    }

    /// 获取武器攻击距离
    pub fn range(&self) -> u32 {
        let base_range = match self.kind {
            WeaponKind::Spear => 2,
            _ => 1,
        };

        // 如果有投射附魔则增加1距离
        if let Some(WeaponEnhance::Projecting) = self.enchanted {
            base_range + 1
        } else {
            base_range
        }
    }

    /// 判断是否为远程武器
    pub fn is_ranged(&self) -> bool {
        self.range() > 1
    }
}

/// 武器类型枚举（还原Shattered PD的武器分类）
#[derive(Copy, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize, Default)]
pub enum WeaponKind {
    #[default]
    Sword, // 剑类：平衡型
    Dagger,   // 匕首：高速低伤（+25%攻速）
    Greataxe, // 巨斧：低速高伤（+30%伤害）
    Spear,    // 长矛：中距攻击（攻击范围+1）
    Mace,     // 钉锤：破甲效果
    Whip,     // 长鞭：高命中
}

/// 武器品阶（还原Shattered PD的5阶系统）
#[derive(Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize, Clone, Default)]
pub enum Tier {
    #[default]
    One, // 普通（白色）
    Two,   // 优秀（绿色）
    Three, // 稀有（蓝色）
    Four,  // 史诗（紫色）
    Five,  // 传奇（橙色）
}

impl Tier {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Tier::One,
            2 => Tier::Two,
            3 => Tier::Three,
            4 => Tier::Four,
            5 => Tier::Five,
            _ => Tier::One,
        }
    }

    pub fn to_u32(&self) -> u32 {
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
            tier: Tier::One,               // 一阶武器
            damage: (1, 6),                // 基础伤害1-6
            hit_chance: 0.8,               // 80%命中率
            str_requirement: 10,           // 力量需求10
            enchanted: None,               // 默认无附魔
            modifier: WeaponMod::Balanced, // 平衡改造
            upgrade_level: 0,              // 未强化
            cursed: false,                 // 未诅咒
            identified: false,             // 未鉴定
            kind: WeaponKind::Sword,       // 剑类武器
            base_value: 300,               // 一阶武器基础价值
        }
    }
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 基础信息
        let mut info = format!(
            "{} ({}阶) - 伤害: {}-{}",
            self.name,
            self.tier.to_u32(),
            self.damage.0,
            self.damage.1
        );

        // 添加强化等级（如果已强化）
        if self.upgrade_level > 0 {
            info.push_str(&format!(" (+{})", self.upgrade_level));
        }

        // 添加武器类型
        info.push_str(&format!("\n类型: {}", self.kind));

        // 添加命中率
        info.push_str(&format!("\n命中率: {:.0}%", self.hit_chance * 100.0));

        // 添加力量需求
        info.push_str(&format!("\n力量需求: {}", self.str_requirement));

        // 添加改造方向
        info.push_str(&format!("\n改造方向: {}", self.modifier));

        // 添加附魔效果（如果已鉴定且有附魔）
        if self.identified
            && let Some(enhance) = &self.enchanted
        {
            info.push_str(&format!("\n附魔效果: {}", enhance));
        }

        // 添加价值信息
        info.push_str(&format!("\n价值: {} 金币", self.value()));

        // 添加鉴定状态
        info.push_str(&format!(
            "\n鉴定状态: {}",
            if self.identified {
                "已鉴定"
            } else {
                "未鉴定"
            }
        ));

        // 添加诅咒状态
        info.push_str(&format!(
            "\n诅咒状态: {}",
            if self.cursed {
                "已诅咒"
            } else {
                "未诅咒"
            }
        ));

        write!(f, "{}", info)
    }
}

impl fmt::Display for WeaponKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            WeaponKind::Sword => "剑类",
            WeaponKind::Dagger => "匕首",
            WeaponKind::Greataxe => "巨斧",
            WeaponKind::Spear => "长矛",
            WeaponKind::Mace => "钉锤",
            WeaponKind::Whip => "长鞭",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for WeaponMod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            WeaponMod::Damage => "伤害强化",
            WeaponMod::Speed => "速度强化",
            WeaponMod::Accuracy => "精准强化",
            WeaponMod::Balanced => "平衡改造",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for WeaponEnhance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            WeaponEnhance::Burning => "燃烧",
            WeaponEnhance::Stunning => "击晕",
            WeaponEnhance::Vampiric => "吸血",
            WeaponEnhance::Lucky => "幸运",
            WeaponEnhance::Projecting => "投射",
            WeaponEnhance::Grim => "致命",
            WeaponEnhance::Chilling => "冰冻",
        };
        write!(f, "{}", name)
    }
}

impl From<(u32, WeaponKind)> for Weapon {
    fn from((tier, kind): (u32, WeaponKind)) -> Self {
        Weapon::new(tier, kind)
    }
}

impl ItemTrait for Weapon {
    /// 武器始终不可堆叠
    fn is_stackable(&self) -> bool {
        false
    }

    /// 生成唯一标识（虽然不可堆叠但仍需实现）
    fn stacking_id(&self) -> u64 {
        // 包含所有属性的哈希计算
        let mut hasher = SeaHasher::new();
        let bytes = encode_to_vec(
            (
                self.tier.to_u32(),
                self.upgrade_level,
                &self.enchanted,
                &self.modifier,
                self.cursed,
                self.kind,
                self.damage,
                self.str_requirement,
                self.base_value,
                self.hit_chance.to_bits(), // 精确处理浮点数
            ),
            BINCODE_CONFIG,
        )
        .unwrap();

        hasher.write(&bytes);
        hasher.finish()
    }

    /// 最大堆叠数量始终为1
    fn max_stack(&self) -> u32 {
        1
    }
    fn display_name(&self) -> String {
        self.name.clone()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Weapon
    }
    fn sort_value(&self) -> u32 {
        (self.tier as u32 * 100) + self.upgrade_level as u32
    }
}

impl From<Weapon> for Item {
    fn from(weapon: Weapon) -> Self {
        Item {
            kind: ItemKind::Weapon(weapon),
            // 其他必要的字段
            ..Default::default()
        }
    }
}
