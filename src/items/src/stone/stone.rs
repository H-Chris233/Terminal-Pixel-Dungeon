use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 魔法石系统（6种符文石）
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Stone {
    pub kind: StoneKind,
    pub charges: u8, // 使用次数（部分石头可重复使用）
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum StoneKind {
    Aggression,   // 侵略（吸引敌人）
    Blink,        // 闪烁（短距传送）
    Clairvoyance, // 透视（显示地图）
    Affection,    // 情感（魅惑敌人）
    Shock,        // 震荡（击退敌人）
    DeepSleep,    // 沉眠（群体催眠）
}

impl Stone {
    /// 获取使用效果（游戏内机制）
    pub fn use_effect(&mut self) -> String {
        self.charges = self.charges.saturating_sub(1);
        match self.kind {
            StoneKind::Aggression => "使敌人互相攻击".to_string(),
            StoneKind::Blink => "传送至视线范围内位置".to_string(),
            StoneKind::Clairvoyance => "显示整层地图".to_string(),
            StoneKind::Affection => "魅惑敌人10回合".to_string(),
            StoneKind::Shock => "击退周围敌人".to_string(),
            StoneKind::DeepSleep => "使敌人陷入沉睡".to_string(),
        }
    }

    /// 是否可重复使用
    pub fn is_reusable(&self) -> bool {
        matches!(
            self.kind,
            StoneKind::Aggression | StoneKind::Affection | StoneKind::Shock
        )
    }
}
