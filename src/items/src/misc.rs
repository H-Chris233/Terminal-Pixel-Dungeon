// src/items/misc/misc.rs

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 杂项物品类型，参考破碎的像素地牢游戏逻辑
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub enum MiscKind {
    /// 金币 - 游戏中的通用货币
    Gold(usize),
    /// 钥匙 - 用于打开上锁的门和箱子
    Key,
    /// 炸弹 - 可以炸开墙壁或造成范围伤害
    Bomb,
    /// 蜂巢罐 - 破碎后释放蜜蜂
    Honeypot,
    /// 火把
    Torch,
    /// 其他未分类物品
    Other(String),
}

/// 杂项物品结构体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct MiscItem {
    /// 物品类型
    pub kind: MiscKind,
    /// 物品数量(对于可堆叠物品)
    pub quantity: u32,
    /// 物品价格(基础值)
    price: usize,
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
            MiscKind::Other(_) => 10,
        };

        MiscItem {
            kind,
            quantity: 1,
            price,
        }
    }

    pub fn value(&self) -> usize {
        self.price
    }

    /// 设置物品数量(用于可堆叠物品)
    pub fn with_quantity(mut self, quantity: u32) -> Self {
        self.quantity = quantity;
        self
    }

    /// 判断物品是否可投掷
    pub fn is_throwable(&self) -> bool {
        match self.kind {
            MiscKind::Bomb | MiscKind::Honeypot => true,
            _ => false,
        }
    }

    /// 获取物品基础名称
    pub fn base_name(&self) -> &str {
        match self.kind {
            MiscKind::Gold(_) => "金币",
            MiscKind::Key => "钥匙",
            MiscKind::Bomb => "炸弹",
            MiscKind::Torch => "火把",
            MiscKind::Honeypot => "蜂巢罐",
            MiscKind::Other(ref name) => name.as_str(),
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
