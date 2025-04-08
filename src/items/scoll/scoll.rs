use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 卷轴系统（完整10种）
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Scroll {
    pub name: String,
    pub kind: ScrollKind,
    pub identified: bool,
    pub exotic: bool, // 是否是异变卷轴
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum ScrollKind {
    Upgrade,       // 强化
    RemoveCurse,   // 祛咒
    Identify,      // 鉴定
    MagicMapping,  // 地图
    MirrorImage,   // 镜像
    Teleportation, // 传送
    Lullaby,       // 催眠
    Rage,          // 狂暴
    Recharging,    // 充能
    Transmutation, // 变形
}
