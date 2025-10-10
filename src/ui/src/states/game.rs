//! 核心游戏状态
//!
//! 实现像素地牢的核心游戏循环：
//! - 基于回合制的地牢探索
//! - 基本战斗系统
//! - 状态驱动渲染

use super::*;
use super::common::{GameState, GameStateID, StateContext, StateTransition};
use dungeon::Dungeon;
use dungeon::level::tiles::{StairDirection, TerrainType};
use hero::Hero;
use hero::class::Class;
use crate::terminal::TerminalController;
use crossterm::event::{KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 核心游戏状态
#[derive(Debug, Serialize)]
pub struct GameplayState {
    dungeon: Dungeon,
    hero: Hero,
    current_level: u8,
    is_paused: bool,
    turn_counter: u32,
    message_log: Vec<(String, MessageType)>,
    #[serde(skip)]
    last_input_time: Instant,
}

/// 消息类型分类
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MessageType {
    Good,   // 增益消息 (绿色)
    Normal, // 普通消息 (白色)
    Bad,    // 负面消息 (红色)
}

impl<'de> Deserialize<'de> for GameplayState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            dungeon: Dungeon,
            hero: Hero,
            current_level: u8,
            is_paused: bool,
            turn_counter: u32,
            message_log: Vec<(String, MessageType)>,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(GameplayState {
            dungeon: helper.dungeon,
            hero: helper.hero,
            current_level: helper.current_level,
            is_paused: helper.is_paused,
            turn_counter: helper.turn_counter,
            message_log: helper.message_log,
            last_input_time: std::time::Instant::now(), // Initialize with current time
        })
    }
}

impl GameplayState {
    /// 创建新游戏状态
    pub fn new(starting_level: u8) -> anyhow::Result<Self> {
        let dungeon = Dungeon::generate(5, 0)?;
        let (hero_x, hero_y) = dungeon.current_level().stair_up;

        Ok(Self {
            dungeon,
            hero: Hero::new(Class::default()),
            current_level: starting_level,
            is_paused: false,
            turn_counter: 0,
            message_log: vec![("Welcome to Pixel Dungeon!".to_string(), MessageType::Normal)],
            last_input_time: Instant::now(),
        })
    }

    /// 保存游戏状态
    pub fn save(&self, filename: &str) -> anyhow::Result<()> {
        let data = bincode::serde::encode_to_vec(self, bincode::config::standard())?;
        std::fs::write(filename, data)?;
        Ok(())
    }

    /// 加载游戏状态
    pub fn load(filename: &str) -> anyhow::Result<Self> {
        let data = std::fs::read(filename)?;
        let mut state: Self = bincode::serde::decode_from_slice(&data, bincode::config::standard())?.0;
        state.last_input_time = Instant::now();
        Ok(state)
    }

    /// 添加分类消息
    fn add_message(&mut self, msg: String, msg_type: MessageType) {
        self.message_log.push((msg, msg_type));
        if self.message_log.len() > 5 {
            self.message_log.remove(0);
        }
    }

    /// 处理英雄移动
    fn handle_hero_movement(&mut self, dx: i8, dy: i8) -> bool {
        let new_x = self.hero.x + dx as i32;
        let new_y = self.hero.y + dy as i32;

        let tile = self.dungeon.get_tile(new_x, new_y);
        if tile.passable && !self.dungeon.has_monster(new_x, new_y) {
            self.hero.x = new_x;
            self.hero.y = new_y;
            self.dungeon.update_visibility(new_x, new_y, 8);
            self.add_message(
                format!("Moved to ({}, {})", new_x, new_y),
                MessageType::Normal,
            );
            return true;
        }
        false
    }

    /// 处理物品交互
    fn handle_item_interaction(&mut self) {
        if let Some(item) = self.dungeon.take_item(self.hero.x, self.hero.y) {
            let _ = item; // integrate with hero inventory later
            self.add_message("Picked up an item".to_string(), MessageType::Good);
        } else {
            self.add_message("No item here".to_string(), MessageType::Normal);
        }
    }

    /// 处理楼梯交互
    fn handle_stairs(&mut self) -> Option<GameStateID> {
        if self.dungeon.can_descend(self.hero.x, self.hero.y) {
            let _ = self.dungeon.descend();
        } else if self.dungeon.can_ascend(self.hero.x, self.hero.y) {
            let _ = self.dungeon.ascend();
        }
        None
    }
}

impl GameState for GameplayState {
    fn id(&self) -> GameStateID {
        GameStateID::Gameplay
    }

    fn handle_input(
        &mut self,
        _context: &mut StateContext,
        event: &crossterm::event::Event,
    ) -> bool {
        if let crossterm::event::Event::Key(key) = event {
            // 限制输入频率
            if self.last_input_time.elapsed() < Duration::from_millis(100) {
                return false;
            }
            self.last_input_time = Instant::now();

            match key.code {
                KeyCode::Esc => {
                    self.is_paused = !self.is_paused;
                    return true;
                }
                KeyCode::Char(' ') => {
                    self.add_message("Hero waits...".to_string(), MessageType::Normal);
                }
                KeyCode::Char('g') => {
                    self.handle_item_interaction();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.handle_hero_movement(0, -1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.handle_hero_movement(0, 1);
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.handle_hero_movement(-1, 0);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.handle_hero_movement(1, 0);
                }
                KeyCode::Char('i') => {
                    return true;
                }
                KeyCode::Char('s') => {
                    if let Err(e) = self.save("save.dat") {
                        self.add_message(format!("Save failed: {}", e), MessageType::Bad);
                    } else {
                        self.add_message("Game saved".to_string(), MessageType::Good);
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn update(&mut self, _context: &mut StateContext, _delta_time: f32) -> Option<GameStateID> {
        if !self.hero.alive {
            return Some(GameStateID::GameOver);
        }
        self.handle_stairs()
    }

    fn render(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        context.render.render_game(
            &mut context.terminal,
            &self.dungeon,
            &self.hero,
        )
    }

    fn enter_transition(&self) -> Option<StateTransition> {
        Some(StateTransition::fade(0.5))
    }
}
