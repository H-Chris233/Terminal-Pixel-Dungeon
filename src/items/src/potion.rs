//src/items/src/potion.rs
use bincode::serde::encode_to_vec;
use bincode::{Decode, Encode};
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hasher;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tui::style::Color;

use crate::ItemCategory;
use crate::ItemTrait;
use crate::BINCODE_CONFIG;
use crate::Item;
use crate::ItemKind;

/// 药水系统（完整12种药水）
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Potion {
    pub kind: PotionKind,
    pub identified: bool,   // 是否已鉴定
    pub color: PotionColor, // 药水颜色（即使已鉴定也保留）
}

impl Potion {
    /// 创建一个新的未鉴定的随机药水
    pub fn random_new() -> Self {
        let mut rng = rand::rng();

        // 随机分配药水类型和颜色（确保不重复）
        let kind = *PotionKind::iter()
            .collect::<Vec<_>>()
            .choose(&mut rng)
            .unwrap();
        let color = PotionColor::assign_random_color(&kind);

        Potion {
            kind,
            identified: false,
            color,
        }
    }

    /// 创建一个炼金产物药水
    pub fn new_alchemy(kind: PotionKind) -> Self {
        // 炼金产物使用标准颜色
        let color = kind.standard_color();

        Potion {
            kind,
            identified: true, // 炼金产物总是已鉴定
            color,
        }
    }

    /// 计算药水价值（考虑类型、鉴定状态和炼金属性）
    pub fn value(&self) -> u32 {
        // 基础价值
        let base_value = match self.kind {
            PotionKind::Strength => 500,   // 力量药水最有价值
            PotionKind::Experience => 400, // 经验药水次之
            PotionKind::Healing => 300,
            PotionKind::Invisibility => 250,
            PotionKind::Haste => 250,
            PotionKind::MindVision => 200,
            PotionKind::Levitation => 150,
            PotionKind::Frost => 150,
            PotionKind::Purity => 120,
            PotionKind::LiquidFlame => 100,
            PotionKind::ToxicGas => 80,
            PotionKind::ParalyticGas => 80,
        };

        // 状态修正

        if !self.identified {
            (base_value as f32 * 0.5) as u32 // 未鉴定药水价值减半
        } else {
            base_value
        }
    }

    /// 获取药水的显示颜色（使用tui::style::Color）
    pub fn display_color(&self) -> Color {
        self.color.to_tui_color()
    }

    /// 获取药水的名称
    pub fn name(&self) -> String {
        if self.identified {
            self.kind.name()
        } else {
            // 未鉴定时使用颜色名称
            format!("{}药水", self.color.name())
        }
    }

    /// 获取药水的效果描述
    pub fn effect(&self) -> String {
        self.kind.effect()
    }

    /// 鉴定药水
    pub fn identify(&mut self) {
        if !self.identified {
            self.identified = true;
            // 鉴定后显示标准颜色
            self.color = self.kind.standard_color();
        }
    }
}

/// 药水类型（完整12种）
#[derive(
    Debug,
    Clone,
    Copy,
    EnumIter,
    Eq,
    Hash,
    PartialEq,
    Encode,
    Decode,
    Serialize,
    Deserialize,
    Default,
)]
pub enum PotionKind {
    #[default]
    Healing, // 治疗
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

impl PotionKind {
    /// 获取药水的标准颜色（用于炼金产物和默认情况）
    pub fn standard_color(&self) -> PotionColor {
        match self {
            PotionKind::Healing => PotionColor::Red,
            PotionKind::Experience => PotionColor::Blue,
            PotionKind::ToxicGas => PotionColor::Green,
            PotionKind::ParalyticGas => PotionColor::Yellow,
            PotionKind::LiquidFlame => PotionColor::Orange,
            PotionKind::Levitation => PotionColor::Purple,
            PotionKind::Invisibility => PotionColor::Silver,
            PotionKind::Purity => PotionColor::White,
            PotionKind::Frost => PotionColor::LightBlue,
            PotionKind::Strength => PotionColor::Pink,
            PotionKind::MindVision => PotionColor::Violet,
            PotionKind::Haste => PotionColor::Turquoise,
        }
    }

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
}

/// 药水颜色系统（12种独特颜色）
#[derive(
    Debug, Clone, Copy, EnumIter, Eq, Hash, PartialEq, Encode, Decode, Serialize, Deserialize,
)]
pub enum PotionColor {
    Red,
    Blue,
    Green,
    Yellow,
    Orange,
    Purple,
    Silver,
    White,
    LightBlue,
    Pink,
    Violet,
    Turquoise,
}

