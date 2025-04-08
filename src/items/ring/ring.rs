use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 戒指系统（基于破碎的像素地牢v1.0.1设计）
/// 每种戒指提供不同的增益效果，可通过升级强化
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Ring {
    pub kind: RingKind,
    pub level: i32,   // 戒指等级(0-10，与游戏平衡性匹配)
    pub cursed: bool, // 是否被诅咒（影响装备效果）
}

impl Ring {
    /// 创建新戒指（默认未诅咒）
    pub fn new(kind: RingKind, level: i32) -> Self {
        Ring {
            kind,
            level: level.clamp(0, 10), // 限制等级范围
            cursed: false,
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

    /// 计算戒指提供的实际增益效果（考虑等级和诅咒状态）
    pub fn effect_value(&self, base_value: f32) -> f32 {
        let multiplier = match self.kind {
            // 线性增长的戒指效果（每级+20%）
            RingKind::Accuracy | RingKind::Evasion | RingKind::Sharpshooting => {
                1.0 + 0.2 * self.level as f32
            }
            // 递减增长的戒指效果（防止后期过强）
            RingKind::Elements | RingKind::Energy | RingKind::Might => {
                1.0 + 0.15 * (self.level as f32).sqrt()
            }
            // 特殊增长曲线
            RingKind::Force => 1.0 + 0.1 * self.level as f32,
            RingKind::Furor => 1.0 + 0.25 * (self.level as f32).log2(),
            RingKind::Haste => 1.0 + 0.05 * self.level as f32,
            RingKind::Wealth => 1.0 + 0.3 * (self.level as f32).powf(0.5),
        };

        if self.cursed {
            base_value * multiplier * 0.5 // 诅咒效果减半
        } else {
            base_value * multiplier
        }
    }
}

/// 戒指类型枚举（10种，与游戏原版一致）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum RingKind {
    Accuracy,      // 提升命中率（影响近战/远程攻击）
    Elements,      // 元素抗性（火焰/冰冻/闪电等）
    Energy,        // 能量恢复（加快魔杖充能）
    Evasion,       // 闪避几率（降低被命中率）
    Force,         // 击退效果（增加击退距离）
    Furor,         // 攻击速度（减少攻击间隔）
    Haste,         // 移动速度（增加每回合移动距离）
    Might,         // 力量增益（相当于力量药水效果）
    Sharpshooting, // 远程伤害（提升投掷武器/远程伤害）
    Wealth,        // 财富掉落（增加金币和物品掉落）
}

impl RingKind {
    /// 获取戒指类型的默认等级（生成时使用）
    pub fn default_level(&self) -> i32 {
        match self {
            // 稀有戒指初始等级较低
            RingKind::Wealth | RingKind::Might => 0,
            // 普通戒指初始等级1-3
            _ => (1..=3).next().unwrap_or(1),
        }
    }

    /// 获取戒指的升级权重（影响升级概率）
    pub fn upgrade_weight(&self) -> f32 {
        match self {
            // 稀有戒指更难升级
            RingKind::Wealth => 0.5,
            RingKind::Might => 0.7,
            // 普通戒指正常概率
            _ => 1.0,
        }
    }
}

/// 戒指相关测试模块
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_effect_calculation() {
        let mut ring = Ring::new(RingKind::Accuracy, 3);
        assert_eq!(ring.effect_value(100.0), 160.0); // 100 * (1 + 0.2*3)

        ring.cursed = true;
        assert_eq!(ring.effect_value(100.0), 80.0); // 诅咒效果减半
    }

    #[test]
    fn test_ring_level_clamping() {
        let ring = Ring::new(RingKind::Might, 15);
        assert_eq!(ring.level, 10); // 超过最大值会被限制
    }
}
