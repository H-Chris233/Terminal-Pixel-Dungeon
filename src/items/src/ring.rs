use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 戒指系统（基于破碎的像素地牢v1.0.1设计）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Ring {
    pub kind: RingKind,
    pub level: i32,   // 戒指等级(0-10)
    pub cursed: bool, // 是否被诅咒
    pub identified: bool,
    pub base_value: usize,
}

impl Ring {
    /// 创建新戒指（可指定等级，默认未鉴定、未诅咒）
    pub fn new(kind: RingKind, level: i32) -> Self {
        Self {
            kind: kind.clone(),
            level: level.clamp(0, 10),
            cursed: false,
            identified: false,
            base_value: Self::base_value_for_kind(&kind), // 根据戒指类型设置基础价值
        }
    }

    /// 创建诅咒戒指（陷阱或特殊房间生成）
    pub fn new_cursed(kind: RingKind, level: i32) -> Self {
        Self {
            kind: kind.clone(),
            level: level.clamp(0, 10),
            cursed: true,
            identified: false,
            base_value: Self::base_value_for_kind(&kind), // 同样设置基础价值
        }
    }

    /// 根据戒指类型获取基础价值
    fn base_value_for_kind(kind: &RingKind) -> usize {
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

    /// 获取戒指的完整价值（考虑等级和诅咒状态）
    pub fn value(&self) -> usize {
        let mut value = self.base_value;

        // 等级加成（每级+20%基础价值）
        if self.level > 0 {
            value += (self.base_value as f32 * 0.2 * self.level as f32) as usize;
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
}

/// 戒指类型枚举
#[derive(Eq, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum RingKind {
    Accuracy,      // 提升命中率
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