impl PotionColor {
    /// 为药水类型分配随机颜色（确保不重复）
    pub fn assign_random_color(kind: &PotionKind) -> Self {
        // 使用药水类型的哈希值作为随机种子，确保相同类型在不同游戏中也不同
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        kind.hash(&mut hasher);
        let hash = hasher.finish();

        let colors = PotionColor::iter().collect::<Vec<_>>();
        let idx = (hash as usize) % colors.len();
        colors[idx]
    }

    /// 转换为tui颜色
    pub fn to_tui_color(&self) -> Color {
        match self {
            PotionColor::Red => Color::Red,
            PotionColor::Blue => Color::Blue,
            PotionColor::Green => Color::Green,
            PotionColor::Yellow => Color::Yellow,
            PotionColor::Orange => Color::Rgb(255, 165, 0),
            PotionColor::Purple => Color::Magenta,
            PotionColor::Silver => Color::Gray,
            PotionColor::White => Color::White,
            PotionColor::LightBlue => Color::LightBlue,
            PotionColor::Pink => Color::LightRed,
            PotionColor::Violet => Color::LightMagenta,
            PotionColor::Turquoise => Color::LightCyan,
        }
    }

    /// 获取颜色名称
    pub fn name(&self) -> &'static str {
        match self {
            PotionColor::Red => "红色",
            PotionColor::Blue => "蓝色",
            PotionColor::Green => "绿色",
            PotionColor::Yellow => "黄色",
            PotionColor::Orange => "橙色",
            PotionColor::Purple => "紫色",
            PotionColor::Silver => "银色",
            PotionColor::White => "白色",
            PotionColor::LightBlue => "浅蓝",
            PotionColor::Pink => "粉色",
            PotionColor::Violet => "紫罗兰",
            PotionColor::Turquoise => "青绿色",
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

impl Default for Potion {
    fn default() -> Self {
        Potion {
            kind: PotionKind::Healing, // 默认类型：治疗药水（最基础类型）
            identified: false,         // 默认未鉴定
            color: PotionColor::Red,   // 治疗药水的标准颜色
        }
    }
}

impl From<PotionKind> for Potion {
    fn from(kind: PotionKind) -> Self {
        let color = kind.standard_color();
        Potion {
            kind,
            identified: false, // Default to unidentified
            color,
        }
    }
}

impl ItemTrait for Potion {
    /// 生成堆叠标识（保持颜色/类型区分）
    fn stacking_id(&self) -> u64 {
        let mut hasher = SeaHasher::new();

        if !self.identified {
            let key = (&self.color, false);
            let bytes = encode_to_vec(key, BINCODE_CONFIG).unwrap();
            hasher.write(&bytes);
            hasher.finish()
        } else {
            let key = (&self.kind, true);
            let bytes = encode_to_vec(key, BINCODE_CONFIG).unwrap();
            hasher.write(&bytes);
            hasher.finish()
        }
    }

    /// 保持可堆叠属性
    fn is_stackable(&self) -> bool {
        true
    }

    /// 设置为u32最大值模拟无限堆叠
    fn max_stack(&self) -> u32 {
        u32::MAX // 4,294,967,295
    }

    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Potion
    }
    fn sort_value(&self) -> u32 {
        match self.kind {
            PotionKind::Healing => 12,
            PotionKind::Experience => 11,
            PotionKind::ToxicGas => 10,
            PotionKind::ParalyticGas => 9,
            PotionKind::LiquidFlame => 8,
            PotionKind::Levitation => 7,
            PotionKind::Invisibility => 6,
            PotionKind::Purity => 5,
            PotionKind::Frost => 4,
            PotionKind::Strength => 3,
            PotionKind::MindVision => 2,
            PotionKind::Haste => 1,
            _ => 0,
        }
    }
}

impl From<Potion> for Item {
    fn from(potion: Potion) -> Self {
        Item {
            name: potion.name(),
            kind: ItemKind::Potion(potion),
            description: "...".to_string(),
            quantity: 1,
            x: -1,
            y: -1,
        }
    }
}
