use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 食物系统（3种类型）
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Food {
    pub kind: FoodKind,
    pub energy: f32, // 饱食度恢复量
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum FoodKind {
    Ration,      // 干粮
    Pasty,       // 肉馅饼
    MysteryMeat, // 神秘肉
}
