use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 药水系统（完整12种药水）
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Potion {
    pub kind: PotionKind,
    pub identified: bool, // 是否已鉴定
    pub alchemy: bool,    // 是否是炼金产物
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum PotionKind {
    Healing,      // 治疗
    Experience,   // 经验
    ToxicGas,     // 毒气
    ParalyticGas, // 麻痹气体
    LiquidFlame,  // 液态火焰
    Levitation,   // 漂浮
    Invisibility, // 隐身
    Purity,       // 净化
    Frost,        // 霜冻
    Strength,     // 力量
    MindVision,   // 心灵视界
    Haste,        // 急速
}
