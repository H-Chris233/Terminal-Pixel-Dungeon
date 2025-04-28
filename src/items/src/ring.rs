//src/items/src/ring.rs
use bincode::serde::encode_to_vec;
use bincode::{Decode, Encode};
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hasher;

use crate::ItemCategory;
use crate::ItemTrait;
use crate::BINCODE_CONFIG;

/// 戒指系统（基于破碎的像素地牢v1.0.1设计）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Ring {
    pub kind: RingKind,
    pub level: i32,   // 戒指等级(0-10)
    pub cursed: bool, // 是否被诅咒
    pub identified: bool,
    pub base_value: u32,
}

impl Ring {
    /// 创建新戒指（可指定等级，默认未鉴定、未诅咒）
    pub fn new(kind: RingKind, level: i32) -> Self {
        Self {
            kind,
            level: level.clamp(0, 10),
            cursed: false,
            identified: false,
            base_value: Self::base_value_for_kind(&kind), // 根据戒指类型设置基础价值
        }
    }

    /// 创建诅咒戒指（陷阱或特殊房间生成）
    pub fn new_cursed(kind: RingKind, level: i32) -> Self {
        Self {
            kind,
            level: level.clamp(0, 10),
            cursed: true,
            identified: false,
            base_value: Self::base_value_for_kind(&kind), // 同样设置基础价值
        }
    }

    /// 根据戒指类型获取基础价值
    fn base_value_for_kind(kind: &RingKind) -> u32 {
        match kind {
            RingKind::Wealth => 2000,        // 财富之戒最有价值
            RingKind::Might => 1500,         // 威力之戒次之
            RingKind::Haste => 1200,         // 急速之戒
            RingKind::Accuracy => 1000,      // 精准之戒
            RingKind::Evasion => 1000,       // 闪避之戒
            RingKind::Sharpshooting => 1000, // 狙击之戒
            RingKind::Elements => 800,       // 元素之戒
            RingKind::Energy => 800,         // 能量之戒
            RingKind::Force => 700,          // 力量之戒
            RingKind::Furor => 600,          // 狂怒之戒
        }
    }

    /// 随机生成新戒指（5%概率为诅咒戒指）
    pub fn random_new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();

        let kinds = [
            RingKind::Accuracy,
            RingKind::Elements,
            RingKind::Energy,
            RingKind::Evasion,
            RingKind::Force,
            RingKind::Furor,
            RingKind::Haste,
            RingKind::Might,
            RingKind::Sharpshooting,
            RingKind::Wealth,
        ];
        let kind = kinds[rng.random_range(0..kinds.len())];
        let level = rng.random_range(0..=3);

        if rng.random_bool(0.05) {
            Ring::new_cursed(kind, level)
        } else {
            Ring::new(kind, level)
        }
    }

    /// 获取戒指的完整价值（考虑等级和诅咒状态）
    pub fn value(&self) -> u32 {
        let mut value = self.base_value;

        // 等级加成（每级+20%基础价值）
        if self.level > 0 {
            value += (self.base_value as f32 * 0.2 * self.level as f32) as u32;
        }

        // 诅咒惩罚（价值减半）
        if self.cursed {
            value /= 2;
        }

        value
    }

    /// 鉴定戒指
    pub fn identify(&mut self) {
        self.identified = true;
    }

    /// 获取显示名称（包含等级信息）
    pub fn name(&self) -> String {
        if !self.identified {
            return if self.cursed {
                "未鉴定的诅咒戒指".to_string()
            } else {
                "未鉴定的戒指".to_string()
            };
        }

        let base = self.base_name();
        if self.level > 0 {
            format!("+{} {}", self.level, base)
        } else {
            base.to_string()
        }
    }

    /// 获取戒指基础名称（不含等级信息）
    pub fn base_name(&self) -> &'static str {
        match self.kind {
            RingKind::Accuracy => "精准之戒",
            RingKind::Elements => "元素之戒",
            RingKind::Energy => "能量之戒",
            RingKind::Evasion => "闪避之戒",
            RingKind::Force => "力量之戒",
            RingKind::Furor => "狂怒之戒",
            RingKind::Haste => "急速之戒",
            RingKind::Might => "威力之戒",
            RingKind::Sharpshooting => "狙击之戒",
            RingKind::Wealth => "财富之戒",
        }
    }

    /// 计算戒指提供的实际增益效果
    pub fn effect_value(&self, base_value: f32) -> f32 {
        let multiplier = match self.kind {
            RingKind::Accuracy | RingKind::Evasion | RingKind::Sharpshooting => {
                1.0 + 0.2 * self.level as f32
            }
            RingKind::Elements | RingKind::Energy | RingKind::Might => {
                1.0 + 0.15 * (self.level as f32).sqrt()
            }
            RingKind::Force => 1.0 + 0.1 * self.level as f32,
            RingKind::Furor => 1.0 + 0.25 * (self.level as f32).log2(),
            RingKind::Haste => 1.0 + 0.05 * self.level as f32,
            RingKind::Wealth => 1.0 + 0.3 * (self.level as f32).powf(0.5),
        };

        if self.cursed {
            base_value * multiplier * 0.5
        } else {
            base_value * multiplier
        }
    }

    /// 更精确的防御加成计算（基于游戏平衡性）
    pub fn defense_bonus(&self) -> f32 {
        if !self.identified || self.cursed {
            return 0.0;
        }

        match self.kind {
            RingKind::Elements => {
                // 元素抗性：每级减少 5% 元素伤害
                // 可以视为等效防御值
                0.5 + (0.05 * self.level as f32).powi(2)
            }
            RingKind::Evasion => {
                // 闪避加成：基础 5% + 每级 2%
                0.05 + (0.02 * self.level as f32)
            }
            RingKind::Might => {
                // 力量加成：每点力量增加 0.5 防御
                // 假设每级提供 0.5 力量
                0.25 * self.level as f32
            }
            RingKind::Force => {
                // 击退效果：每级增加 5% 击退几率
                // 击退成功相当于完全防御一次攻击
                0.05 * self.level as f32
            }
            _ => 0.0,
        }
    }

    pub fn crit_bonus(&self) -> f32 {
        if self.cursed {
            return 0.0;
        }

        match self.kind {
            RingKind::Accuracy => self.level as f32 * 0.015, // 1.5% per level
            RingKind::Sharpshooting => self.level as f32 * 0.02, // 2% per level
            RingKind::Furor => self.level as f32 * 0.012,    // 1.2% per level
            _ => 0.0,
        }
    }
}

