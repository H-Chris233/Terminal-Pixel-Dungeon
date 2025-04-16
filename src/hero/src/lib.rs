
// src/hero/src/lib.rs
#![allow(dead_code)]
#![allow(unused)]

// 核心模块
mod core;
mod combat;
mod effects;
mod rng;
mod bag;

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
}

/// 战斗系统接口
pub trait CombatSystem {
    /// 计算攻击力
    fn attack_power(&self) -> u32;
    
    /// 计算防御力
    fn defense(&self) -> u32;
    
    /// 承受伤害
    fn take_damage(&mut self, amount: u32) -> bool;
}

/// 效果系统接口
pub trait EffectSystem {
    /// 添加效果
    fn add_effect(&mut self, effect: Effect);
    
    /// 移除效果
    fn remove_effect(&mut self, effect_type: EffectType);
    
    /// 检查效果
    fn has_effect(&self, effect_type: EffectType) -> bool;
}

/// 物品系统接口
pub trait InventorySystem {
    /// 添加物品
    fn add_item(&mut self, item: Item) -> Result<(), BagError>;
    
    /// 移除物品
    fn remove_item(&mut self, index: usize) -> Result<(), BagError>;
    
    /// 装备物品
    fn equip_item(&mut self, index: usize, strength: u8) -> Result<Option<Item>, BagError>;
}

// 实现模块组合
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct FullHero {
    core: Hero,
    combat: CombatSystem,
    effects: EffectManager,
    rng: HeroRng,
    bag: Bag,
}

impl HeroBehavior for FullHero {
    // ... 实现所有trait方法 ...
}

