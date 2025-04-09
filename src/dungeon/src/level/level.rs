//src/dungeon/level/level.rs
use bincode::{Decode, Encode};
use items::Item;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::tiles::tiles::*;
use crate::level::rooms::rooms::Room;
use combat::enemy::enemy::Enemy;

/// 表示地牢的一个层级
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Level {
    pub rooms: Vec<Room>,
    pub corridors: Vec<Corridor>,
    pub enemies: Vec<Enemy>,
    pub items: Vec<Item>,
    pub stair_down: (i32, i32),
    pub stair_up: (i32, i32),
    pub tiles: Vec<Vec<Tile>>,
    pub width: i32,
    pub height: i32,
    pub visible_tiles: HashSet<(i32, i32)>,  // 可见的图块
    pub explored_tiles: HashSet<(i32, i32)>, // 已探索的图块
}

impl Level {
    /// 检查位置是否可通行
    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        // 边界检查
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return false;
        }

        // 检查是否有敌人
        let has_enemy = self.enemies.iter().any(|e| e.x == x && e.y == y);

        // 检查图块是否可通行
        let tile_passable = self
            .tiles
            .flatten()
            .find(|t| t.x == x && t.y == y)
            .map(|t| t.info.passable)
            .unwrap_or(false);

        tile_passable && !has_enemy
    }

    /// 获取指定位置的敌人
    pub fn enemy_at(&mut self, x: i32, y: i32) -> Option<&mut Enemy> {
        self.enemies.iter_mut().find(|e| e.x == x && e.y == y)
    }

    /// 获取指定位置的物品
    pub fn get_item(&self, x: i32, y: i32) -> Option<Item> {
        self.items.iter().find(|i| i.x == x && i.y == y)
    }

    /// 移除指定位置的物品
    pub fn remove_item(&mut self, x: i32, y: i32) -> Option<Item> {
        self.current_level_mut()
            .items
            .iter()
            .position(|item| (item.x, item.y) == (x, y))
            .map(|i| self.current_level_mut().items.remove(i))
    }

    /// 更新视野范围
    pub fn update_visibility(&mut self, x: i32, y: i32, radius: u8) {
        self.visible_tiles.clear();

        // 简单的圆形视野算法
        for dx in -(radius as i32)..=radius as i32 {
            for dy in -(radius as i32)..=radius as i32 {
                let nx = x + dx;
                let ny = y + dy;

                // 检查是否在视野范围内
                if dx * dx + dy * dy <= (radius as i32 * radius as i32)
                    && nx >= 0
                    && ny >= 0
                    && nx < self.width
                    && ny < self.height
                {
                    self.visible_tiles.insert((nx, ny));
                    self.explored_tiles.insert((nx, ny));

                    // 更新图块可见性
                    if let Some(tile) = self.tiles.iter_mut().find(|t| t.x == nx && t.y == ny) {
                        tile.info.is_visible = true;
                    }
                }
            }
        }
    }

    /// 检查位置是否有怪物
    pub fn has_monster(&self, x: i32, y: i32) -> bool {
        self.enemies.iter().any(|e| e.x == x && e.y == y)
    }

    /// 检查是否是楼梯
    pub fn is_stair(&self, x: i32, y: i32) -> bool {
        self.stair_down == (x, y) || self.stair_up == (x, y)
    }

    /// 检查位置是否被探索
    pub fn is_explored(&self, x: i32, y: i32) -> bool {
        self.explored_tiles.contains(&(x, y))
    }

    /// 检查位置是否可见
    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        self.visible_tiles.contains(&(x, y))
    }
}

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Corridor {
    pub start: (i32, i32),
    pub end: (i32, i32),
    pub tiles: Vec<Tile>,
}
