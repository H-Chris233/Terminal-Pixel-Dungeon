//! 核心游戏状态
//!
//! 实现像素地牢的核心游戏循环：
//! - 基于回合制的地牢探索
//! - 基本战斗系统
//! - 状态驱动渲染

use super::*;
use super::common::{GameState, GameStateID, StateContext, StateTransition};
use dungeon::Dungeon;
use dungeon::level::tiles::Tile;
use hero::{Hero, HeroAction};
use crate::terminal::TerminalController;
use crossterm::event::{KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 核心游戏状态
#[derive(Debug, Serialize, Deserialize)]
pub struct GameplayState {
    dungeon: Dungeon,
    hero: Hero,
    current_level: u8,
    is_paused: bool,
    turn_counter: u32,
    action_queue: Vec<HeroAction>,
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

impl GameplayState {
    /// 创建新游戏状态
    pub fn new(starting_level: u8) -> anyhow::Result<Self> {
        let dungeon = Dungeon::generate_with_retry(starting_level, 5)?;
        let (hero_x, hero_y) = dungeon.find_start_position();

        Ok(Self {
            dungeon,
            hero: Hero::new(hero_x, hero_y),
            current_level: starting_level,
            is_paused: false,
            turn_counter: 0,
            action_queue: Vec::with_capacity(3),
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
        let new_x = (self.hero.x as i16 + dx as i16) as u8;
        let new_y = (self.hero.y as i16 + dy as i16) as u8;

        if let Some(tile) = self.dungeon.get_tile(new_x, new_y) {
            if tile.is_passable() && !self.dungeon.has_monster(new_x, new_y) {
                self.hero.x = new_x;
                self.hero.y = new_y;
                self.dungeon.update_visibility(new_x, new_y, 8);
                self.add_message(
                    format!("Moved to ({}, {})", new_x, new_y),
                    MessageType::Normal,
                );
                return true;
            }
        }
        false
    }

    /// 处理物品交互
    fn handle_item_interaction(&mut self) {
        if let Some(item) = self.dungeon.get_item(self.hero.x, self.hero.y) {
            let result = self.hero.use_item(item);
            self.dungeon.remove_item(self.hero.x, self.hero.y);
            self.add_message(result.message, result.message_type);
        } else {
            self.add_message("No item here".to_string(), MessageType::Normal);
        }
    }

    /// 处理楼梯交互
    fn handle_stairs(&mut self) -> Option<GameStateID> {
        match self.dungeon.get_tile(self.hero.x, self.hero.y) {
            Some(Tile::StairsDown) => {
                self.current_level += 1;
                *self = Self::new(self.current_level).unwrap();
                None
            }
            Some(Tile::StairsUp) if self.current_level > 1 => {
                self.current_level -= 1;
                *self = Self::new(self.current_level).unwrap();
                None
            }
            _ => None,
        }
    }
}

impl GameState for GameplayState {
    fn id(&self) -> GameStateID {
        GameStateID::Gameplay
    }

    fn handle_input(
        &mut self,
        context: &mut StateContext,
        event: &crossterm::event::Event,
    ) -> Option<GameStateID> {
        if let crossterm::event::Event::Key(key) = event {
            // 限制输入频率
            if self.last_input_time.elapsed() < Duration::from_millis(100) {
                return None;
            }
            self.last_input_time = Instant::now();

            match key.code {
                KeyCode::Esc => {
                    self.is_paused = !self.is_paused;
                    if self.is_paused {
                        return Some(GameStateID::PauseMenu);
                    }
                }
                KeyCode::Char(' ') => {
                    self.action_queue.push(HeroAction::Wait);
                    self.add_message("Hero waits...".to_string(), MessageType::Normal);
                }
                KeyCode::Char('g') => {
                    self.action_queue.push(HeroAction::Interact);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.action_queue.push(HeroAction::Move(0, -1));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.action_queue.push(HeroAction::Move(0, 1));
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.action_queue.push(HeroAction::Move(-1, 0));
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.action_queue.push(HeroAction::Move(1, 0));
                }
                KeyCode::Char('i') => {
                    return Some(GameStateID::Inventory);
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
        None
    }

    fn update(&mut self, context: &mut StateContext, delta_time: f32) -> Option<GameStateID> {
        if self.is_paused {
            return None;
        }

        while let Some(action) = self.action_queue.pop() {
            match action {
                HeroAction::Move(dx, dy) => {
                    if self.handle_hero_movement(dx, dy) {
                        // 战斗逻辑已移至combat模块
                        self.turn_counter += 1;
                    }
                }
                HeroAction::Interact => {
                    self.handle_item_interaction();
                    self.turn_counter += 1;
                }
                HeroAction::Wait => {
                    self.turn_counter += 1;
                }
            }

            if let Some(state) = self.handle_stairs() {
                return Some(state);
            }

            if !self.hero.alive {
                return Some(GameStateID::GameOver);
            }
        }

        None
    }

    fn render(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        context.render.render_game(
            &mut context.terminal,
            &self.dungeon,
            &self.hero,
            &self.message_log,
            self.current_level,
            self.is_paused,
        )
    }

    fn pause_lower_states(&self) -> bool {
        self.is_paused
    }

    fn enter_transition(&self) -> Option<StateTransition> {
        Some(StateTransition::fade(0.5))
    }
}
