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

// 重新导出主要类型
pub use self::{
    bag::{Bag},
    core::{Hero, HeroError},
    effects::EffectManager,
    rng::HeroRng,
};

use crate::class::Class;
use crate::effects::Effect;
use crate::effects::EffectType;

use ::combat::enemy::Enemy;
use combat::Combatant;
use dungeon::trap::Trap;
use dungeon::trap::TrapEffect;
use dungeon::InteractionEvent;
use items::Weapon;

// 游戏系统导入
use dungeon::Dungeon;
use items::{Item, ItemCategory};

/// 英雄系统主接口
pub trait HeroBehavior: Combatant + fmt::Debug {
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
    fn move_to(
        &mut self,
        dx: i32,
        dy: i32,
        dungeon: &mut Dungeon,
    ) -> Result<Vec<InteractionEvent>, HeroError>;

    /// 获取经验
    fn gain_exp(&mut self, exp: u32);
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
    fn add_item(&mut self, item: Item) -> Result<(), HeroError>;

    /// 移除物品
    fn remove_item(&mut self, index: usize) -> Result<(), HeroError>;

    /// 装备物品
    fn equip_item(&mut self, index: usize) -> Result<Option<Item>, HeroError>;

    /// 使用物品
    fn use_item(&mut self, index: usize) -> Result<Item, HeroError>;
}
