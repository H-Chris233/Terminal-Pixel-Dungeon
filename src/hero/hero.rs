use crate::combat::combat::Combat;
use crate::dungeon::dungeon::Dungeon;
use crate::items::potion::PotionKind;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;

use crate::hero::class::class::*;
use crate::items::items::{Item, ItemKind};

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
        // 先检查是否需要鉴定
        let needs_identify = match self.inventory.get(item_index) {
            Some(Item {
                kind: ItemKind::Potion(p),
                ..
            }) => !p.identified,
            _ => false,
        };

        if needs_identify {
            self.identify_item(item_index)?;
        }

        // 重新获取物品进行效果处理
        let item = match self.inventory.get(item_index) {
            Some(item) => item,
            None => return Err(anyhow::anyhow!("物品槽位 {} 为空", item_index)),
        };

        match &item.kind {
            ItemKind::Potion(potion) => {
                let potion_kind = potion.kind.clone();
                match potion_kind {
                    PotionKind::Healing => {
                        self.hp = (self.hp + 20).min(self.max_hp);
                        self.notify(format!("恢复20点生命值"));
                    } // 其他药水效果...
                    PotionKind::Strength => {
                        self.attack += 3;
                        self.notify(format!("攻击力提升3点"));
                    }
                    _ => todo!(), // 其他药水类型...
                }
            } // 其他物品类型处理...
            ItemKind::Weapon(weapon) => {
                self.notify(format!("装备了 {}", weapon.name));
                // 武器装备逻辑...
            }
            ItemKind::Armor(armor) => {
                self.notify(format!("穿上了 {}", armor.name));
                // 护甲装备逻辑...
            }
            ItemKind::Scroll(scroll) => {
                self.notify(format!("使用了 {} 卷轴", scroll.name));
                // 卷轴使用逻辑...
            }
            _ => todo!(),
        }

        // 消耗品移除逻辑保持不变
        if item.is_consumable() {
            self.inventory.remove(item_index);
        }

        Ok(())
    }

    /// 鉴定物品（保持原有参数不变）
    pub fn identify_item(&mut self, item_index: usize) -> anyhow::Result<()> {
        let potion_kind = {
            let item = self
                .inventory
                .get_mut(item_index)
                .ok_or_else(|| anyhow::anyhow!("物品不存在"))?;

            match &mut item.kind {
                ItemKind::Potion(potion) => {
                    potion.identified = true;
                    potion.kind.clone() // 先获取值再通知
                }
                _ => return Err(anyhow::anyhow!("非药水物品")),
            }
        };

        // 分离通知逻辑
        self.notify(format!("鉴定出药水: {:?}", potion_kind));
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
