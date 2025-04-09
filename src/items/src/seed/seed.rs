use crate::potion::potion::{Potion, PotionKind};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 种子系统（8种植物种子）
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Seed {
    pub kind: SeedKind,
    pub turns_to_grow: u8, // 生长所需回合数
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum SeedKind {
    Earthroot,  // 地根草（护盾）
    Fadeleaf,   // 消隐叶（传送）
    Firebloom,  // 火焰花（燃烧）
    Icecap,     // 冰帽草（冻结）
    Sorrowmoss, // 哀伤苔（中毒）
    Dreamfoil,  // 幻梦草（净化）
    Stormvine,  // 风暴藤（闪电）
    Rotberry,   // 腐浆果（诅咒）
}

impl Seed {
    /// 获取种植效果描述（游戏内文本）
    pub fn effect_description(&self) -> String {
        match self.kind {
            SeedKind::Earthroot => "生成护盾格挡伤害".to_string(),
            SeedKind::Fadeleaf => "触发传送效果".to_string(),
            SeedKind::Firebloom => "产生火焰爆炸".to_string(),
            SeedKind::Icecap => "冻结周围水体".to_string(),
            SeedKind::Sorrowmoss => "释放毒云".to_string(),
            SeedKind::Dreamfoil => "清除负面状态".to_string(),
            SeedKind::Stormvine => "召唤闪电".to_string(),
            SeedKind::Rotberry => "释放诅咒能量".to_string(),
        }
    }

    /// 获取对应药水（炼金系统）
    pub fn to_potion(&self) -> Option<PotionKind> {
        match self.kind {
            SeedKind::Earthroot => Some(PotionKind::Purity),
            SeedKind::Fadeleaf => Some(PotionKind::Invisibility),
            SeedKind::Firebloom => Some(PotionKind::LiquidFlame),
            SeedKind::Icecap => Some(PotionKind::Frost),
            SeedKind::Sorrowmoss => Some(PotionKind::ToxicGas),
            SeedKind::Dreamfoil => Some(PotionKind::MindVision),
            SeedKind::Stormvine => Some(PotionKind::Levitation),
            SeedKind::Rotberry => None, // 不能炼金
        }
    }
}
