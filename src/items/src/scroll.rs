//src/items/src/scroll.rs
//! 卷轴系统模块
//!
//! 实现了破碎的像素地牢(SPD)中的10种卷轴逻辑
//! 注意：所有渲染由其他模块处理，这里只处理数据逻辑

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// 卷轴系统（完整10种）
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Scroll {
    /// 卷轴种类
    pub kind: ScrollKind,
    /// 是否已鉴定
    pub identified: bool,
    /// 是否是异变卷轴（SPD中的"异变卷轴"变种）
    pub exotic: bool,
}

impl Scroll {
    /// 创建一个新的未鉴定卷轴
    pub fn new(kind: ScrollKind) -> Self {
        Self {
            kind,
            identified: false,
            exotic: false,
        }
    }

    /// 创建一个新的异变卷轴
    pub fn new_exotic(kind: ScrollKind) -> Self {
        Self {
            kind,
            identified: false,
            exotic: true,
        }
    }
    
    /// 随机生成新卷轴（10%概率为异变卷轴）
    pub fn random_new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        
        let kinds = ScrollKind::iter().collect::<Vec<_>>();
        let kind = kinds[rng.random_range(0..kinds.len())].clone();
        
        if rng.random_bool(0.1) {
            Scroll::new_exotic(kind)
        } else {
            Scroll::new(kind)
        }
    }
    
    /// 计算卷轴价值（考虑类型、鉴定状态和异变状态）
    pub fn value(&self) -> u32 {
        // 基础价值
        let base_value = match self.kind {
            ScrollKind::Upgrade => 400,       // 强化装备最有价值
            ScrollKind::RemoveCurse => 350,   // 解除诅咒次之
            ScrollKind::Identify => 300,
            ScrollKind::Transmutation => 300, // 改变物品很有价值
            ScrollKind::Recharging => 250,    // 充能魔杖
            ScrollKind::MagicMapping => 200,   // 地图探索
            ScrollKind::MirrorImage => 200,    // 分身辅助
            ScrollKind::Teleportation => 180,  // 传送逃生
            ScrollKind::Lullaby => 150,        // 控制敌人
            ScrollKind::Rage => 120,           // 狂暴战术价值较低
        };

        // 状态修正
        let mut value = if !self.identified {
            (base_value as f32 * 0.6) as u32  // 未鉴定卷轴价值降低40%
        } else {
            base_value
        };

        // 异变卷轴加成（价值提升50%）
        if self.exotic {
            value = (value as f32 * 1.5) as u32;
        }

        value
    }

    /// 获取卷轴名称（根据鉴定状态返回相应名称）
    pub fn name(&self) -> String {
        if !self.identified {
            return if self.exotic {
                "未鉴定的异变卷轴".to_string()
            } else {
                "未鉴定的卷轴".to_string()
            };
        }

        match (self.kind, self.exotic) {
            // 普通卷轴
            (ScrollKind::Upgrade, false) => "强化卷轴".to_string(),
            (ScrollKind::RemoveCurse, false) => "祛咒卷轴".to_string(),
            (ScrollKind::Identify, false) => "鉴定卷轴".to_string(),
            (ScrollKind::MagicMapping, false) => "地图卷轴".to_string(),
            (ScrollKind::MirrorImage, false) => "镜像卷轴".to_string(),
            (ScrollKind::Teleportation, false) => "传送卷轴".to_string(),
            (ScrollKind::Lullaby, false) => "催眠卷轴".to_string(),
            (ScrollKind::Rage, false) => "狂暴卷轴".to_string(),
            (ScrollKind::Recharging, false) => "充能卷轴".to_string(),
            (ScrollKind::Transmutation, false) => "变形卷轴".to_string(),

            // 异变卷轴（按照新要求修改的名称）
            (ScrollKind::Upgrade, true) => "附魔卷轴".to_string(),
            (ScrollKind::RemoveCurse, true) => "圣洁卷轴".to_string(),
            (ScrollKind::Identify, true) => "预见卷轴".to_string(),
            (ScrollKind::MagicMapping, true) => "探知卷轴".to_string(),
            (ScrollKind::MirrorImage, true) => "复生卷轴".to_string(),
            (ScrollKind::Teleportation, true) => "回归卷轴".to_string(),
            (ScrollKind::Lullaby, true) => "魅惑卷轴".to_string(),
            (ScrollKind::Rage, true) => "决斗卷轴".to_string(),
            (ScrollKind::Recharging, true) => "魔能卷轴".to_string(),
            (ScrollKind::Transmutation, true) => "蜕变卷轴".to_string(),
        }
    }

    /// 鉴定卷轴
    pub fn identify(&mut self) {
        self.identified = true;
    }
}

/// 卷轴种类（对应SPD中的10种卷轴）
#[derive(Eq, Hash, PartialEq, Debug, Copy, Clone, Encode, Decode, Serialize, Deserialize, EnumIter)]
pub enum ScrollKind {
    Upgrade,       // 强化卷轴 - 强化装备
    RemoveCurse,   // 祛咒卷轴 - 解除装备诅咒
    Identify,      // 鉴定卷轴 - 鉴定物品
    MagicMapping,  // 地图卷轴 - 显示当前楼层地图
    MirrorImage,   // 镜像卷轴 - 创建分身
    Teleportation, // 传送卷轴 - 随机传送
    Lullaby,       // 催眠卷轴 - 使敌人沉睡
    Rage,          // 狂暴卷轴 - 激怒敌人
    Recharging,    // 充能卷轴 - 充能魔杖
    Transmutation, // 变形卷轴 - 改变物品
}

impl Default for Scroll {
    fn default() -> Self {
        Scroll {
            kind: ScrollKind::Identify,  // 默认选择鉴定卷轴（基础类型）
            identified: false,          // 默认未鉴定
            exotic: false,              // 默认非异变卷轴
        }
    }
}

impl Default for ScrollKind {
    fn default() -> Self {
        ScrollKind::Identify  // 默认鉴定卷轴类型
    }
}
