// src/dungeon/src/level/level.rs

use bincode::{Decode, Encode};
use rand::Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod rooms;
pub mod tiles;

use crate::level::tiles::Tile;
use items::Item;
use combat::enemy::Enemy;

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Level {
    pub rooms: Vec<Room>,
    pub corridors: Vec<Corridor>,
    pub enemies: Vec<Enemy>,
    pub items: Vec<Item>,
    pub stair_down: (i32, i32),
    pub stair_up: (i32, i32),
    pub tiles: Vec<Tile>,
    pub width: i32,
    pub height: i32,
    pub visible_tiles: HashSet<(i32, i32)>,
    pub explored_tiles: HashSet<(i32, i32)>,
}

impl Level {
    pub fn generate(seed: u64) -> anyhow::Result<Self> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let width = rng.random_range(50..100);
        let height = rng.random_range(50..100);
        
        Ok(Self {
            rooms: Vec::new(),
            corridors: Vec::new(),
            enemies: Vec::new(),
            items: Vec::new(),
            stair_down: (0, 0),
            stair_up: (0, 0),
            tiles: Vec::new(),
            width,
            height,
            visible_tiles: HashSet::new(),
            explored_tiles: HashSet::new(),
        })
    }

    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return false;
        }

        let has_enemy = self.has_monster(x, y);
        let tile_passable = self.tiles.iter()
            .find(|t| t.x == x && t.y == y)
            .map(|t| t.info.passable)
            .unwrap_or(false);

        tile_passable && !has_enemy
    }

    pub fn enemy_at(&mut self, x: i32, y: i32) -> Option<&mut Enemy> {
        self.enemies.iter_mut().find(|e| e.x == x && e.y == y)
    }

    pub fn get_item_name(&self, x: i32, y: i32) -> Option<&Item> {
        self.items.iter().find(|i| i.x == x && i.y == y)
    }

    pub fn take_item(&mut self, x: i32, y: i32) -> Option<Item> {
        self.items.iter()
            .position(|item| item.x == x && item.y == y)
            .map(|i| self.items.remove(i))
    }

    pub fn update_visibility(&mut self, x: i32, y: i32, radius: u8) {
        self.visible_tiles.clear();

        for dx in -(radius as i32)..=radius as i32 {
            for dy in -(radius as i32)..=radius as i32 {
                let nx = x + dx;
                let ny = y + dy;

                if dx * dx + dy * dy <= (radius as i32).pow(2)
                    && nx >= 0 && ny >= 0
                    && nx < self.width && ny < self.height
                {
                    self.visible_tiles.insert((nx, ny));
                    self.explored_tiles.insert((nx, ny));

                    if let Some(tile) = self.tiles.iter_mut()
                        .find(|t| t.x == nx && t.y == ny) 
                    {
                        tile.info.is_visible = true;
                    }
                }
            }
        }
    }

    pub fn has_monster(&self, x: i32, y: i32) -> bool {
        self.enemies.iter().any(|e| e.x == x && e.y == y)
    }

    pub fn is_stair(&self, x: i32, y: i32) -> bool {
        self.stair_down == (x, y) || self.stair_up == (x, y)
    }

    pub fn is_explored(&self, x: i32, y: i32) -> bool {
        self.explored_tiles.contains(&(x, y))
    }

    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        self.visible_tiles.contains(&(x, y))
    }
}

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Corridor {
    pub start: (i32, i32),
    pub end: (i32, i32),
    pub tiles: Vec<(i32, i32)>,
}


