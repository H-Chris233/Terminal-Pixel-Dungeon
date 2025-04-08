use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 法杖系统（8种法杖，精确还原游戏机制）
#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Wand {
    pub kind: WandKind,
    pub level: i32,      // 强化等级（0-3基础，+3上限）
    pub charges: u8,     // 当前充能
    pub max_charges: u8, // 最大充能（基础1，每级+1）
    pub cursed: bool,    // 是否被诅咒
}

#[derive(PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum WandKind {
    MagicMissile,   // 魔法飞弹（基础法杖）
    Fireblast,      // 火焰冲击
    Frost,          // 寒冰
    Lightning,      // 闪电
    Disintegration, // 瓦解
    Corruption,     // 腐化
    LivingEarth,    // 活体大地
    Regrowth,       // 再生
}

impl Wand {
    /// 获取法杖基础伤害（按游戏内公式计算）
    pub fn base_damage(&self) -> i32 {
        match self.kind {
            WandKind::MagicMissile => 1 + self.level,
            WandKind::Fireblast => 4 + self.level * 2,
            WandKind::Frost => 3 + self.level,
            WandKind::Lightning => 8 + self.level * 3,
            WandKind::Disintegration => 6 + self.level * 2,
            WandKind::Corruption => 0, // 特殊效果
            WandKind::LivingEarth => 2 + self.level,
            WandKind::Regrowth => 0, // 无伤害
        }
    }

    /// 充能恢复（每步恢复逻辑）
    pub fn recharge(&mut self) {
        if self.charges < self.max_charges {
            self.charges += 1;
        }
    }
}
