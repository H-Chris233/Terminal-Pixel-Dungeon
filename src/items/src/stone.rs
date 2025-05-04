//src/items/src/stone.rs
//! 魔法石系统模块
//!
//! 实现了与卷轴系统对应的10种魔法石
//! 注意：所有石头均为一次性使用

use bincode::serde::encode_to_vec;
use bincode::{Decode, Encode};
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::hash::Hasher;

use crate::ItemCategory;
use crate::ItemTrait;
use crate::BINCODE_CONFIG;
use crate::Item;
use crate::ItemKind;

/// 魔法石系统（10种对应卷轴）
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Stone {
    /// 魔法石种类
    pub kind: StoneKind,
    /// 是否已使用（所有石头都只能使用一次）
    pub used: bool,
}

impl Stone {
    /// 创建新的魔法石（初始未使用状态）
    pub fn new(kind: StoneKind) -> Self {
        Self { kind, used: false }
    }

    /// 计算魔法石价值（考虑类型和使用状态）
    pub fn value(&self) -> u32 {
        // 基础价值（约为对应卷轴价值的60%）
        let base_value = match self.kind {
            StoneKind::Upgrade => 240,     // 强化卷轴400*0.6
            StoneKind::RemoveCurse => 210, // 祛咒卷轴350*0.6
            StoneKind::Identify => 180,
            StoneKind::Transmutation => 180, // 变形卷轴300*0.6
            StoneKind::Recharging => 150,    // 充能卷轴250*0.6
            StoneKind::MagicMapping => 120,  // 地图卷轴200*0.6
            StoneKind::MirrorImage => 120,   // 镜像卷轴200*0.6
            StoneKind::Teleportation => 110, // 传送卷轴180*0.6
            StoneKind::Lullaby => 90,        // 催眠卷轴150*0.6
            StoneKind::Rage => 70,           // 狂暴卷轴120*0.6
        };

        // 使用状态修正（使用后价值降为10%）
        if self.used {
            (base_value as f32 * 0.1) as u32
        } else {
            base_value
        }
    }

    /// 随机生成新魔法石
    pub fn random_new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();

        let kinds = [
            StoneKind::Upgrade,
            StoneKind::RemoveCurse,
            StoneKind::Identify,
            StoneKind::MagicMapping,
            StoneKind::MirrorImage,
            StoneKind::Teleportation,
            StoneKind::Lullaby,
            StoneKind::Rage,
            StoneKind::Recharging,
            StoneKind::Transmutation,
        ];
        let kind = kinds[rng.random_range(0..kinds.len())];

        Stone::new(kind)
    }

    /// 获取魔法石名称
    pub fn name(&self) -> String {
        match self.kind {
            StoneKind::Upgrade => "强化之石".to_string(),
            StoneKind::RemoveCurse => "祛咒之石".to_string(),
            StoneKind::Identify => "鉴定之石".to_string(),
            StoneKind::MagicMapping => "地图之石".to_string(),
            StoneKind::MirrorImage => "镜像之石".to_string(),
            StoneKind::Teleportation => "传送之石".to_string(),
            StoneKind::Lullaby => "催眠之石".to_string(),
            StoneKind::Rage => "狂暴之石".to_string(),
            StoneKind::Recharging => "充能之石".to_string(),
            StoneKind::Transmutation => "变形之石".to_string(),
        }
    }

    /// 使用魔法石效果（返回效果描述）
    pub fn use_effect(&mut self) -> Option<String> {
        if self.used {
            return None;
        }
        self.used = true;
        Some(match self.kind {
            StoneKind::Upgrade => "临时强化一件装备（持续30回合）".to_string(),
            StoneKind::RemoveCurse => "解除装备诅咒（范围效果）".to_string(),
            StoneKind::Identify => "鉴定背包内所有物品".to_string(),
            StoneKind::MagicMapping => "显示当前楼层完整地图".to_string(),
            StoneKind::MirrorImage => "创造2个分身协助战斗".to_string(),
            StoneKind::Teleportation => "随机传送到本层某处".to_string(),
            StoneKind::Lullaby => "使周围敌人陷入沉睡".to_string(),
            StoneKind::Rage => "激怒所有敌人互相攻击".to_string(),
            StoneKind::Recharging => "立即恢复所有魔杖能量".to_string(),
            StoneKind::Transmutation => "随机改变一个物品的类型".to_string(),
        })
    }

    /// 是否已耗尽
    pub fn is_depleted(&self) -> bool {
        self.used
    }
}

