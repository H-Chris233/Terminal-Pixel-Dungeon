//src/items/src/food.rs
use bincode::serde::encode_to_vec;
use bincode::{Decode, Encode};
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hasher;
use tui::style::Color;

use crate::ItemCategory;
use crate::ItemTrait;
use crate::BINCODE_CONFIG;
use crate::Item;
use crate::ItemKind;

/// 食物系统（精确还原游戏机制）
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Food {
    pub kind: FoodKind,
    pub energy: u32,        // 饱食度恢复量
    pub quantity: u8,       // 数量（如肉馅饼可能有多个）
    pub cooked: bool,       // 是否已烹饪（仅对神秘肉有效）
    pub contaminated: bool, // 是否被污染（效果降低）
}

/// 食物类型（3种基础类型+特殊类型）
#[derive(Copy, Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum FoodKind {
    Ration,          // 干粮 - 标准食物
    Pasty,           // 肉馅饼（可分割）
    MysteryMeat,     // 神秘肉 - 可烹饪或生吃
    FrozenCarpaccio, // 冰冻肉片（特殊）
}

impl Food {
    /// 创建新食物
    pub fn new(kind: FoodKind) -> Self {
        let (energy, quantity) = match kind {
            FoodKind::Ration => (350, 1),
            FoodKind::Pasty => (450, 3), // 肉馅饼初始可分割为3份
            FoodKind::MysteryMeat => (100, 1),
            FoodKind::FrozenCarpaccio => (200, 1),
        };

        Food {
            kind,
            energy,
            quantity,
            cooked: false,
            contaminated: false,
        }
    }

    /// 获取食物名称（用于UI显示）
    pub fn name(&self) -> String {
        match self.kind {
            FoodKind::Ration => "干粮".to_string(),
            FoodKind::Pasty => "肉馅饼".to_string(),
            FoodKind::MysteryMeat if self.cooked => "熟肉".to_string(),
            FoodKind::MysteryMeat => "生肉".to_string(),
            FoodKind::FrozenCarpaccio => "冰冻肉片".to_string(),
        }
    }

    /// 随机生成新食物
    pub fn random_new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();

        let kinds = [
            FoodKind::Ration,
            FoodKind::Pasty,
            FoodKind::MysteryMeat,
            FoodKind::FrozenCarpaccio,
        ];
        let kind = kinds[rng.random_range(0..kinds.len())];

        let mut food = Food::new(kind);

        // 如果是神秘肉，有30%概率已烹饪
        if let FoodKind::MysteryMeat = kind {
            if rng.random_bool(0.3) {
                food.cook();
            }
        }

        // 10%概率被污染
        if rng.random_bool(0.1) {
            food.contaminated = true;
        }

        food
    }

    /// 计算食物基础价值（考虑类型、状态和数量）
    pub fn value(&self) -> u32 {
        // 基础价值
        let base_value = match self.kind {
            FoodKind::Ration => 50,
            FoodKind::Pasty => 100,
            FoodKind::MysteryMeat if self.cooked => 80,
            FoodKind::MysteryMeat => 30,
            FoodKind::FrozenCarpaccio => 120,
        };

        // 状态修正
        let mut value = if self.contaminated {
            (base_value as f32 * 0.6) as u32 // 污染食物价值降低40%
        } else {
            base_value
        };

        // 数量加成（线性增长）
        value * self.quantity as u32
    }

    /// 获取食物颜色（整合污染状态）
    pub fn color(&self) -> Color {
        // 污染状态优先显示
        if self.contaminated {
            return Color::Green; // 污染状态使用绿色
        }

        match self.kind {
            FoodKind::Ration => Color::Rgb(210, 180, 140), // 沙金色
            FoodKind::Pasty => Color::LightRed,
            FoodKind::MysteryMeat if self.cooked => Color::Rgb(139, 69, 19), // 棕色
            FoodKind::MysteryMeat => Color::Red,
            FoodKind::FrozenCarpaccio => Color::LightBlue,
        }
    }

    /// 分割食物（仅肉馅饼可分割）
    pub fn divide(&mut self) -> Option<Food> {
        if let FoodKind::Pasty = self.kind {
            if self.quantity > 1 {
                self.quantity -= 1;
                Some(Food {
                    kind: FoodKind::Pasty,
                    energy: self.energy / 3,
                    quantity: 1,
                    cooked: false,
                    contaminated: self.contaminated,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 食用食物（返回实际恢复的饱食度）
    pub fn eat(&mut self) -> u32 {
        if self.quantity == 0 {
            return 0;
        }

        self.quantity -= 1;
        let mut actual_energy = self.energy;

        // 污染效果
        if self.contaminated {
            actual_energy = (actual_energy as f32 * 0.7) as u32;
        }

        // 特殊效果处理
        match self.kind {
            FoodKind::MysteryMeat if !self.cooked => {
                // 生吃神秘肉有50%效果
                actual_energy /= 2;
            }
            FoodKind::FrozenCarpaccio => {
                // 冰冻肉片有额外冷却效果
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
        let name = self.name();
        let quantity = if self.quantity > 1 {
            format!("×{}", self.quantity)
        } else {
            String::new()
        };

        write!(f, "{}{}", name, quantity)
    }
}

impl Default for Food {
    fn default() -> Self {
        Food {
            kind: FoodKind::Ration, // 默认类型：干粮
            energy: 350,            // 标准饱食度
            quantity: 1,            // 单个物品
            cooked: false,          // 未烹饪（对干粮无意义）
            contaminated: false,    // 未污染
        }
    }
}

impl From<FoodKind> for Food {
    fn from(kind: FoodKind) -> Self {
        Food::new(kind)
    }
}

impl ItemTrait for Food {
    /// 生成堆叠标识（基于关键属性）
    fn stacking_id(&self) -> u64 {
        let mut hasher = SeaHasher::new();
        // 关键区分属性：类型 + 烹饪状态 + 污染状态
        let key = (self.kind, self.cooked, self.contaminated);
        let bytes = encode_to_vec(key, BINCODE_CONFIG).unwrap();
        hasher.write(&bytes);
        hasher.finish()
    }

    /// 食物可以堆叠（相同类型和状态）
    fn is_stackable(&self) -> bool {
        true
    }

    /// 设置差异化的最大堆叠数量（基于游戏平衡）
    fn max_stack(&self) -> u32 {
        match self.kind {
            FoodKind::Ration => 8,          // 基础干粮高堆叠
            FoodKind::Pasty => 4,           // 肉馅饼中等堆叠
            FoodKind::MysteryMeat => 3,     // 生肉低堆叠
            FoodKind::FrozenCarpaccio => 2, // 特殊物品最低堆叠
        }
    }
    fn display_name(&self) -> String {
        self.name()
    }
    fn category(&self) -> ItemCategory {
        ItemCategory::Food
    }
    fn sort_value(&self) -> u32 {
        match self.kind {
            FoodKind::Ration => 100,
            FoodKind::Pasty => 90,
            FoodKind::MysteryMeat => 80,
            _ => 0,
        }
    }
}

impl From<Food> for Item {
    fn from(food: Food) -> Self {
        Item {
            name: food.name(),
            kind: ItemKind::Food(food),
            description: "...".to_string(),
            quantity: 1,
            x: -1,
            y: -1,
        }
    }
}
