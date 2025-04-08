use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 戒指系统（10种戒指）
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Ring {
    pub kind: RingKind,
    pub level: i32,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum RingKind {
    Accuracy,      // 精准
    Elements,      // 元素
    Energy,        // 能量
    Evasion,       // 闪避
    Force,         // 力量
    Furor,         // 狂怒
    Haste,         // 急速
    Might,         // 威力
    Sharpshooting, // 狙击
    Wealth,        // 财富
}
