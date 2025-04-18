
// src/hero/core.rs
use crate::{
    class::Class,
    effects::{Effect, EffectManager, EffectType},
    rng::HeroRng,
};
use crate::HeroBehavior;


use dungeon::InteractionEvent;
use combat::{Combatant};
use dungeon::trap::TrapEffect;
use dungeon::trap::Trap;
use dungeon::Dungeon;
use thiserror::Error;

use serde::{Deserialize, Serialize};

pub mod events;
pub mod item;

pub use ::items::*;
pub use self::events::*;

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
}

/// 英雄核心数据结构
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub satiety: u8, // 0-10, 0=饥饿, 5=正常, 10=饱食

    // 游戏进度
    pub gold: u32,
    pub x: i32,
    pub y: i32,
    pub alive: bool,
    pub turns: u32,

    // 子系统
    pub effects: EffectManager,
    pub rng: HeroRng,
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

    /// 移动并返回交互事件
    pub fn move_to(
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

        // 基础验证
        if !dungeon.current_level().in_bounds(new_x, new_y) {
            return Err(HeroError::ActionFailed);
        }
        if !dungeon.is_passable(new_x, new_y) {
            return Err(HeroError::ActionFailed);
        }

        // 更新位置
        self.x = new_x;
        self.y = new_y;

        // 触发地图交互
        Ok(dungeon.on_hero_enter(self, new_x, new_y))
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

        let effect = trap.trigger(&mut self.rng);
        match effect {
            TrapEffect::Damage(amount) => {
                self.take_damage(amount);
            }
            TrapEffect::Poison(dmg, turns) => {
                self.effects.add(Effect::poison(dmg, turns));
            }
            _ => {}
        }

        Ok(())
    }

    pub fn take_damage(&mut self, amount: u32) -> bool {
        let defense_roll = self.defense() as f32 * (0.7 + self.rng.gen_range(0.0..0.6));
        let actual_damage = (amount as f32 - defense_roll).max(1.0) as u32;

        self.hp = self.hp.saturating_sub(actual_damage);
        self.alive = self.hp > 0;
        self.alive
    }

    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn defense(&self) -> u32 {
        self.base_defense // 装备防御由外部系统计算
    }

    pub fn is_immobilized(&self) -> bool {
        self.effects.has(EffectType::Paralysis) || self.effects.has(EffectType::Rooted)
    }
}

impl Default for Hero {
    fn default() -> Self {
        Self::new(Class::default())
    }
}

impl HeroBehavior for Hero {
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

    /// 移动英雄
    fn move_to(&mut self, dx: i32, dy: i32, dungeon: &mut Dungeon) -> Result<(), String> {
        match Hero::move_to(self, dx, dy, dungeon) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    /// 使用物品
    fn use_item(&mut self, category: ItemCategory, index: usize) -> Result<(), HeroError> {
        // TODO: Implement actual item usage logic
        // For now, just return ActionFailed as a placeholder
        Err(HeroError::ActionFailed)
    }

    /// 获取经验
    fn gain_exp(&mut self, exp: u32) {
        self.gain_exp(exp)
    }

    /// 获取当前生命值
    fn hp(&self) -> u32 {
        self.hp
    }

    /// 获取最大生命值
    fn max_hp(&self) -> u32 {
        self.max_hp
    }
}
