//src/items/src/wand.rs
//! 法杖系统模块
//!
//! 实现了破碎的像素地牢中的8种法杖逻辑
//! 注意：所有渲染由其他模块处理，这里只处理数据逻辑

use bincode::serde::encode_to_vec;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::ItemCategory;
use crate::ItemTrait;
use crate::BINCODE_CONFIG;

/// 法杖系统（8种法杖）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Wand {
    /// 法杖种类
    pub kind: WandKind,
    /// 强化等级（0-3基础，可通过卷轴强化至+3）
    pub level: u8,
    /// 当前充能数（初始等于最大充能）
    pub charges: u8,
    /// 最大充能数（瓦解法杖为2+等级，其他为3+等级）
    pub max_charges: u8,
    /// 是否被诅咒（影响使用效果）
    pub cursed: bool,
    /// 是否已鉴定
    pub identified: bool,
}

impl Wand {
    /// 创建新法杖（可指定等级，默认未鉴定、未诅咒）
    pub fn new(kind: WandKind, level: u8) -> Self {
        // 计算最大充能（瓦解法杖2+等级，其他3+等级）
        let max_charges = match kind {
            WandKind::Disintegration => 2 + level,
            _ => 3 + level,
        };

        Self {
            kind,
            level,
            charges: max_charges, // 初始充能等于最大值
            max_charges,
            cursed: false,
            identified: false,
        }
    }

    /// 随机生成新法杖（5%概率为诅咒法杖）
    pub fn random_new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();

        let kinds = [
            WandKind::MagicMissile,
            WandKind::Fireblast,
            WandKind::Frost,
            WandKind::Lightning,
            WandKind::Corruption,
            WandKind::LivingEarth,
            WandKind::Regrowth,
            WandKind::Disintegration,
        ];
        let kind = kinds[rng.random_range(0..kinds.len())];
        let level = rng.random_range(0..=2);

        if rng.random_bool(0.05) {
            Wand::new_cursed(kind, level)
        } else {
            Wand::new(kind, level)
        }
    }

    /// 创建诅咒法杖（陷阱或特殊房间生成）
    pub fn new_cursed(kind: WandKind, level: u8) -> Self {
        let mut wand = Self::new(kind, level);
        wand.cursed = true;
        wand
    }

    /// 计算法杖价值（考虑类型、等级、充能和诅咒状态）
    pub fn value(&self) -> u32 {
        // 基础价值
        let base_value = match self.kind {
            WandKind::Disintegration => 500, // 瓦解法杖最有价值
            WandKind::Corruption => 450,     // 腐化法杖次之
            WandKind::Lightning => 400,
            WandKind::Fireblast => 350,
            WandKind::Frost => 300,
            WandKind::LivingEarth => 300,
            WandKind::Regrowth => 250,
            WandKind::MagicMissile => 200, // 基础法杖价值最低
        };

        // 等级加成（每级+30%基础价值）
        let level_bonus = if self.level > 0 {
            (base_value as f32 * 0.3 * self.level as f32) as u32
        } else {
            0
        };

        // 充能加成（当前充能比例影响价值）
        let charge_ratio = self.charges as f32 / self.max_charges as f32;
        let charge_bonus = (base_value as f32 * 0.2 * charge_ratio) as u32;

        // 计算总价值
        let mut value = base_value + level_bonus + charge_bonus;

        // 诅咒惩罚（价值减半）
        if self.cursed {
            value /= 2;
        }

        // 未鉴定惩罚（价值降低30%）
        if !self.identified {
            value = (value as f32 * 0.7) as u32;
        }

        value
    }

    /// 获取法杖名称（含等级信息）
    pub fn name(&self) -> String {
        if !self.identified {
            return if self.cursed {
                "未鉴定的诅咒法杖".to_string()
            } else {
                "未鉴定的法杖".to_string()
            };
        }

        let base_name = match self.kind {
            WandKind::MagicMissile => "魔法飞弹法杖",
            WandKind::Fireblast => "火焰冲击法杖",
            WandKind::Frost => "寒冰法杖",
            WandKind::Lightning => "闪电法杖",
            WandKind::Corruption => "腐化法杖",
            WandKind::LivingEarth => "活体大地法杖",
            WandKind::Regrowth => "再生法杖",
            WandKind::Disintegration => "瓦解法杖",
        };

        if self.level > 0 {
            format!("+{} {}", self.level, base_name)
        } else {
            base_name.to_string()
        }
    }

    /// 获取基础伤害值（考虑诅咒状态）
    pub fn base_damage(&self) -> u32 {
        let damage = match self.kind {
            WandKind::MagicMissile => 1 + self.level,
            WandKind::Fireblast => 4 + self.level * 2,
            WandKind::Frost => 3 + self.level,
            WandKind::Lightning => 8 + self.level * 3,
            WandKind::Corruption => 0, // 特殊效果无直接伤害
            WandKind::LivingEarth => 2 + self.level,
            WandKind::Regrowth => 0, // 治疗型法杖
            WandKind::Disintegration => 10 + self.level * 4,
        };

        if self.cursed {
            (damage as f32 * 0.7).floor() as u32 // 诅咒效果降低30%
        } else {
            damage.into()
        }
    }

    /// 使用法杖（消耗1充能）
    pub fn use_wand(&mut self) -> bool {
        if self.charges == 0 {
            return false;
        }
        self.charges -= 1;
        true
    }

    /// 自然充能（每回合恢复概率）
    pub fn natural_recharge(&mut self) {
        if self.charges < self.max_charges {
            let recharge_chance = match self.level {
                0 => 0.1,
                1 => 0.15,
                2 => 0.2,
                _ => 0.25,
            };

            if rand::random::<f32>() < recharge_chance {
                self.charges += 1;
            }
        }
    }

    /// 鉴定法杖
    pub fn identify(&mut self) {
        self.identified = true;
    }
}

