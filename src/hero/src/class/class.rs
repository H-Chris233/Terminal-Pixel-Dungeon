//src/hero/class/class.rs

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub mod huntress;
pub mod mage;
pub mod rogue;
pub mod warrior;

use crate::hero::hero::*;

/// 英雄职业枚举
#[derive(Default, Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub enum Class {
    #[default]
    Warrior, // 战士（高生命值，中等攻击）

    Mage,     // 法师（低生命值，高攻击，特殊能力）
    Rogue,    // 盗贼（中等生命值，高暴击率）
    Huntress, // 女猎手（远程攻击，中等属性）
}
