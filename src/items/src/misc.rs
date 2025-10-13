//src/items/src/misc.rs
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

/// 杂项物品类型，参考破碎的像素地牢游戏逻辑
#[derive(Copy, Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub enum MiscKind {
    /// 金币 - 游戏中的通用货币
    Gold(u32),
    /// 钥匙 - 用于打开上锁的门和箱子
    Key,
    /// 炸弹 - 可以炸开墙壁或造成范围伤害
    Bomb,
    /// 蜂巢罐 - 破碎后释放蜜蜂
    Honeypot,
    /// 火把
    Torch,
    /// 其他未分类物品
    Other,
}

/// 杂项物品结构体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct MiscItem {
    /// 物品类型
    pub kind: MiscKind,
    /// 物品数量(对于可堆叠物品)
    pub quantity: u32,
    /// 物品价格(基础值)
    price: u32,
}

impl MiscItem {
    /// 创建一个新的杂项物品
    pub fn new(kind: MiscKind) -> Self {
        let price = match kind {
            MiscKind::Gold(g) => g,
            MiscKind::Key => 30,
            MiscKind::Bomb => 100,
            MiscKind::Honeypot => 60,
            MiscKind::Torch => 50,
            MiscKind::Other => 10,
        };

        MiscItem {
            kind,
            quantity: 1,
            price,
        }
    }

    /// 随机生成新杂项物品
    pub fn random_new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();

        let kinds = [
            MiscKind::Key,
            MiscKind::Bomb,
            MiscKind::Honeypot,
            MiscKind::Torch,
            MiscKind::Other,
        ];
        let kind = kinds[rng.random_range(0..kinds.len())];

        let mut item = MiscItem::new(kind);

        // 如果是金币，随机生成数量
        if let MiscKind::Gold(_) = item.kind {
            let amount = rng.random_range(1..=100);
            item.kind = MiscKind::Gold(amount);
        }

        // 可堆叠物品随机数量
        if matches!(
            item.kind,
            MiscKind::Bomb | MiscKind::Honeypot | MiscKind::Torch
        ) {
            item.quantity = rng.random_range(1..=3);
        }

        item
    }

    pub fn value(&self) -> u32 {
        self.price
    }

    /// 设置物品数量(用于可堆叠物品)
    pub fn with_quantity(mut self, quantity: u32) -> Self {
        self.quantity = quantity;
        self
    }

    /// 判断物品是否可投掷
    pub fn is_throwable(&self) -> bool {
        matches!(self.kind, MiscKind::Bomb | MiscKind::Honeypot)
    }

    /// 获取物品基础名称
    pub fn base_name(&self) -> &str {
        match self.kind {
            MiscKind::Gold(_) => "金币",
            MiscKind::Key => "钥匙",
            MiscKind::Bomb => "炸弹",
            MiscKind::Torch => "火把",
            MiscKind::Honeypot => "蜂巢罐",
            MiscKind::Other => "神秘碎片",
        }
    }

    /// 获取完整物品名称(包含状态信息)
    pub fn name(&self) -> String {
        let mut name = self.base_name().to_string();
        if self.quantity > 1 {
            name = format!("{}x {}", self.quantity, name);
        }

        name
    }
}

impl Default for MiscItem {
    fn default() -> Self {
        MiscItem {
            kind: MiscKind::Gold(1), // 默认类型：1金币（最小单位）
            quantity: 1,             // 单个物品
            price: 1,                // 1金币价值为1
        }
    }
}

impl From<MiscKind> for MiscItem {
    fn from(kind: MiscKind) -> Self {
        MiscItem::new(kind)
    }
}

impl ItemTrait for MiscItem {
    /// 生成堆叠标识（根据不同类型决定属性）
    fn stacking_id(&self) -> u64 {
        let mut hasher = SeaHasher::new();

        // 序列化关键属性（金币需要包含金额）
        let key = match self.kind {
            MiscKind::Gold(g) => (self.kind, g), // 不同面额金币分开堆叠
            _ => (self.kind, 0),                 // 其他物品只考虑类型
        };

        let bytes = encode_to_vec(key, BINCODE_CONFIG).unwrap();
        hasher.write(&bytes);
        hasher.finish()
    }

    /// 设置可堆叠性（根据最新需求）
    fn is_stackable(&self) -> bool {
        matches!(
            self.kind,
            MiscKind::Gold(_)
                | MiscKind::Bomb
                | MiscKind::Honeypot
                | MiscKind::Key
                | MiscKind::Torch
                | MiscKind::Other
        )
    }

    /// 设置最大堆叠数量
    fn max_stack(&self) -> u32 {
        match self.kind {
            MiscKind::Gold(_) => u32::MAX, // 金币无限堆叠
            MiscKind::Bomb => 10,          // 炸弹最多10个
            MiscKind::Honeypot => 5,       // 蜂巢罐5个
            MiscKind::Key => 20,           // 钥匙20把
            MiscKind::Torch => 15,         // 火把15支
            MiscKind::Other => 30,         // 神秘碎片30个
        }
    }

    fn display_name(&self) -> String {
        match self.kind {
            MiscKind::Gold(_) => "金币".to_string(),
            MiscKind::Key => "钥匙".to_string(),
            MiscKind::Torch => "火把".to_string(),
            // ... other misc items
            _ => "杂项".to_string(),
        }
    }

    fn category(&self) -> ItemCategory {
        ItemCategory::Misc
    }

    fn sort_value(&self) -> u32 {
        match self.kind {
            MiscKind::Gold(_) => 100,
            MiscKind::Key => 90,
            MiscKind::Torch => 80,
            // ... other misc items
            _ => 0,
        }
    }
}

impl From<MiscItem> for Item {
    fn from(misc: MiscItem) -> Self {
        Item {
            name: misc.name(),
            kind: ItemKind::Misc(misc),
            description: "...".to_string(),
            quantity: 1,
            x: -1,
            y: -1,
        }
    }
}
