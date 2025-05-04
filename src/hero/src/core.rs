// src/hero/core.rs
use crate::bag::Bag;
use crate::bag::BagError;
use crate::HeroBehavior;
use crate::InventorySystem;
use crate::{
    class::Class,
    effects::{Effect, EffectManager, EffectType},
    rng::HeroRng,
};

use combat::enemy::Enemy;
use combat::Combatant;
use combat::EffectType::Poison;
use dungeon::trap::Trap;
use dungeon::trap::TrapEffect;
use dungeon::Dungeon;
use dungeon::InteractionEvent;
use thiserror::Error;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub mod events;
pub mod item;

pub use self::events::*;
pub use items::*;

#[derive(Debug, Error)]
pub enum HeroError {
    #[error("无法执行此动作")]
    ActionFailed,
    #[error("力量不足")]
    Underpowered,
    #[error("饥饿过度")]
    Starvation,
    #[error("效果冲突")]
    EffectConflict,
    #[error("被控制效果影响")]
    Immobilized,
    #[error("背包已满")]
    BagFull(#[from] BagError),
}

/// 英雄核心数据结构
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode)]
pub struct Hero {
    // 基础属性
    pub class: Class,
    pub name: String,
    pub hp: u32,
    pub max_hp: u32,
    pub base_attack: u32,
    pub base_defense: u32,

    // 成长系统
    pub experience: u32,
    pub level: u32,
    pub strength: u8,
    pub satiety: u8,

    // 游戏进度
    pub gold: u32,
    pub x: i32,
    pub y: i32,
    pub alive: bool,
    pub turns: u32,

    // 子系统
    pub effects: EffectManager,
    pub rng: HeroRng,
    pub bag: Bag,
}

impl Hero {
    pub fn new(class: Class) -> Self {
        Self::with_seed(class, rand::random())
    }

    pub fn with_seed(class: Class, seed: u64) -> Self {
        let mut hero = Self {
            class: class.clone(),
            name: "Adventurer".to_string(),
            hp: 0,
            max_hp: 0,
            base_attack: 0,
            base_defense: 0,
            experience: 0,
            level: 1,
            strength: 10,
            satiety: 5,
            gold: 0,
            x: 0,
            y: 0,
            alive: true,
            turns: 0,
            effects: EffectManager::new(),
            rng: HeroRng::new(seed),
            bag: Bag::new(),
        };

        // 根据职业初始化属性
        match hero.class {
            Class::Warrior => {
                hero.hp = 25;
                hero.max_hp = 25;
                hero.base_attack = 10;
                hero.base_defense = 4;
                hero.strength += 1;
            }
            Class::Mage => {
                hero.hp = 20;
                hero.max_hp = 20;
                hero.base_attack = 8;
                hero.base_defense = 2;
            }
            Class::Rogue => {
                hero.hp = 22;
                hero.max_hp = 22;
                hero.base_attack = 6;
                hero.base_defense = 3;
            }
            Class::Huntress => {
                hero.hp = 20;
                hero.max_hp = 20;
                hero.base_attack = 5;
                hero.base_defense = 2;
            }
        }

        hero
    }

    pub fn on_turn(&mut self) -> Result<(), HeroError> {
        self.turns += 1;

        // 饥饿系统
        if self.turns % 100 == 0 {
            self.satiety = self.satiety.saturating_sub(1);
            if self.satiety == 0 {
                self.take_damage(1);
                return Err(HeroError::Starvation);
            }
        }

        // 更新效果
        self.effects.update();
        Ok(())
    }

    pub fn level_up(&mut self) {
        self.level += 1;
        self.max_hp += self.class.hp_per_level();
        self.hp = self.max_hp;
        self.base_attack += self.class.attack_per_level();
        self.base_defense += self.class.defense_per_level();

        if self.level % 4 == 0 {
            self.strength += 1;
        }
    }

    /// 增强的事件处理
    fn handle_events(&mut self, events: Vec<InteractionEvent>) -> Result<(), HeroError> {
        for event in events {
            match event {
                InteractionEvent::TrapTriggered(effect) => self.apply_trap_effect(effect),
                InteractionEvent::ItemFound(item) => self.add_item(item)?,
                InteractionEvent::EnemyEncounter(enemy) => self.enter_combat(enemy),
                _ => {}
            }
        }
        Ok(())
    }

    pub fn gain_exp(&mut self, exp: u32) {
        self.experience += exp;
        while self.experience >= self.level * 100 {
            self.experience -= self.level * 100;
            self.level_up();
        }
    }

    pub fn trigger_trap(&mut self, trap: &mut Trap) -> Result<(), HeroError> {
        if !trap.is_active() {
            return Err(HeroError::ActionFailed);
        }

        if let Some(effect) = trap.trigger() {
            self.apply_trap_effect(effect);
        }
        Ok(())
    }

    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn is_immobilized(&self) -> bool {
        self.effects.has(EffectType::Paralysis) || self.effects.has(EffectType::Rooted)
    }

    /// 应用陷阱效果
    pub fn apply_trap_effect(&mut self, effect: TrapEffect) {
        match effect {
            TrapEffect::Damage(damage) => {
                self.take_damage(damage);
            }
            TrapEffect::Poison(_, turn) => {
                self.effects.add(Effect::new(EffectType::Poison, turn));
            }
            _ => {}
        };
    }

    /// 进入战斗状态
    pub fn enter_combat(&mut self, enemy: Enemy) {
        // 战斗初始化逻辑
    }

    pub fn notify(&self, msg: &str) {
        todo!();
    }
}

impl HeroBehavior for Hero {
    fn move_to(
        &mut self,
        dx: i32,
        dy: i32,
        dungeon: &mut Dungeon,
    ) -> Result<Vec<InteractionEvent>, HeroError> {
        if self.is_immobilized() {
            return Err(HeroError::Immobilized);
        }

        let new_x = self.x.saturating_add(dx);
        let new_y = self.y.saturating_add(dy);

        // 边界检查
        if !dungeon.is_passable(new_x, new_y) {
            return Err(HeroError::ActionFailed);
        }

        // 更新位置
        self.x = new_x;
        self.y = new_y;

        // 获取事件
        let events = dungeon.on_hero_enter(new_x, new_y);
        self.handle_events(events.clone())?;
        Ok(events)
    }

    /// 创建新英雄
    fn new(class: Class) -> Self
    where
        Self: Sized,
    {
        Hero::new(class)
    }

    /// 带种子创建英雄
    fn with_seed(class: Class, seed: u64) -> Self
    where
        Self: Sized,
    {
        Hero::with_seed(class, seed)
    }

    /// 每回合更新
    fn on_turn(&mut self) -> Result<(), HeroError> {
        self.on_turn()
    }

    /// 获取经验
    fn gain_exp(&mut self, exp: u32) {
        self.gain_exp(exp)
    }
}

impl Default for Hero {
    fn default() -> Self {
        Self::new(Class::default())
    }
}
