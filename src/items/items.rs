
// src/items.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ItemKind {
    Weapon(Weapon),
    Armor(Armor),
    Potion(Potion),
    Scroll(Scroll),
    // 其他物品类型
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Weapon {
    pub tier: usize,  // 武器等级
    pub damage: i32,
    pub enchanted: bool,  // 是否附魔
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Armor {
    pub defense: i32,
    pub tier: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Potion {
    pub kind: PotionKind,
    pub identified: bool,  // 是否已鉴定
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PotionKind {
    Healing,    // 治疗药水
    Strength,   // 力量药水
    // 其他药水类型
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Scroll {
    pub kind: ScrollKind,
    pub identified: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

