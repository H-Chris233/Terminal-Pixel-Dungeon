//src/dungeon/src/lib.rs
use bincode::{Decode, Encode};
use items::Item; // 完整路径
use rand::Rng;
use serde::{Deserialize, Serialize};

pub mod level;

use crate::level::level::*;
use crate::level::tiles::tiles::TileInfo;

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Dungeon {
    pub depth: usize, // 当前层数(1-26)
    pub levels: Vec<Level>,
    pub seed: u64,
    pub max_depth: usize,
    // 其他地牢属性...
}

impl Dungeon {
    /// 生成指定深度的地牢
    pub fn generate(max_depth: usize, seed: u64) -> anyhow::Result<Self> {
        let mut levels = Vec::with_capacity(max_depth);
        for _ in 1..=max_depth {
            levels.push(Self::generate_level(seed)?);
        }
        Ok(Self {
            depth: 1,
            seed,
            levels,
            max_depth,
        })
    }

    /// 生成单层地牢
    fn generate_level(seed: u64) -> anyhow::Result<Level> {
        Ok(Level {
            stair_down: (0, 0), // 初始化下楼楼梯
            stair_up: (0, 0),   // 初始化上楼楼梯
            rooms: Vec::new(),
            corridors: Vec::new(),
            enemies: Vec::new(),
            items: Vec::new(),
            tiles: Vec::new(),
            width: i32,
            height: i32,
            visible_tiles: HashSet::new(), // 可见的图块
            explored_tiles: HashSet::new(),
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
        self.current_level().stair_down == (x, y)
    }

    pub fn can_ascend(&self, x: i32, y: i32) -> bool {
        // 检查当前位置是否有上楼楼梯
        self.depth > 1 &&  // 不能从第一层上楼
        self.current_level().stair_up == (x, y)
    }
    pub fn remove_item(&mut self, x: i32, y: i32) -> Option<Item> {
        self.current_level_mut().remove_item(x, y)
    }
    pub fn get_item(&self, x: i32, y: i32) -> Option<Item> {
        self.current_level().get_item(x, y)
    }
    pub fn get_tile(&self, x: i32, y: i32) -> TileInfo {
        let level = self.current_level();
        TileInfo {
            passable: level.is_passable(x, y),
            has_item: level.items.iter().any(|item| item.x == x && item.y == y),
            has_enemy: level
                .enemies
                .iter()
                .any(|enemy| enemy.x == x && enemy.y == y),
            blocks_sight: bool, // 是否阻挡视线
            terrain_type: TerrainType,
            is_visible: bool,
        }
    }
    pub fn has_monster(&self, x: i32, y: i32) -> bool {
        self.current_level()
            .enemies
            .iter()
            .any(|e| e.x == x && e.y == y)
    }
    pub fn update_visibility(&mut self, x: i32, y: i32, radius: u8) {
        self.current_level_mut().update_visibility(x, y, radius);
    }
    pub fn descend(&mut self) -> anyhow::Result<()> {
        if self.depth >= self.max_depth {
            return Err(anyhow::anyhow!("已达最底层"));
        }
        self.depth += 1;
        Ok(())
    }

    pub fn ascend(&mut self) -> anyhow::Result<()> {
        if self.depth <= 1 {
            return Err(anyhow::anyhow!("已在顶层"));
        }
        self.depth -= 1;
        Ok(())
    }
}
