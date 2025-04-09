use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 药水系统（完整12种药水）
/// 实现了破碎的像素地牢中的所有药水类型和逻辑
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Potion {
    pub kind: PotionKind,
    pub identified: bool, // 是否已鉴定
    pub alchemy: bool,    // 是否是炼金产物
}

impl Potion {
    /// 创建一个新的未鉴定的随机药水
    pub fn new_random() -> Self {
        use PotionKind::*;
        let kind = match rand::random::<u8>() % 12 {
            0 => Healing,
            1 => Experience,
            2 => ToxicGas,
            3 => ParalyticGas,
            4 => LiquidFlame,
            5 => Levitation,
            6 => Invisibility,
            7 => Purity,
            8 => Frost,
            9 => Strength,
            10 => MindVision,
            11 => Haste,
            _ => unreachable!(),
        };

        Potion {
            kind,
            identified: false,
            alchemy: false,
        }
    }

    /// 创建一个炼金产物药水
    pub fn new_alchemy(kind: PotionKind) -> Self {
        Potion {
            kind,
            identified: true, // 炼金产物总是已鉴定
            alchemy: true,
        }
    }

    /// 获取药水的默认颜色（用于未鉴定时的显示）
    pub fn color(&self) -> Color {
        self.kind.color()
    }

    /// 获取药水的名称
    pub fn name(&self) -> String {
        if self.identified {
            self.kind.name()
        } else {
            // 未鉴定时使用颜色名称
            format!("{}药水", self.color().name())
        }
    }

    /// 获取药水的效果描述
    pub fn effect(&self) -> String {
        self.kind.effect()
    }

    /// 鉴定药水
    pub fn identify(&mut self) {
        self.identified = true;
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum PotionKind {
    Healing,      // 治疗 - 恢复生命值
    Experience,   // 经验 - 提升经验值
    ToxicGas,     // 毒气 - 产生毒云
    ParalyticGas, // 麻痹气体 - 产生麻痹云
    LiquidFlame,  // 液态火焰 - 产生火焰
    Levitation,   // 漂浮 - 允许漂浮越过陷阱
    Invisibility, // 隐身 - 暂时隐身
    Purity,       // 净化 - 清除负面效果
    Frost,        // 霜冻 - 冻结液体和生物
    Strength,     // 力量 - 永久增加力量
    MindVision,   // 心灵视界 - 暂时看到所有生物
    Haste,        // 急速 - 暂时增加速度
}

impl PotionKind {
    /// 获取药水的名称
    pub fn name(&self) -> String {
        match self {
            PotionKind::Healing => "治疗药水".to_string(),
            PotionKind::Experience => "经验药水".to_string(),
            PotionKind::ToxicGas => "毒气药水".to_string(),
            PotionKind::ParalyticGas => "麻痹药水".to_string(),
            PotionKind::LiquidFlame => "液态火焰药水".to_string(),
            PotionKind::Levitation => "漂浮药水".to_string(),
            PotionKind::Invisibility => "隐身药水".to_string(),
            PotionKind::Purity => "净化药水".to_string(),
            PotionKind::Frost => "霜冻药水".to_string(),
            PotionKind::Strength => "力量药水".to_string(),
            PotionKind::MindVision => "心灵视界药水".to_string(),
            PotionKind::Haste => "急速药水".to_string(),
        }
    }

    /// 获取药水的效果描述
    pub fn effect(&self) -> String {
        match self {
            PotionKind::Healing => "恢复大量生命值".to_string(),
            PotionKind::Experience => "立即获得经验值".to_string(),
            PotionKind::ToxicGas => "产生一片有毒的云雾".to_string(),
            PotionKind::ParalyticGas => "产生一片麻痹云雾".to_string(),
            PotionKind::LiquidFlame => "产生一片火焰区域".to_string(),
            PotionKind::Levitation => "允许你漂浮越过陷阱和障碍".to_string(),
            PotionKind::Invisibility => "使你暂时隐形".to_string(),
            PotionKind::Purity => "清除所有负面效果".to_string(),
            PotionKind::Frost => "冻结水和敌人".to_string(),
            PotionKind::Strength => "永久增加你的力量属性".to_string(),
            PotionKind::MindVision => "暂时看到所有生物的位置".to_string(),
            PotionKind::Haste => "暂时大幅提高移动速度".to_string(),
        }
    }

    /// 获取药水的默认颜色
    pub fn color(&self) -> Color {
        match self {
            PotionKind::Healing => Color::Crimson,
            PotionKind::Experience => Color::Azure,
            PotionKind::ToxicGas => Color::Jade,
            PotionKind::ParalyticGas => Color::Amber,
            PotionKind::LiquidFlame => Color::Ruby,
            PotionKind::Levitation => Color::Indigo,
            PotionKind::Invisibility => Color::Silver,
            PotionKind::Purity => Color::Ivory,
            PotionKind::Frost => Color::Sapphire,
            PotionKind::Strength => Color::Burgundy,
            PotionKind::MindVision => Color::Violet,
            PotionKind::Haste => Color::Emerald,
        }
    }
}

/// 药水颜色系统（与破碎的像素地牢一致）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum Color {
    Crimson,  // 深红
    Azure,    // 天蓝
    Jade,     // 翡翠
    Amber,    // 琥珀
    Ruby,     // 红宝石
    Indigo,   // 靛蓝
    Silver,   // 银色
    Ivory,    // 象牙白
    Sapphire, // 蓝宝石
    Burgundy, // 酒红
    Violet,   // 紫罗兰
    Emerald,  // 祖母绿
}

impl Color {
    /// 获取颜色的名称
    pub fn name(&self) -> &'static str {
        match self {
            Color::Crimson => "深红",
            Color::Azure => "天蓝",
            Color::Jade => "翡翠",
            Color::Amber => "琥珀",
            Color::Ruby => "红宝石",
            Color::Indigo => "靛蓝",
            Color::Silver => "银色",
            Color::Ivory => "象牙",
            Color::Sapphire => "蓝宝石",
            Color::Burgundy => "酒红",
            Color::Violet => "紫罗兰",
            Color::Emerald => "祖母绿",
        }
    }
}

impl fmt::Display for Potion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Display for PotionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_potion_creation() {
        let mut potion = Potion::new_random();
        assert!(!potion.identified);
        assert!(!potion.alchemy);

        potion.identify();
        assert!(potion.identified);
    }

    #[test]
    fn test_alchemy_potion() {
        let potion = Potion::new_alchemy(PotionKind::Strength);
        assert!(potion.identified);
        assert!(potion.alchemy);
        assert_eq!(potion.name(), "力量药水");
    }

    #[test]
    fn test_potion_kind_properties() {
        let kind = PotionKind::Healing;
        assert_eq!(kind.name(), "治疗药水");
        assert_eq!(kind.effect(), "恢复大量生命值");
        assert_eq!(kind.color(), Color::Crimson);
    }
}
