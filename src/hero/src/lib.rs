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
use anyhow::Result;
use std::fmt;

// 外部crate导入
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// 重新导出主要类型
pub use self::{
    bag::{Bag, BagError},
    core::{Hero, HeroError},
    effects::EffectManager,
    rng::HeroRng,
};

use crate::class::Class;
use combat::effect::*;
use combat::Combatant;
use dungeon::trap::Trap;
use items::Weapon;

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

    /// 获取英雄名称
    pub fn name(&self) -> &str {
        &self.core.name
    }

    /// 设置英雄名称
    pub fn set_name(&mut self, name: String) {
        self.core.name = name;
    }

    /// 获取当前等级
    pub fn level(&self) -> u32 {
        self.core.level
    }

    /// 获取当前经验值
    pub fn experience(&self) -> u32 {
        self.core.experience
    }

    /// 获取当前金币
    pub fn gold(&self) -> u32 {
        self.core.gold
    }

    /// 添加金币
    pub fn add_gold(&mut self, amount: u32) {
        self.core.gold += amount;
    }

    /// 消耗金币
    pub fn spend_gold(&mut self, amount: u32) -> Result<(), HeroError> {
        if self.core.gold >= amount {
            self.core.gold -= amount;
            Ok(())
        } else {
            Err(HeroError::Underpowered)
        }
    }

    /// 获取当前饱食度
    pub fn satiety(&self) -> u8 {
        self.core.satiety
    }

    /// 增加饱食度
    pub fn feed(&mut self, amount: u8) {
        self.core.satiety = (self.core.satiety + amount).min(10);
    }

    /// 获取当前回合数
    pub fn turns(&self) -> u32 {
        self.core.turns
    }

    /// 获取英雄职业
    pub fn class(&self) -> &Class {
        &self.core.class
    }

    /// 获取英雄位置
    pub fn position(&self) -> (i32, i32) {
        (self.core.x, self.core.y)
    }

    /// 设置英雄位置
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.core.x = x;
        self.core.y = y;
    }

    /// 检查英雄是否被控制（无法移动）
    pub fn is_immobilized(&self) -> bool {
        self.core.is_immobilized()
    }

    /// 执行攻击
    pub fn perform_attack(&mut self, target: &mut dyn Combatant) -> (bool, u32) {
        self.core.perform_attack(target)
    }

    /// 计算命中概率
    pub fn hit_probability(&self, target: &dyn Combatant) -> f32 {
        self.core.hit_probability(target)
    }

    /// 尝试闪避攻击
    pub fn try_evade(&mut self, attacker: &dyn Combatant) -> bool {
        self.core.try_evade(attacker)
    }

    /// 触发陷阱
    pub fn trigger_trap(&mut self, trap: &mut Trap) -> Result<(), HeroError> {
        self.core.trigger_trap(trap)
    }

    /// 升级武器
    pub fn upgrade_weapon(&mut self) -> Result<(), HeroError> {
        self.core.upgrade_weapon()
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
