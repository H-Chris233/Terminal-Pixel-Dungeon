//! 核心游戏系统，用于整合 ECS 和模块化架构
//!
//! 这个模块提供了一个统一的接口来管理 ECS 实体、组件和系统，
//! 同时与各个功能模块（战斗、地牢、英雄等）进行交互。

use hecs::World;
use std::sync::{Arc, Mutex};

pub mod entity_factory;
pub mod game_state;

pub use entity_factory::EntityFactory;
pub use game_state::GameState;

/// 核心游戏引擎结构
pub struct GameEngine {
    /// ECS 实体世界
    pub world: Arc<Mutex<World>>,
    /// 游戏状态管理
    pub game_state: GameState,
    /// 实体工厂
    pub entity_factory: EntityFactory,
}

impl GameEngine {
    pub fn new() -> Self {
        Self {
            world: Arc::new(Mutex::new(World::new())),
            game_state: GameState::new(),
            entity_factory: EntityFactory::new(),
        }
    }

    /// 更新游戏状态
    pub fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 在这里可以集成 ECS 系统更新
        Ok(())
    }
}
