use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 护甲数据（精确还原游戏机制）
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Armor {
    pub name: String,
    pub tier: usize,               // 品阶1-5
    pub defense: i32,              // 基础防御
    pub glyph: Option<ArmorGlyph>, // 护甲刻印
}

/// 护甲刻印类型（全部10种）
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum ArmorGlyph {
    Affection,   // 魅惑
    AntiEntropy, // 抗熵
    Brimstone,   // 硫磺
    Camouflage,  // 伪装
    Flow,        // 流动
    Obfuscation, // 混淆
    Potential,   // 潜能
    Repulsion,   // 排斥
    Stone,       // 石肤
    Thorns,      // 荆棘
}
