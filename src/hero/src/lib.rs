// src/hero/src/lib.rs
#![allow(dead_code)]
#![allow(unused)]

// 核心模块
mod bag;
mod combat;
mod core;
mod effects;
mod rng;

// 子模块
pub mod class;

// 标准库导入
use std::fmt;

// 外部crate导入
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// 重新导出主要类型
pub use self::{
    bag::{Bag, BagError},
    combat::Combatant,
    core::{Hero, HeroError},
    effects::EffectManager,
    rng::HeroRng,
};

use crate::class::Class;
use combat::effect::Effect;
use combat::effect::EffectType;

// 游戏系统导入
use dungeon::Dungeon;
use items::{Item, ItemCategory};

/// 英雄系统主接口
pub trait HeroBehavior: fmt::Debug {
    /// 创建新英雄
    fn new(class: Class) -> Self
    where
        Self: Sized;

    /// 带种子创建英雄
    fn with_seed(class: Class, seed: u64) -> Self
    where
        Self: Sized;

    /// 每回合更新
    fn on_turn(&mut self) -> Result<(), HeroError>;

    /// 移动英雄
    fn move_to(&mut self, dx: i32, dy: i32, dungeon: &mut Dungeon) -> Result<(), String>;

    /// 使用物品
    fn use_item(&mut self, category: ItemCategory, index: usize) -> Result<(), HeroError>;

    /// 获取经验
    fn gain_exp(&mut self, exp: u32);

    /// 获取当前生命值
    fn hp(&self) -> u32;

    /// 获取最大生命值
    fn max_hp(&self) -> u32;
}

/// 战斗系统接口
pub trait CombatSystem {
    /// 计算攻击力
    fn attack_power(&self) -> u32;

    /// 计算防御力
    fn defense(&self) -> u32;

    /// 承受伤害
    fn take_damage(&mut self, amount: u32) -> bool;

    /// 治疗
    fn heal(&mut self, amount: u32);

    /// 是否存活
    fn is_alive(&self) -> bool;
}

/// 效果系统接口
pub trait EffectSystem {
    /// 添加效果
    fn add(&mut self, effect: Effect);

    /// 移除效果
    fn remove(&mut self, effect_type: EffectType);

    /// 检查效果
    fn has(&self, effect_type: EffectType) -> bool;

    /// 更新效果
    fn update(&mut self);
}

/// 物品系统接口
pub trait InventorySystem {
    /// 添加物品
    fn add_item(&mut self, item: Item) -> Result<(), BagError>;

    /// 移除物品
    fn remove_item(&mut self, index: usize) -> Result<(), BagError>;

    /// 装备物品
    fn equip_item(&mut self, index: usize, strength: u8) -> Result<Option<Item>, BagError>;

    /// 使用物品
    fn use_item(&mut self, index: usize) -> Result<Item, BagError>;
}

// 实现模块组合
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct FullHero {
    core: Hero,
    combat: Box<dyn CombatSystem>,
    effects: EffectManager,
    rng: HeroRng,
    bag: Bag,
}

impl FullHero {
    pub fn new(class: Class) -> Self {
        Self::with_seed(class, rand::random())
    }

    pub fn with_seed(class: Class, seed: u64) -> Self {
        Self {
            core: Hero::with_seed(class, seed),
            combat: Box::new(DefaultCombatSystem::new()),
            effects: EffectManager::new(),
            rng: HeroRng::new(seed),
            bag: Bag::new(),
        }
    }
}

impl HeroBehavior for FullHero {
    fn new(class: Class) -> Self {
        FullHero::new(class)
    }

    fn with_seed(class: Class, seed: u64) -> Self {
        FullHero::with_seed(class, seed)
    }

    fn on_turn(&mut self) -> Result<(), HeroError> {
        self.core.on_turn()?;
        self.effects.update();
        Ok(())
    }

    fn move_to(&mut self, dx: i32, dy: i32, dungeon: &mut Dungeon) -> Result<(), String> {
        self.core.move_to(dx, dy, dungeon)
    }

    fn use_item(&mut self, category: ItemCategory, index: usize) -> Result<(), HeroError> {
        self.core.use_item(category, index)
    }

    fn gain_exp(&mut self, exp: u32) {
        self.core.gain_exp(exp)
    }

    fn hp(&self) -> u32 {
        self.core.hp
    }

    fn max_hp(&self) -> u32 {
        self.core.max_hp
    }
}

impl CombatSystem for FullHero {
    fn attack_power(&self) -> u32 {
        self.core.attack_power() + self.combat.attack_power()
    }

    fn defense(&self) -> u32 {
        self.core.defense() + self.combat.defense()
    }

    fn take_damage(&mut self, amount: u32) -> bool {
        self.core.take_damage(amount)
    }

    fn heal(&mut self, amount: u32) {
        self.core.heal(amount)
    }

    fn is_alive(&self) -> bool {
        self.core.is_alive()
    }
}

impl EffectSystem for FullHero {
    fn add_effect(&mut self, effect: Effect) {
        self.effects.add(effect)
    }

    fn remove_effect(&mut self, effect_type: EffectType) {
        self.effects.remove(effect_type)
    }

    fn has_effect(&self, effect_type: EffectType) -> bool {
        self.effects.has(effect_type)
    }

    fn update_effects(&mut self) {
        self.effects.update();
    }
}

impl InventorySystem for FullHero {
    fn add_item(&mut self, item: Item) -> Result<(), BagError> {
        self.core.add_item(item)
    }

    fn remove_item(&mut self, index: usize) -> Result<(), BagError> {
        self.core.remove_item(index)
    }

    fn equip_item(&mut self, index: usize, strength: u8) -> Result<Option<Item>, BagError> {
        self.core.equip_item(index, strength)
    }

    fn use_item(&mut self, index: usize) -> Result<Item, BagError> {
        self.core.use_item(index)
    }
}

/// 默认战斗系统实现
#[derive(Default)]
struct DefaultCombatSystem;

impl DefaultCombatSystem {
    fn new() -> Self {
        Self
    }
}

impl CombatSystem for DefaultCombatSystem {
    fn attack_power(&self) -> u32 {
        0 // 基础实现无额外加值
    }

    fn defense(&self) -> u32 {
        0 // 基础实现无额外防御
    }

    fn take_damage(&mut self, _amount: u32) -> bool {
        false // 基础实现不处理伤害
    }

    fn heal(&mut self, _amount: u32) {
        // 基础实现不处理治疗
    }

    fn is_alive(&self) -> bool {
        true // 基础实现总是存活
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHero {
        hp: u32,
    }

    impl HeroBehavior for MockHero {
        fn new(_class: Class) -> Self {
            MockHero { hp: 100 }
        }

        fn with_seed(_class: Class, _seed: u64) -> Self {
            MockHero { hp: 100 }
        }

        fn on_turn(&mut self) -> Result<(), HeroError> {
            Ok(())
        }

        fn move_to(&mut self, _dx: i32, _dy: i32, _dungeon: &mut Dungeon) -> Result<(), String> {
            Ok(())
        }

        fn use_item(&mut self, _category: ItemCategory, _index: usize) -> Result<(), HeroError> {
            Ok(())
        }

        fn gain_exp(&mut self, _exp: u32) {}

        fn hp(&self) -> u32 {
            self.hp
        }

        fn max_hp(&self) -> u32 {
            100
        }
    }

    #[test]
    fn test_mock_hero() {
        let mut hero = MockHero::new(Class::Warrior);
        assert_eq!(hero.hp(), 100);
        assert_eq!(hero.max_hp(), 100);
        assert!(hero.on_turn().is_ok());
    }
}
