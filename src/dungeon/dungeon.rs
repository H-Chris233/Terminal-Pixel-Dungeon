// src/dungeon.rs
use crate::items::items::Item; // 完整路径
use bincode::{Decode, Encode};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Dungeon {
    pub depth: usize, // 当前层数(1-26)
    pub levels: Vec<Level>,
    pub seed: Option<u64>,
    // 其他地牢属性...
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Level {
    pub rooms: Vec<Room>,
    pub corridors: Vec<Corridor>,
    pub enemies: Vec<Enemy>,
    pub items: Vec<Item>,
    pub stairs_down: Vec<(i32, i32)>,
    pub stairs_up: Vec<(i32, i32)>,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Corridor {
    pub start: (i32, i32),
    pub end: (i32, i32),
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub exp_value: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub enum EnemyKind {
    Rat,
    Snake,
    Gnoll,
    Crab,
    // 其他敌人类型...
}

impl Dungeon {
    /// 生成指定深度的地牢
    pub fn generate(depth: usize) -> anyhow::Result<Self> {
        let seed = Some(rand::random()); // 生成随机种子
        let mut levels = Vec::with_capacity(depth);
        for d in 1..=depth {
            levels.push(Self::generate_level(d)?);
        }
        Ok(Self {
            depth: 1,
            seed,
            levels,
        })
    }

    /// 生成单层地牢
    fn generate_level(depth: usize) -> anyhow::Result<Level> {
        Ok(Level {
            stairs_down: Vec::new(), // 初始化下楼楼梯
            stairs_up: Vec::new(),   // 初始化上楼楼梯
            rooms: Vec::new(),
            corridors: Vec::new(),
            enemies: Vec::new(),
            items: Vec::new(),
        })
    }
    pub fn current_level(&self) -> &Level {
        &self.levels[self.depth - 1]
    }

    pub fn current_level_mut(&mut self) -> &mut Level {
        &mut self.levels[self.depth - 1]
    }

    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        self.current_level().is_passable(x, y)
    }
    pub fn can_descend(&self, x: i32, y: i32) -> bool {
        // 检查当前位置是否有下楼楼梯
        self.current_level()
            .stairs_down
            .iter()
            .any(|&(sx, sy)| sx == x && sy == y)
    }

    pub fn can_ascend(&self, x: i32, y: i32) -> bool {
        // 检查当前位置是否有上楼楼梯
        self.depth > 1 &&  // 不能从第一层上楼
        self.current_level().stairs_up.iter().any(|&(sx, sy)| sx == x && sy == y)
    }
}

impl Level {
    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        // 简单的可通行检查
        !self.enemies.iter().any(|e| e.x == x && e.y == y)
    }

    pub fn enemy_at(&mut self, x: i32, y: i32) -> Option<&mut Enemy> {
        self.enemies.iter_mut().find(|e| e.x == x && e.y == y)
    }

    pub fn item_at(&mut self, x: i32, y: i32) -> Option<Item> {
        // 简化实现
        None
    }
    pub fn take_item(&self, x: i32, y: i32) -> Option<Item> {
        //todo
        None
    }
}
