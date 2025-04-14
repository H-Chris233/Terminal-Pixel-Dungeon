//src/hero/hero.rs
#![allow(dead_code)]
#![allow(unused)]

use bincode::{Decode, Encode};
use dungeon::Dungeon;
use items::potion::PotionKind;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;

use crate::class::*;
use combat::enemy::*;
use items::{Item, ItemKind};
use combat::Combat;

pub mod bag;
pub mod class;

/// 英雄角色数据结构
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Hero {
    // 基础属性
    pub class: Class,
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,

    // 成长系统
    pub experience: i32,
    pub level: i32,

    // 游戏进度
    pub gold: i32,
    pub x: i32,
    pub y: i32,
    pub alive: bool,
    pub inventory: Vec<Item>,
    pub start_time: u64,
    pub last_update: Option<SystemTime>,
    pub play_time: Duration,
}

impl Hero {
    /// 创建新英雄实例
    pub fn new(class: Class) -> Self {
        let mut hero = Self {
            class: class.clone(), // 显式克隆避免所有权问题
            hp: 0,
            max_hp: 0,
            attack: 0,
            defense: 0,
            experience: 0,
            level: 1,
            gold: 0,
            x: 0,
            y: 0,
            inventory: Vec::with_capacity(10), // 预分配10个物品槽位
            name: "Adventurer".to_string(),
            alive: true,
            start_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            last_update: Some(SystemTime::now()),
            play_time: Duration::from_secs(0),
        };

        // 根据职业初始化属性
        match hero.class {
            Class::Warrior => {
                hero.hp = 25;
                hero.max_hp = 25;
                hero.attack = 10;
                hero.defense = 5;
            }
            Class::Mage => {
                hero.hp = 15;
                hero.max_hp = 15;
                hero.attack = 12;
                hero.defense = 3;
            }
            Class::Rogue => {
                hero.hp = 20;
                hero.max_hp = 20;
                hero.attack = 8;
                hero.defense = 4;
            }
            Class::Huntress => {
                hero.hp = 18;
                hero.max_hp = 18;
                hero.attack = 9;
                hero.defense = 6;
            }
        }

        hero
    }

    /// 升级英雄属性
    pub fn level_up(&mut self) {
        // 确保每次升级都有提升
        self.max_hp += 5;
        self.hp = self.max_hp; // 升级时恢复满血
        self.attack += 2;
        self.defense += 1;
        self.level += 1;

        // 职业特定加成
        match self.class {
            Class::Warrior => self.max_hp += 2,   // 战士额外生命值
            Class::Mage => self.attack += 1,      // 法师额外攻击
            Class::Rogue => {}                    // 盗贼特殊能力在战斗逻辑中处理
            Class::Huntress => self.defense += 1, // 女猎手额外防御
        }
    }

    /// 移动英雄位置（带边界检查）
    pub fn move_to(&mut self, dx: i32, dy: i32, dungeon: &mut Dungeon) {
        let new_x = self.x.saturating_add(dx); // 使用饱和运算防止溢出
        let new_y = self.y.saturating_add(dy);

        // 检查移动是否有效
        if dungeon.current_level().is_passable(new_x, new_y) {
            self.x = new_x;
            self.y = new_y;
            self.explore_current_tile(dungeon);
        } else {
            self.notify(format!("无法到达！"));
        }
    }

    /// 探索当前位置
    fn explore_current_tile(&mut self, dungeon: &mut Dungeon) {
        let level = dungeon.current_level_mut();

        // 使用 enemy_at 获取可变引用
        if let Some(enemy) = level.enemy_at(self.x, self.y) {
            Combat::engage(self, enemy);
        }

        // 添加物品拾取逻辑（假设有 take_item 方法）
        if let Some(item) = level.take_item(self.x, self.y) {
            self.inventory.push(item.clone());
            self.notify(format!("拾取了物品: {}", item.name()));
        }
    }

    // 调整 use_item 方法避免借用冲突
    pub fn use_item(&mut self, item_index: usize) -> anyhow::Result<()> {
        // 检查物品是否存在
        let item = self
            .inventory
            .get(item_index)
            .ok_or_else(|| anyhow::anyhow!("物品槽位 {} 为空", item_index))?;

        // 如果需要鉴定，先进行鉴定
        if item.needs_identify() {
            self.identify_item(item_index)?;
        }

        // 重新获取物品引用（因为前面的借用已经结束）
        let item = self.inventory.get(item_index).unwrap(); // 安全：前面已检查存在

        match &item.kind {
            ItemKind::Potion(potion) => {
                self.use_potion(&potion.kind)?;
            }
            ItemKind::Weapon(weapon) => {
                self.equip_weapon(weapon)?;
            }
            ItemKind::Armor(armor) => {
                self.equip_armor(armor)?;
            }
            ItemKind::Scroll(scroll) => {
                self.use_scroll(scroll)?;
            }
            ItemKind::Ring(ring) => {
                self.equip_ring(ring)?;
            }
            _ => return Err(anyhow::anyhow!("无法使用此类型物品")),
        }

        // 如果是消耗品则移除
        if item.is_consumable() {
            self.inventory.remove(item_index);
        }

        Ok(())
    }

    /// 鉴定物品（支持多种可鉴定物品）
    pub fn identify_item(&mut self, item_index: usize) -> anyhow::Result<()> {
    let notification = {
        let item = self
            .inventory
            .get_mut(item_index)
            .ok_or_else(|| anyhow::anyhow!("物品槽位 {} 为空", item_index))?;

        match &mut item.kind {
            ItemKind::Potion(potion) => {
                potion.identified = true;
                format!("鉴定出药水: {}", potion.name())
            }
            ItemKind::Scroll(scroll) => {
                scroll.identified = true;
                format!("鉴定出卷轴: {}", scroll.name())
            }
            ItemKind::Weapon(weapon) => {
                weapon.identified = true;
                format!("鉴定出武器: {}", weapon.name)
            }
            ItemKind::Armor(armor) => {
                armor.identified = true;
                format!("鉴定出护甲: {}", armor.name())
            }
            _ => return Err(anyhow::anyhow!("该物品不需要鉴定")),
        }
    };

    self.notify(notification);
    Ok(())
}

    /// 获取经验值（带升级检查）
    pub fn gain_exp(&mut self, exp: i32) {
        self.experience += exp;

        // 简单升级公式：每100经验升1级
        while self.experience >= self.level * 100 {
            self.experience -= self.level * 100;
            self.level_up();
            self.notify(format!("升级到 {} 级！", self.level));
        }
    }

    /// 更新游戏时间（精确到毫秒）
    pub fn update_play_time(&mut self) {
        if let Some(last) = self.last_update {
            if let Ok(duration) = SystemTime::now().duration_since(last) {
                self.play_time += duration;
            }
        }
        self.last_update = Some(SystemTime::now());
    }

    /// 显示消息（保持原有简单实现）
    pub fn notify(&self, message: String) {
        println!("[英雄] {}", message);
    }
    pub fn get_start_instant(&self) -> Instant {
        Instant::now()
            - Duration::from_millis(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
                    - self.start_time,
            )
    }
}

impl Default for Hero {
    fn default() -> Self {
        Self::new(Class::default())
    }
}
