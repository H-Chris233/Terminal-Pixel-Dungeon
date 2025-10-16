// src/dungeon/src/lib.rs

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub mod level;
pub mod trap;

pub fn affect_adjacent_enemies(_x: i32, _y: i32, _f: impl Fn(&mut Enemy)) {}
pub fn reveal_current_level(_x: i32, _y: i32) {}
pub fn alert_nearby_enemies(_x: i32, _y: i32) {}

use crate::level::Level;
pub use crate::level::tiles::{TerrainType, TileInfo};
use crate::trap::TrapEffect;

use combat::enemy::Enemy;
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

    pub fn is_door(&self, x: i32, y: i32) -> bool {
        self.current_level().is_door(x, y)
    }

    pub fn can_descend(&self, x: i32, y: i32) -> bool {
        self.current_level().stair_down == (x, y)
    }

    pub fn can_ascend(&self, x: i32, y: i32) -> bool {
        self.depth > 1 && self.current_level().stair_up == (x, y)
    }

    pub fn take_item(&mut self, x: i32, y: i32) -> Option<Item> {
        self.current_level_mut().take_item(x, y)
    }

    pub fn get_item(&self, x: i32, y: i32) -> Option<Item> {
        self.current_level().get_item(x, y).cloned()
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
            blocks_sight: level
                .tiles
                .iter()
                .any(|t| t.x == x && t.y == y && t.info.blocks_sight),
            terrain_type: level
                .tiles
                .iter()
                .find(|t| t.x == x && t.y == y)
                .map(|t| t.info.terrain_type.clone())
                .unwrap_or(TerrainType::Wall),
            is_visible: level.visible_tiles.contains(&(x, y)),
            explored: false,
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

    /// 统一处理英雄进入新格子的所有交互
    pub fn on_hero_enter(&mut self, x: i32, y: i32) -> Vec<InteractionEvent> {
        self.on_hero_enter_with_events(x, y)
    }

    /// 统一处理英雄进入新格子的所有交互
    pub fn on_hero_enter_with_events(&mut self, x: i32, y: i32) -> Vec<InteractionEvent> {
        let mut events = Vec::new();

        // 1. 陷阱检测（优先处理）
        if let Some(mut trap) = self.current_level_mut().get_trap(x, y)
            && let Some(effect) = trap.trigger()
        {
            events.push(InteractionEvent::TrapTriggered(effect.clone()));
        }

        // 2. 物品拾取
        if let Some(item) = self.take_item(x, y) {
            events.push(InteractionEvent::ItemFound(item.clone()));
            // 实际拾取逻辑交给游戏状态机处理
        }

        // 3. 敌人遭遇
        if let Some(enemy) = self.current_level().enemy_at(x, y) {
            events.push(InteractionEvent::EnemyEncounter(enemy.clone()));
        }

        // 4. 楼梯检测
        if self.can_ascend(x, y) {
            events.push(InteractionEvent::StairsUp);
        } else if self.can_descend(x, y) {
            events.push(InteractionEvent::StairsDown);
        }

        events
    }

    /// 获取当前层的交互状态
    pub fn get_tile_interactions(&self, x: i32, y: i32) -> TileInteraction {
        TileInteraction {
            has_trap: self.current_level().has_trap(x, y),
            has_item: self.get_item(x, y).is_some(),
            is_stair: self.can_descend(x, y) || self.can_ascend(x, y),
            is_door: self.is_door(x, y),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InteractionEvent {
    TrapTriggered(TrapEffect),
    ItemFound(Item),
    EnemyEncounter(Enemy),
    StairsUp,
    StairsDown,
    DoorOpened(i32, i32),
    SecretRevealed(i32, i32),
}

/// 用于UI显示的交互信息
#[derive(Debug, Clone)]
pub struct TileInteraction {
    pub has_trap: bool,
    pub has_item: bool,
    pub is_stair: bool,
    pub is_door: bool,
}
