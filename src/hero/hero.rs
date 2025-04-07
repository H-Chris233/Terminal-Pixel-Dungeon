use crate::dungeon::dungeon::Dungeon;
use crate::combat::combat::Combat;
use crate::items::items::*;
use std::time::SystemTime;

// src/hero.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub enum Class {
    #[default]
    Warrior,  // 战士
    Mage,     // 法师
    Rogue,    // 盗贼
    Huntress, // 女猎手
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Hero {
    pub class: Class,
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub experience: i32,
    pub level: i32,
    pub gold: i32,
    pub x: i32,  // 添加位置信息
    pub y: i32,
    pub alive: bool,
    pub inventory: Vec<Item>,  // 物品栏
    pub name: String,
    pub play_time: f64, // 累计游戏时间(秒)
    
    #[serde(skip)] // 不序列化临时状态
    pub last_update: Option<SystemTime>,
    pub play_time: f64,
}

impl Hero {
    pub fn new(class: Class) -> Self {
        let mut hero = Self {
            class: class.clone(),
            hp: 0,
            max_hp: 0,
            attack: 0,
            defense: 0,
            experience: 0,
            level: 1,
            gold: 0,
            x: 0,
            y: 0,
            inventory: Vec::new(),
            name: "Adventurer".to_string(),
            alive: true,
            play_time: 0.0,
            last_update: Some(SystemTime::now()),
            // 其他字段...
        };
        
        match class {
            Class::Warrior => {
                hero.hp = 25;
                hero.max_hp = 25;
                hero.attack = 10;
                hero.defense = 5;
            },
            // 其他职业初始化...
            _ => {}
        }
        
        hero
    }
    
    /// 升级英雄
    pub fn level_up(&mut self) {
        self.max_hp += 5;
        self.hp = self.max_hp;
        self.attack += 2;
        self.defense += 1;
        self.level += 1;
    }
    
    pub fn move_to(&mut self, dx: i32, dy: i32, dungeon: &mut Dungeon) {
        let new_x = self.x + dx;
        let new_y = self.y + dy;
        
        // 检查是否可以移动
        if dungeon.current_level().is_passable(new_x, new_y) {
            self.x = new_x;
            self.y = new_y;
            self.explore_current_tile(dungeon);
        }
    }
    
    fn explore_current_tile(&mut self, dungeon: &mut Dungeon) {
        let level = dungeon.current_level_mut();
        
        // 检查是否有敌人
        if let Some(enemy) = level.enemy_at(self.x, self.y) {
            Combat::engage(self, enemy);
        }
        
        // 检查是否有物品
        if let Some(item) = level.item_at(self.x, self.y) {
            self.notify(format!("你发现了: {}", item.name()));
        }
    }
    pub fn use_item(&mut self, item_index: usize) -> anyhow::Result<()> {
        if let Some(item) = self.inventory.get_mut(item_index) {
            match item.kind {
                ItemKind::Potion(ref potion) => {
                    if !potion.identified {
                        self.identify_item(item);
                    }
                    match potion.kind {
                        PotionKind::Healing => self.hp = self.max_hp,
                        PotionKind::Strength => self.attack += 1,
                        // 其他药水效果...
                    }
                    self.inventory.remove(item_index);
                }
                ItemKind::Weapon(_) => todo!("处理武器"),
                ItemKind::Armor(_) => todo!("处理护甲"),
                ItemKind::Scroll(_) => todo!("处理卷轴")
                // 其他物品类型处理...
            }
        }
        Ok(())
    }
    
    pub fn notify(&self, message: String) {
        println!("{}", message);
    }
    
    pub fn gain_exp(&mut self, exp: i32) {
        self.experience += exp;
        // 检查是否升级
    }
    pub fn identify_item(&mut self, item: &mut Item) {
        // 实现物品鉴定逻辑
        match &mut item.kind {
            ItemKind::Potion(potion) => {
                // 处理药水
            },
            ItemKind::Weapon(_) => todo!("处理武器"),
            ItemKind::Armor(_) => todo!("处理护甲"),
            ItemKind::Scroll(_) => todo!("处理卷轴"),
        }
    }
    pub fn update_play_time(&mut self) {
        let now = SystemTime::now();
        if let Some(last) = self.last_update {
            if let Ok(duration) = now.duration_since(last) {
                self.play_time += duration.as_secs_f64();
            }
        }
        self.last_update = Some(now);
    }
}


impl Default for Hero {
    fn default() -> Self {
        Self {
            class: class.clone(),
            hp: 0,
            max_hp: 0,
            attack: 0,
            defense: 0,
            experience: 0,
            level: 1,
            gold: 0,
            x: 0,
            y: 0,
            inventory: Vec::new(),
            name: "Adventurer".to_string(),
            alive: true,
            play_time: 0.0,
            last_update: Some(SystemTime::now()),
            // 其他字段...
        }
    }
}

