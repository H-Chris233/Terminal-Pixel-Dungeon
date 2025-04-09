// src/items/food/food.rs

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 食物系统（精确还原游戏机制）
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Food {
    pub kind: FoodKind,
    pub energy: u32,  // 饱食度恢复量
    pub quantity: u8, // 数量（如肉馅饼可能有多个）
    pub cooked: bool, // 是否已烹饪（仅对神秘肉有效）
}

/// 食物类型（3种基础类型+特殊类型）
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum FoodKind {
    Ration,          // 干粮 - 标准食物
    Pasty,           // 肉馅饼
    MysteryMeat,     // 神秘肉 - 可烹饪或生吃
    FrozenCarpaccio, // 冰冻肉片（特殊）
}

impl Food {
    /// 创建新食物
    pub fn new(kind: FoodKind) -> Self {
        let (energy, quantity) = match kind {
            FoodKind::Ration => (350, 1),
            FoodKind::Pasty => (450, 1),
            FoodKind::MysteryMeat => (100, 1),
            FoodKind::FrozenCarpaccio => (200, 1),
        };

        Food {
            kind,
            energy,
            quantity,
            cooked: false,
        }
    }

    /// 食用食物（返回实际恢复的饱食度）
    pub fn eat(&mut self) -> u32 {
        if self.quantity == 0 {
            return 0;
        }

        self.quantity -= 1;
        let mut actual_energy = self.energy;

        // 特殊效果处理
        match self.kind {
            FoodKind::MysteryMeat if !self.cooked => {
                // 生吃神秘肉有50%效果
                actual_energy /= 2;
            }
            FoodKind::FrozenCarpaccio => {
                // 冰冻肉片有冷却效果
                actual_energy += 50;
            }
            _ => {}
        }

        actual_energy
    }

    /// 烹饪神秘肉
    pub fn cook(&mut self) -> bool {
        if let FoodKind::MysteryMeat = self.kind {
            self.cooked = true;
            self.energy = 150; // 烹饪后增加效果
            true
        } else {
            false
        }
    }

    /// 冷冻食物（制作冰冻肉片）
    pub fn freeze(&mut self) -> bool {
        if let FoodKind::MysteryMeat = self.kind {
            self.kind = FoodKind::FrozenCarpaccio;
            self.energy = 200;
            true
        } else {
            false
        }
    }
}

impl fmt::Display for Food {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.kind {
            FoodKind::Ration => "干粮",
            FoodKind::MysteryMeat if self.cooked => "熟肉",
            FoodKind::Pasty => "肉馅饼",
            FoodKind::MysteryMeat => "生肉",
            FoodKind::FrozenCarpaccio => "冰冻肉片",
        };

        let quantity = if self.quantity > 1 {
            format!("×{}", self.quantity)
        } else {
            String::new()
        };

        write!(f, "{}{}", name, quantity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_food_basics() {
        let mut ration = Food::new(FoodKind::Ration);
        assert_eq!(ration.eat(), 350);
        assert_eq!(ration.quantity, 0);
    }

    #[test]
    fn test_pasty_division() {
        let mut pasty = Food::new(FoodKind::Pasty);
        assert_eq!(pasty.quantity, 3);

        let piece = pasty.divide().unwrap();
        assert_eq!(pasty.quantity, 2);
        assert_eq!(piece.quantity, 1);
    }

    #[test]
    fn test_mystery_meat() {
        let mut raw_meat = Food::new(FoodKind::MysteryMeat);
        assert_eq!(raw_meat.eat(), 50); // 生吃效果减半

        raw_meat.cook();
        assert_eq!(raw_meat.eat(), 150);
    }

    #[test]
    fn test_contamination() {
        let mut food = Food::new(FoodKind::Ration);
        food.contaminated = true;
        assert_eq!(food.eat(), (350 as f32 * 0.7) as u32);
    }
}
