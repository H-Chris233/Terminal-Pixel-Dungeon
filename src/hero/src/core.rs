// src/hero/core.rs
use crate::bag::Bag;
use crate::bag::BagError;

use crate::{
    class::Class,
    effects::{EffectManager, EffectType},
    rng::HeroRng,
};

use combat::Combatant;
use dungeon::trap::Trap;


use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use thiserror::Error;

pub mod events;
pub mod item;

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

        // 根据职业初始化属性（SPD标准值）
        match hero.class {
            Class::Warrior => {
                hero.hp = 30;     // SPD标准：战士30生命值
                hero.max_hp = 30;
                hero.base_attack = 10;  // 初始基础攻击力
                hero.base_defense = 4; // 初始基础防御力
                hero.strength = 11;    // SPD标准：战士11力量
            }
            Class::Mage => {
                hero.hp = 20;     // SPD标准：法师20生命值
                hero.max_hp = 20;
                hero.base_attack = 8;
                hero.base_defense = 2;
                hero.strength = 10;    // SPD标准：法师10力量
            }
            Class::Rogue => {
                hero.hp = 25;     // SPD标准：盗贼25生命值
                hero.max_hp = 25;
                hero.base_attack = 6;
                hero.base_defense = 3;
                hero.strength = 10;    // SPD标准：盗贼10力量
            }
            Class::Huntress => {
                hero.hp = 22;     // SPD标准：女猎手22生命值
                hero.max_hp = 22;
                hero.base_attack = 5;
                hero.base_defense = 2;
                hero.strength = 10;    // SPD标准：女猎手10力量
            }
        }

        hero
    }

    pub fn on_turn(&mut self) -> Result<(), HeroError> {
        self.turns += 1;

    // SPD标准饥饿系统
    if self.turns % 20 == 0 {  // 每20回合减少1饥饿度（SPD标准）
        self.satiety = self.satiety.saturating_sub(1);
        
        // 饥饿状态效果
        match self.satiety {
            0 => { // 饥饿致死
                self.take_damage(1);
                return Err(HeroError::Starvation);
            }
            1..=5 => { // 饥饿状态：属性降低
                if self.satiety % 2 == 0 { // 每2回合掉血
                    self.take_damage(1);
                }
            }
            _ => {} // 正常状态
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
    fn handle_events(&mut self, events: Vec<dungeon::InteractionEvent>) -> Result<(), HeroError> {
        for event in events {
            match event {
                dungeon::InteractionEvent::TrapTriggered(effect) => self.apply_trap_effect(effect),
                dungeon::InteractionEvent::ItemFound(item) => self.bag.add_item(item)?,
                dungeon::InteractionEvent::EnemyEncounter(enemy) => self.enter_combat(enemy),
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
    pub fn apply_trap_effect(&mut self, effect: dungeon::trap::TrapEffect) {
        use dungeon::trap::TrapEffect as DungeonTrapEffect;
        match effect {
            DungeonTrapEffect::Damage(damage) => {
                self.take_damage(damage);
            }
            DungeonTrapEffect::Poison(_damage, turn) => {
                self.effects.add(crate::effects::Effect::new(EffectType::Poison, turn));
            }
            _ => {}
        };
    }

    /// 进入战斗状态
    pub fn enter_combat(&mut self, _enemy: combat::enemy::Enemy) {
        // 战斗初始化逻辑
    }

    pub fn notify(&self, _msg: &str) {
        // Non-critical: notification sink for now
    }
}



impl Default for Hero {
    fn default() -> Self {
        Self::new(Class::default())
    }
}