/// 魔法石种类枚举（10种对应卷轴）
#[derive(
    Copy, Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize, Default,
)]
pub enum StoneKind {
    Upgrade,     // 对应强化卷轴
    RemoveCurse, // 对应祛咒卷轴
    #[default]
    Identify, // 对应鉴定卷轴
    MagicMapping, // 对应地图卷轴
    MirrorImage, // 对应镜像卷轴
    Teleportation, // 对应传送卷轴
    Lullaby,     // 对应催眠卷轴
    Rage,        // 对应狂暴卷轴
    Recharging,  // 对应充能卷轴
    Transmutation, // 对应变形卷轴
}

impl Default for Stone {
    fn default() -> Self {
        Stone {
            kind: StoneKind::Identify, // 默认选择鉴定之石（基础类型）
            used: false,               // 默认未使用
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stone_creation() {
        let stone = Stone::new(StoneKind::Upgrade);
        assert_eq!(stone.name(), "强化之石");
        assert!(!stone.used);
    }

    #[test]
    fn test_use_effect() {
        let mut stone = Stone::new(StoneKind::MagicMapping);
        assert!(stone.use_effect().is_some());
        assert!(stone.used);
        assert!(stone.use_effect().is_none()); // 第二次使用失败
    }

    #[test]
    fn test_stone_values() {
        // 测试基础价值
        let upgrade = Stone::new(StoneKind::Upgrade);
        assert_eq!(upgrade.value(), 240);

        let rage = Stone::new(StoneKind::Rage);
        assert_eq!(rage.value(), 70);

        // 测试使用后价值
        let mut identify = Stone::new(StoneKind::Identify);
        identify.use_effect();
        assert_eq!(identify.value(), 18); // 180*0.1

        // 测试特殊类型价值
        let mapping = Stone::new(StoneKind::MagicMapping);
        assert_eq!(mapping.value(), 120);

        let mut trans = Stone::new(StoneKind::Transmutation);
        trans.use_effect();
        assert_eq!(trans.value(), 18); // 180*0.1
    }
}

impl From<StoneKind> for Stone {
    fn from(kind: StoneKind) -> Self {
        Stone::new(kind)
    }
}

impl ItemTrait for Stone {
    /// 生成堆叠标识（包含种类和使用状态）
    fn stacking_id(&self) -> u64 {
        let mut hasher = SeaHasher::new();
        let bytes = encode_to_vec((self.kind, self.used), BINCODE_CONFIG).unwrap();
        hasher.write(&bytes);
        hasher.finish()
    }

    /// 魔法石可以堆叠（相同状态）
    fn is_stackable(&self) -> bool {
        true
    }

    /// 设置合理的堆叠上限
    fn max_stack(&self) -> u32 {
        // 未使用的魔法石堆叠上限更高
        if !self.used {
            20 // 未使用状态可堆叠10个
        } else {
            50 // 已使用状态
        }
    }
    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Stone
    }
    fn sort_value(&self) -> u32 {
        30
    }
}

impl From<Stone> for Item {
    fn from(stone: Stone) -> Self {
        Item {
            name: stone.name(),
            kind: ItemKind::Stone(stone),
            description: "...".to_string(),
            quantity: 1,
            x: -1,
            y: -1,
        }
    }
}
