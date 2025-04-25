//src/items/src/seed.rs
use crate::potion::PotionKind;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 种子系统（8种植物种子）
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Seed {
    pub kind: SeedKind,
    pub turns_to_grow: u8, // 生长所需回合数
}

#[derive(Copy, Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
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
    /// 创建新种子
    pub fn new(kind: SeedKind) -> Self {
        let turns_to_grow = match kind {
            SeedKind::Earthroot => 10,
            SeedKind::Fadeleaf => 8,
            SeedKind::Firebloom => 12,
            SeedKind::Icecap => 10,
            SeedKind::Sorrowmoss => 15,
            SeedKind::Dreamfoil => 8,
            SeedKind::Stormvine => 12,
            SeedKind::Rotberry => 20,
        };
        Seed {
            kind,
            turns_to_grow,
        }
    }
    
    /// 随机生成新种子
    pub fn random_new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        
        let kinds = [
            SeedKind::Earthroot,
            SeedKind::Fadeleaf,
            SeedKind::Firebloom,
            SeedKind::Icecap,
            SeedKind::Sorrowmoss,
            SeedKind::Dreamfoil,
            SeedKind::Stormvine,
            SeedKind::Rotberry,
        ];
        let kind = kinds[rng.random_range(0..kinds.len())];
        
        Seed::new(kind)
    }

    /// 获取种子名称（用于UI显示）
    pub fn name(&self) -> String {
        match self.kind {
            SeedKind::Earthroot => "地根草".to_string(),
            SeedKind::Fadeleaf => "消隐叶".to_string(),
            SeedKind::Firebloom => "火焰花".to_string(),
            SeedKind::Icecap => "冰帽草".to_string(),
            SeedKind::Sorrowmoss => "哀伤苔".to_string(),
            SeedKind::Dreamfoil => "幻梦草".to_string(),
            SeedKind::Stormvine => "风暴藤".to_string(),
            SeedKind::Rotberry => "腐浆果".to_string(),
        }
    }
    
    /// 计算种子价值（考虑类型、生长时间和可炼金性）
    pub fn value(&self) -> u32 {
        // 基础价值
        let base_value = match self.kind {
            SeedKind::Dreamfoil => 50,   // 幻梦草（净化效果）最有价值
            SeedKind::Earthroot => 40,   // 地根草（护盾）
            SeedKind::Fadeleaf => 35,    // 消隐叶（传送）
            SeedKind::Icecap => 30,      // 冰帽草（冻结）
            SeedKind::Firebloom => 25,   // 火焰花（燃烧）
            SeedKind::Stormvine => 20,   // 风暴藤（闪电）
            SeedKind::Sorrowmoss => 15,  // 哀伤苔（中毒）
            SeedKind::Rotberry => 5,     // 腐浆果（诅咒）价值最低
        };

        // 生长时间修正（生长越快价值越高）
        let growth_factor = match self.turns_to_grow {
            0..=8 => 1.2,   // 快速生长
            9..=12 => 1.0,   // 中等生长
            _ => 0.8,        // 慢速生长
        };

        // 可炼金性加成（能制作药水的种子更值钱）
        let alchemy_bonus = if self.to_potion().is_some() { 1.3 } else { 1.0 };

        (base_value as f32 * growth_factor * alchemy_bonus) as u32
    }
    
    /// 获取种子颜色（用于UI显示）
    pub fn color(&self) -> &'static str {
        match self.kind {
            SeedKind::Earthroot => "green",
            SeedKind::Fadeleaf => "cyan",
            SeedKind::Firebloom => "red",
            SeedKind::Icecap => "blue",
            SeedKind::Sorrowmoss => "purple",
            SeedKind::Dreamfoil => "white",
            SeedKind::Stormvine => "yellow",
            SeedKind::Rotberry => "darkred",
        }
    }

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
            SeedKind::Rotberry => None, // 腐浆果不能用于炼金
        }
    }
}

impl Default for Seed {
    fn default() -> Self {
        Seed {
            kind: SeedKind::Earthroot,  // 默认选择地根草（基础类型）
            turns_to_grow: 10,         // 标准生长时间
        }
    }
}

impl Default for SeedKind {
    fn default() -> Self {
        SeedKind::Earthroot  // 默认地根草类型
    }
}


impl From<SeedKind> for Seed {
    fn from(kind: SeedKind) -> Self {
        Seed::new(kind)
    }
}
