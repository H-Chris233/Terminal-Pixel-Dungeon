// src/items/misc/misc.rs

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 杂项物品类型，参考破碎的像素地牢游戏逻辑
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub enum MiscType {
    /// 金币 - 游戏中的通用货币
    Gold,
    /// 钥匙 - 用于打开上锁的门和箱子
    Key,
    /// 炸弹 - 可以炸开墙壁或造成范围伤害
    Bomb,
    /// 神秘肉 - 可以烹饪或直接食用(有风险)
    MysteryMeat,
    /// 蜂巢罐 - 破碎后释放蜜蜂
    Honeypot,
    /// 冰冻肉 - 安全可食用
    FrozenCarpaccio,
    /// 其他未分类物品
    Other(String),
}

/// 杂项物品结构体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct MiscItem {
    /// 物品类型
    pub item_type: MiscType,
    /// 物品数量(对于可堆叠物品)
    pub quantity: u32,
    /// 物品价格(基础值)
    pub price: u32,
}

impl MiscItem {
    /// 创建一个新的杂项物品
    pub fn new(item_type: MiscType) -> Self {
        let price = match item_type {
            MiscType::Gold => 1,
            MiscType::Key => 30,
            MiscType::Bomb => 100,
            MiscType::MysteryMeat => 20,
            MiscType::Honeypot => 60,
            MiscType::FrozenCarpaccio => 30,
            MiscType::Other(_) => 10,
        };

        MiscItem {
            item_type,
            quantity: 1,
            price,
        }
    }

    /// 设置物品数量(用于可堆叠物品)
    pub fn with_quantity(mut self, quantity: u32) -> Self {
        self.quantity = quantity;
        self
    }

    /// 判断物品是否可投掷
    pub fn is_throwable(&self) -> bool {
        match self.item_type {
            MiscType::Bomb | MiscType::Honeypot => true,
            _ => false,
        }
    }

    /// 判断物品是否可食用
    pub fn is_edible(&self) -> bool {
        match self.item_type {
            MiscType::MysteryMeat | MiscType::FrozenCarpaccio => true,
            _ => false,
        }
    }

    /// 获取物品基础名称
    pub fn base_name(&self) -> &str {
        match self.item_type {
            MiscType::Gold => "金币",
            MiscType::Key => "钥匙",
            MiscType::Bomb => "炸弹",
            MiscType::MysteryMeat => "神秘肉",
            MiscType::Honeypot => "蜂巢罐",
            MiscType::FrozenCarpaccio => "冰冻肉",
            MiscType::Other(ref name) => name.as_str(),
        }
    }

    /// 获取完整物品名称(包含状态信息)
    pub fn full_name(&self) -> String {
        let mut name = self.base_name().to_string();
        if self.quantity > 1 {
            name = format!("{}x {}", self.quantity, name);
        }

        name
    }
}