/// 法杖种类枚举（8种）
#[derive(Copy, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize, Default)]
pub enum WandKind {
    #[default]
    MagicMissile, // 魔法飞弹（基础法杖）
    Fireblast,      // 火焰冲击（范围伤害）
    Frost,          // 寒冰（冻结效果）
    Lightning,      // 闪电（连锁伤害）
    Corruption,     // 腐化（转化敌人）
    LivingEarth,    // 活体大地（召唤石元素）
    Regrowth,       // 再生（治疗和植物生长）
    Disintegration, // 瓦解（穿透性光束，充能上限2+等级）
}

impl Default for Wand {
    fn default() -> Self {
        Wand {
            kind: WandKind::MagicMissile, // 默认选择魔法飞弹法杖（基础类型）
            level: 0,                     // 默认等级0
            charges: 3,                   // 基础充能3
            max_charges: 3,               // 最大充能3
            cursed: false,                // 默认未诅咒
            identified: false,            // 默认未鉴定
        }
    }
}

impl From<WandKind> for Wand {
    fn from(kind: WandKind) -> Self {
        Wand::new(kind, 0) // Default to level 0
    }
}

impl ItemTrait for Wand {
    /// 生成唯一堆叠标识（包含所有关键属性）
    fn stacking_id(&self) -> u64 {
        use seahash::SeaHasher;
        use std::hash::Hasher;

        let mut hasher = SeaHasher::new();
        let bytes = encode_to_vec(
            (
                self.kind,
                self.level,
                self.cursed,
                self.identified,
                self.max_charges, // 包含最大充能数
                self.charges,     // 包含当前充能数
            ),
            BINCODE_CONFIG,
        )
        .unwrap();

        hasher.write(&bytes);
        hasher.finish()
    }

    /// 法杖不可堆叠
    fn is_stackable(&self) -> bool {
        false
    }

    /// 最大堆叠数量固定为1
    fn max_stack(&self) -> u32 {
        1
    }

    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Wand
    }
    fn sort_value(&self) -> u32 {
        // 基础权重 = 等级 * 10（确保高等级法杖优先）
        let level_weight = self.level as u32 * 10;

        // 按法杖稀有度和战略价值分配基础分值
        let type_weight = match self.kind {
            WandKind::Disintegration => 100, // 瓦解法杖（最强攻击）
            WandKind::Corruption => 95,      // 腐化法杖（战略控制）
            WandKind::Lightning => 90,       // 闪电法杖（连锁攻击）
            WandKind::LivingEarth => 85,     // 活体大地（召唤坦克）
            WandKind::Fireblast => 80,       // 火焰冲击（范围伤害）
            WandKind::Frost => 75,           // 寒冰法杖（控场能力）
            WandKind::Regrowth => 70,        // 再生法杖（治疗辅助）
            WandKind::MagicMissile => 65,    // 魔法飞弹（基础法杖）
        };

        // 诅咒惩罚（降低排序优先级）
        let curse_penalty = if self.cursed { 50 } else { 0 };

        // 充能状态加成（满充能提升优先级）
        let charge_bonus = if self.charges == self.max_charges {
            (self.max_charges as u32) * 2
        } else {
            0
        };

        // 最终计算公式（确保类型权重主导排序）
        (type_weight * 100) + level_weight + charge_bonus - curse_penalty
    }
}
