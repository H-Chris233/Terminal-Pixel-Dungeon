//src/combat/enemy/enemy.rs

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub exp_value: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub enum EnemyKind {
    Rat,
    Snake,
    Gnoll,
    Crab,
    Bat,
    Scorpion,
    // 其他敌人类型...
}