/// 戒指类型枚举
#[derive(Copy, Eq, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize, Default)]
pub enum RingKind {
    #[default]
    Accuracy, // 提升命中率
    Elements,      // 元素抗性
    Energy,        // 能量恢复
    Evasion,       // 闪避几率
    Force,         // 击退效果
    Furor,         // 攻击速度
    Haste,         // 移动速度
    Might,         // 力量增益
    Sharpshooting, // 远程伤害
    Wealth,        // 财富掉落
}

impl RingKind {
    /// 获取戒指类型的默认等级
    pub fn default_level(&self) -> i32 {
        match self {
            RingKind::Wealth | RingKind::Might => 0,
            _ => (1..=3).next().unwrap_or(1),
        }
    }

    /// 获取戒指的升级权重
    pub fn upgrade_weight(&self) -> f32 {
        match self {
            RingKind::Wealth => 0.5,
            RingKind::Might => 0.7,
            _ => 1.0,
        }
    }
}

impl Default for Ring {
    fn default() -> Self {
        Ring {
            kind: RingKind::Accuracy, // 默认选择精准之戒（基础类型）
            level: 0,                 // 默认等级0
            cursed: false,            // 默认未诅咒
            identified: false,        // 默认未鉴定
            base_value: 1000,         // 精准之戒的基础价值
        }
    }
}

impl fmt::Display for Ring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 基础信息
        let mut info = self.name().to_string();

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

        // 如果已鉴定，添加详细效果描述
        if self.identified {
            info.push_str(&format!("\n\n效果: {}", self.effect_description()));
        }

        write!(f, "{}", info)
    }
}

impl Ring {
    /// 获取戒指效果的详细描述
    fn effect_description(&self) -> String {
        match self.kind {
            RingKind::Accuracy => {
                format!("提升命中率 {:.0}%", (self.effect_value(1.0) - 1.0) * 100.0)
            }
            RingKind::Elements => "提供元素抗性（火焰、冰冻、闪电）".to_string(),
            RingKind::Energy => format!(
                "加快能量恢复速度 {:.0}%",
                (self.effect_value(1.0) - 1.0) * 100.0
            ),
            RingKind::Evasion => format!(
                "提升闪避几率 {:.0}%",
                (self.effect_value(1.0) - 1.0) * 100.0
            ),
            RingKind::Force => format!(
                "增强击退效果 {:.0}%",
                (self.effect_value(1.0) - 1.0) * 100.0
            ),
            RingKind::Furor => format!(
                "提高攻击速度 {:.0}%",
                (self.effect_value(1.0) - 1.0) * 100.0
            ),
            RingKind::Haste => format!(
                "增加移动速度 {:.0}%",
                (self.effect_value(1.0) - 1.0) * 100.0
            ),
            RingKind::Might => "暂时提升力量属性".to_string(),
            RingKind::Sharpshooting => "增强远程武器伤害".to_string(),
            RingKind::Wealth => "增加金币和物品掉落".to_string(),
        }
    }
}

impl fmt::Display for RingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            RingKind::Accuracy => "精准之戒",
            RingKind::Elements => "元素之戒",
            RingKind::Energy => "能量之戒",
            RingKind::Evasion => "闪避之戒",
            RingKind::Force => "力量之戒",
            RingKind::Furor => "狂怒之戒",
            RingKind::Haste => "急速之戒",
            RingKind::Might => "威力之戒",
            RingKind::Sharpshooting => "狙击之戒",
            RingKind::Wealth => "财富之戒",
        };
        write!(f, "{}", name)
    }
}

impl From<RingKind> for Ring {
    fn from(kind: RingKind) -> Self {
        Ring::new(kind, kind.default_level())
    }
}

impl ItemTrait for Ring {
    /// 生成唯一标识（包含所有属性）
    fn stacking_id(&self) -> u64 {
        let mut hasher = SeaHasher::new();
        let bytes = encode_to_vec(
            &(
                self.kind,
                self.level,
                self.cursed,
                self.identified,
                self.base_value,
            ),
            BINCODE_CONFIG,
        )
        .unwrap();

        hasher.write(&bytes);
        hasher.finish()
    }

    /// 戒指完全不可堆叠
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
        ItemCategory::Ring
    }
    fn sort_value(&self) -> u32 {
        100
    }
}
