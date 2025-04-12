// src/dungeon/src/lib.rs

#![allow(dead_code)]
#![allow(unused)]

use bincode::{Decode, Encode};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod level;

use crate::level::Level;
use crate::level::tiles::{TerrainType, TileInfo};
use items::Item;


#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Dungeon {
    pub depth: usize,
    pub levels: Vec<Level>,
    pub seed: u64,
    pub max_depth: usize,
}

impl Dungeon {
    pub fn generate(max_depth: usize, seed: u64) -> anyhow::Result<Self> {
        let mut levels = Vec::with_capacity(max_depth);
        for _ in 1..=max_depth {
            levels.push(Level::generate(seed)?);
        }
        Ok(Self {
            depth: 1,
            seed,
            levels,
            max_depth,
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
        self.current_level().stair_down == (x, y)
    }

    pub fn can_ascend(&self, x: i32, y: i32) -> bool {
        self.depth > 1 && self.current_level().stair_up == (x, y)
    }

    pub fn remove_item(&mut self, x: i32, y: i32) -> Option<Item> {
        self.current_level_mut().remove_item(x, y)
    }

    pub fn get_item(&self, x: i32, y: i32) -> Option<Item> {
        self.current_level().get_item(x, y).cloned()
    }

    pub fn get_tile(&self, x: i32, y: i32) -> TileInfo {
        let level = self.current_level();
        TileInfo {
            passable: level.is_passable(x, y),
            has_item: level.items.iter().any(|item| item.x == x && item.y == y),
            has_enemy: level.enemies.iter().any(|enemy| enemy.x == x && enemy.y == y),
            blocks_sight: level.tiles.iter()
                .any(|t| t.x == x && t.y == y && t.info.blocks_sight),
            terrain_type: level.tiles.iter()
                .find(|t| t.x == x && t.y == y)
                .map(|t| t.info.terrain_type)
                .unwrap_or(TerrainType::Wall),
            is_visible: level.visible_tiles.contains(&(x, y)),
        }
    }

    pub fn has_monster(&self, x: i32, y: i32) -> bool {
        self.current_level().has_monster(x, y)
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
