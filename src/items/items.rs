use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use crate::items::weapon::weapon::*;

// src/items.rs
#[derive(Debug, Encode, Decode, Serialize, Deserialize)] // 添加Encode和Decode派生
pub enum ItemKind {
    Weapon(Weapon),
    Armor(Armor),
    Potion(Potion),
    Scroll(Scroll),
    // 其他物品类型
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub name: String,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Armor {
    pub name: String,
    pub defense: i32,
    pub tier: usize,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Potion {
    pub name: String,
    pub kind: PotionKind,
    pub identified: bool, // 是否已鉴定
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub enum PotionKind {
    Healing, // 治疗药水
    Strength, // 力量药水
             // 其他药水类型
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Scroll {
    pub name: String,
    pub kind: ScrollKind,
    pub identified: bool,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub enum ScrollKind {
    Identify,
    MagicMapping,
    // 其他卷轴类型...
}

impl Item {
    pub fn name(&self) -> &str {
        &self.name
    }
}
