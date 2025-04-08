//! 核心游戏状态
//!
//! 实现像素地牢的核心游戏循环：
//! - 基于回合制的地牢探索
//! - 实时战斗系统
//! - 状态驱动渲染

use super::*;
use crate::ui::states::common;
use crate::ui::states::common::GameStateID;
use crate::ui::states::common::StateContext;
use crate::ui::states::common::StateTransition;
use crate::{
    dungeon::{Dungeon, Tile, Visibility},
    hero::{Hero, HeroAction},
    items::Item,
    monsters::Monster,
};
use crossterm::event::{KeyCode, KeyEvent};
use std::time::Duration;

/// 核心游戏状态
pub struct GameState {
    dungeon: Dungeon,
    hero: Hero,
    current_level: u8,
    is_paused: bool,
    turn_counter: u32,             // 回合计数器（用于怪物行动）
    action_queue: Vec<HeroAction>, // 行动队列（支持连续输入）
}

impl GameState {
    /// 创建新游戏状态（从指定层开始）
    pub fn new(starting_level: u8) -> Self {
        let mut dungeon = Dungeon::generate(starting_level);
        let (hero_x, hero_y) = dungeon.find_start_position();

        Self {
            dungeon,
            hero: Hero::new(hero_x, hero_y),
            current_level: starting_level,
            is_paused: false,
            turn_counter: 0,
            action_queue: Vec::with_capacity(3), // 预分配3个行动槽
        }
    }

    /// 处理英雄移动（包含碰撞检测）
    fn handle_hero_movement(&mut self, dx: i8, dy: i8) -> bool {
        let new_x = (self.hero.x as i16 + dx as i16) as u8;
        let new_y = (self.hero.y as i16 + dy as i16) as u8;

        // 检查目标位置是否可通行
        if let Some(tile) = self.dungeon.get_tile(new_x, new_y) {
            if tile.is_passable() {
                self.hero.x = new_x;
                self.hero.y = new_y;
                self.dungeon.update_visibility(new_x, new_y);
                return true;
            }
        }
        false
    }

    /// 处理与物品的交互
    fn handle_item_interaction(&mut self) {
        if let Some(item) = self.dungeon.get_item(self.hero.x, self.hero.y) {
            match item {
                Item::Potion => {
                    self.hero.drink_potion();
                    self.dungeon.remove_item(self.hero.x, self.hero.y);
                }
                Item::Scroll => {
                    self.hero.read_scroll();
                    self.dungeon.remove_item(self.hero.x, self.hero.y);
                } // 其他物品类型...
            }
        }
    }

    /// 处理与怪物的战斗
    fn handle_combat(&mut self) {
        if let Some(monster) = self.dungeon.get_monster(self.hero.x, self.hero.y) {
            let damage = self.hero.attack(&monster);
            if monster.health <= damage {
                self.dungeon.remove_monster(self.hero.x, self.hero.y);
                self.hero.gain_exp(monster.exp_value);
            } else {
                // 怪物反击
                let counter_damage = monster.attack(&self.hero);
                self.hero.take_damage(counter_damage);
            }
        }
    }

    /// 更新怪物行为（每10回合触发）
    fn update_monsters(&mut self) {
        if self.turn_counter % 10 == 0 {
            for monster in self.dungeon.monsters_mut() {
                if self.dungeon.is_visible(monster.x, monster.y) {
                    // 简单AI：向英雄移动
                    let dx = (self.hero.x as i16 - monster.x as i16).signum() as i8;
                    let dy = (self.hero.y as i16 - monster.y as i16).signum() as i8;

                    let new_x = (monster.x as i16 + dx as i16) as u8;
                    let new_y = (monster.y as i16 + dy as i16) as u8;

                    if self
                        .dungeon
                        .get_tile(new_x, new_y)
                        .map_or(false, |t| t.is_passable())
                    {
                        monster.x = new_x;
                        monster.y = new_y;
                    }
                }
            }
        }
    }

    /// 处理楼梯交互
    fn handle_stairs(&mut self) -> Option<GameStateID> {
        match self.dungeon.get_tile(self.hero.x, self.hero.y) {
            Some(Tile::StairsDown) => {
                self.current_level += 1;
                *self = Self::new(self.current_level);
                None
            }
            Some(Tile::StairsUp) if self.current_level > 1 => {
                self.current_level -= 1;
                *self = Self::new(self.current_level);
                None
            }
            _ => None,
        }
    }
}

impl common::GameState for GameState {
    fn id(&self) -> GameStateID {
        if self.is_paused {
            GameStateID::PauseMenu
        } else {
            GameStateID::Gameplay
        }
    }

    fn handle_input(
        &mut self,
        context: &mut StateContext,
        event: &crossterm::event::Event,
    ) -> bool {
        if let Some(key) = context.input.match_key(event) {
            match key {
                KeyCode::Esc => {
                    self.is_paused = !self.is_paused;
                    true
                }
                KeyCode::Char(' ') => {
                    self.action_queue.push(HeroAction::Wait);
                    true
                }
                KeyCode::Char('g') => {
                    self.handle_item_interaction();
                    true
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.action_queue.push(HeroAction::Move(0, -1));
                    true
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.action_queue.push(HeroAction::Move(0, 1));
                    true
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.action_queue.push(HeroAction::Move(-1, 0));
                    true
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.action_queue.push(HeroAction::Move(1, 0));
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn update(&mut self, context: &mut StateContext, delta_time: f32) -> Option<GameStateID> {
        // 处理行动队列
        while let Some(action) = self.action_queue.pop() {
            match action {
                HeroAction::Move(dx, dy) => {
                    if self.handle_hero_movement(dx, dy) {
                        self.handle_combat();
                        self.turn_counter += 1;
                        self.update_monsters();
                    }
                }
                HeroAction::Wait => {
                    self.turn_counter += 1;
                    self.update_monsters();
                }
            }

            // 检查楼梯交互
            if let Some(state) = self.handle_stairs() {
                return Some(state);
            }

            // 检查游戏结束条件
            if self.hero.is_dead() {
                return Some(GameStateID::GameOver);
            }
        }

        None
    }

    fn render(&mut self, context: &mut StateContext) -> anyhow::Result<()> {
        context
            .render
            .render_game(&mut context.terminal, &self.dungeon, &self.hero)
    }

    fn on_enter(&mut self, context: &mut StateContext) {
        // 初始化视野
        self.dungeon.update_visibility(self.hero.x, self.hero.y);

        // 播放背景音乐
        //context.audio.play_music("dungeon_theme.ogg");
    }

    fn pause_lower_states(&self) -> bool {
        self.is_paused
    }

    fn enter_transition(&self) -> Option<StateTransition> {
        Some(StateTransition::fade(0.5))
    }
}
