//src/hero/src/lib.rs
#![allow(dead_code)]
#![allow(unused)]

use bincode::{Decode, Encode};
use dungeon::Dungeon;
use items::potion::PotionKind;
use items::ItemCategory;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime};
use thiserror::Error;

use crate::bag::*;
use crate::class::*;
use combat::effect::Effect;
use combat::effect::EffectType;
use combat::enemy::*;
use combat::{Combat, Combatant};
use dungeon::trap::{Trap, TrapEffect, TrapKind};
use items::{Armor, Item, ItemKind, Ring, Weapon};

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
    pub base_attack: i32,  // 基础攻击力（不含装备加成）
    pub base_defense: i32, // 基础防御力（不含装备加成）

    // 成长系统
    pub experience: i32,
    pub level: i32,
    pub strength: u8, // 力量值（影响装备穿戴）

    // 游戏进度
    pub gold: u32, // 使用u32与Bag一致
    pub x: i32,
    pub y: i32,
    pub alive: bool,
    pub start_time: u64,
    pub last_update: Option<SystemTime>,
    pub play_time: Duration,
    pub bag: Bag,
    pub effect: Option<Vec<Effect>>,
}

#[derive(Debug, Error)]
pub enum HeroError {
    #[error("物品索引无效")]
    InvalidIndex,
    #[error("无法使用此物品")]
    UnusableItem,
    #[error("背包已满")]
    InventoryFull,
    #[error("力量不足")]
    Underpowered,
    #[error("鉴定失败")]
    IdentifyFailed,
    #[error(transparent)]
    BagError(#[from] BagError),
}

impl Hero {
    /// 创建新英雄实例（根据SPD逻辑调整初始属性）
    pub fn new(class: Class) -> Self {
        let mut hero = Self {
            class: class.clone(),
            hp: 0,
            max_hp: 0,
            base_attack: 0,
            base_defense: 0,
            experience: 0,
            level: 1,
            strength: 10, // 初始力量值
            gold: 0,
            x: 0,
            y: 0,
            name: "Adventurer".to_string(),
            alive: true,
            start_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            last_update: Some(SystemTime::now()),
            play_time: Duration::from_secs(0),
            bag: Bag::new(),
        };

        // 根据职业初始化属性
        match hero.class {
            Class::Warrior => {
                hero.hp = 25;
                hero.max_hp = 25;
                hero.base_attack = 10;
                hero.base_defense = 4;
                hero.strength += 1; // 战士额外力量
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

    /// 升级系统（根据SPD每级提升）
    pub fn level_up(&mut self) {
        self.level += 1;
        self.max_hp += self.class.hp_per_level();
        self.hp = self.max_hp; // 升级时恢复满血
        self.base_attack += self.class.attack_per_level();
        self.base_defense += self.class.defense_per_level();

        // 每4级增加1点力量
        if self.level % 4 == 0 {
            self.strength += 1;
        }

        // 战士系额外生命值
        if self.class == Class::Warrior {
            self.max_hp += 2;
            self.hp = self.max_hp;
        }
    }

    /// 移动英雄位置（带边界检查）
    pub fn move_to(&mut self, dx: i32, dy: i32, dungeon: &mut Dungeon) -> Result<(), String> {
        let new_x = self.x.saturating_add(dx);
        let new_y = self.y.saturating_add(dy);

        if !dungeon.current_level().in_bounds(new_x, new_y) {
            return Err("超出地图边界".to_string());
        }

        if !dungeon.current_level().is_passable(new_x, new_y) {
            return Err("路径被阻挡".to_string());
        }

        self.x = new_x;
        self.y = new_y;
        self.explore_current_tile(dungeon);
        Ok(())
    }

    /// 探索当前位置（SPD式探索机制）
    fn explore_current_tile(&mut self, dungeon: &mut Dungeon) {
        let level = dungeon.current_level_mut();

        // 敌人遭遇战
        if let Some(enemy) = level.enemy_at(self.x, self.y) {
            Combat::engage(self, enemy);
        }

        // 物品拾取
        if let Some(item) = level.take_item(self.x, self.y) {
            match self.bag.add_item(item) {
                Ok(_) => self.notify(format!("拾取了: {}", item.name())),
                Err(BagError::InventoryFull) => {
                    level.drop_item(self.x, self.y, item); // 放回物品
                    self.notify("背包已满，无法拾取".into());
                }
                _ => {}
            }
        }

        // 陷阱检测（需要灵视效果）
        if level.has_trap(self.x, self.y) && !self.has_effect(EffectType::MindVision) {
            self.trigger_trap(level.get_trap(self.x, self.y).unwrap());
        }
    }

    /// 使用物品（分类处理）
    pub fn use_item(&mut self, category: ItemCategory, index: usize) -> Result<(), HeroError> {
        match category {
            ItemCategory::Potion => self.use_potion(index),
            ItemCategory::Scroll => self.use_scroll(index),
            ItemCategory::Weapon => self.equip_weapon(index),
            ItemCategory::Armor => self.equip_armor(index),
            ItemCategory::Ring => self.equip_ring(index),
            _ => Err(HeroError::UnusableItem),
        }
    }

    /// 药水使用逻辑
    fn use_potion(&mut self, index: usize) -> Result<(), HeroError> {
        let potion = self
            .bag
            .potions()
            .get(index)
            .ok_or(HeroError::InvalidIndex)?;

        // 未鉴定药水有风险
        if !potion.identified {
            self.notify("你喝下了未知的药水...".into());
            // 50%几率负面效果
            if rand::thread_rng().gen_bool(0.5) {
                return Err(HeroError::IdentifyFailed);
            }
        }

        match potion.kind {
            PotionKind::Healing => self.heal(self.max_hp / 3),
            PotionKind::Strength => self.strength += 1,
            PotionKind::MindVision => self.add_effect(EffectType::MindVision, 20),
            // 其他药水效果...
        }

        self.bag.remove_item(index)?;
        Ok(())
    }

    /// 使用卷轴
    fn use_scroll(&mut self, index: usize) -> Result<(), HeroError> {
        // 实现卷轴使用逻辑
        Ok(())
    }

    /// 装备武器（带力量检查）
    fn equip_weapon(&mut self, index: usize) -> Result<(), HeroError> {
        let weapon = self
            .bag
            .weapons()
            .get(index)
            .ok_or(HeroError::InvalidIndex)?;

        if weapon.str_requirement > self.strength {
            return Err(HeroError::Underpowered);
        }

        let old_weapon = self.bag.equip_item(index, self.strength)?;
        if let Some(w) = old_weapon {
            self.bag
                .add_item(w.into())
                .map_err(|_| HeroError::InventoryFull)?;
        }

        Ok(())
    }

    /// 装备护甲
    fn equip_armor(&mut self, index: usize) -> Result<(), HeroError> {
        let armor = self
            .bag
            .armors()
            .get(index)
            .ok_or(HeroError::InvalidIndex)?;

        if armor.str_requirement > self.strength as u8 {
            return Err(HeroError::Underpowered);
        }

        let old_armor = self.bag.equip_item(index, self.strength)?;
        if let Some(a) = old_armor {
            self.bag
                .add_item(a.into())
                .map_err(|_| HeroError::InventoryFull)?;
        }

        Ok(())
    }

    /// 装备戒指
    fn equip_ring(&mut self, index: usize) -> Result<(), HeroError> {
        let ring = self.bag.rings().get(index).ok_or(HeroError::InvalidIndex)?;

        let old_ring = self.bag.equip_item(index, self.strength)?;
        if let Some(r) = old_ring {
            self.bag
                .add_item(r.into())
                .map_err(|_| HeroError::InventoryFull)?;
        }

        Ok(())
    }

    /// 获取经验值（带升级检查）
    pub fn gain_exp(&mut self, exp: i32) {
        self.experience += exp;

        // SPD式升级公式
        while self.experience >= self.level * 100 {
            self.experience -= self.level * 100;
            self.level_up();
            self.notify(format!("升级到 {} 级！", self.level));
        }
    }

    /// 更新游戏时间
    pub fn update_play_time(&mut self) {
        if let Some(last) = self.last_update {
            if let Ok(duration) = SystemTime::now().duration_since(last) {
                self.play_time += duration;
            }
        }
        self.last_update = Some(SystemTime::now());
    }

    /// 显示消息
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

    pub fn trigger_trap(&mut self, trap: &mut Trap) -> Result<(), String> {
        // 检查陷阱是否可触发
        if !trap.is_active() {
            return Err("陷阱已失效".to_string());
        }

        if trap.is_triggered() {
            return Err("陷阱已被触发过".to_string());
        }

        // 触发陷阱并获取效果
        let effect = trap.trigger();

        // 根据不同类型处理效果
        match effect {
            TrapEffect::Damage(amount) => {
                self.take_damage(amount);
                Ok(())
            }
            TrapEffect::Poison(damage, duration) => {
                self.apply_poison(damage, duration);
                Ok(())
            }
            TrapEffect::Alarm => {
                self.alert_nearby_enemies();
                Ok(())
            }
            TrapEffect::Teleport => self
                .random_teleport()
                .map_err(|e| format!("传送失败: {}", e)),
            TrapEffect::Paralyze(duration) => {
                self.apply_paralysis(duration);
                Ok(())
            }
            TrapEffect::Summon => self
                .summon_enemies_around(trap.position())
                .map_err(|e| format!("召唤失败: {}", e)),
            TrapEffect::Fire(damage) => {
                self.set_on_fire(damage);
                Ok(())
            }
            TrapEffect::Pitfall => self
                .fall_to_lower_level()
                .map_err(|e| format!("掉落失败: {}", e)),
            TrapEffect::Grip(duration) => {
                self.apply_gripping(duration);
                Ok(())
            }
            TrapEffect::DisarmOtherTraps => {
                self.disarm_all_traps_on_level();
                Ok(())
            }
        }
    }

    pub fn armor(&self) -> Option<&Armor> {
        self.bag.equipment().armor()
    }

    pub fn rings(&self) -> Vec<&Ring> {
        self.bag.equipment().rings()
    }
    /// 中毒效果
    fn apply_poison(&mut self, damage: i32, duration: i32) {
        self.add_effect(
            Effect::new(
                EffectType::Poison,
                duration,
                Some(damage),
                "你中毒了！".to_string(),
            )
        );
    }

    /// 警报附近敌人
    fn alert_nearby_enemies(&mut self) {
        // 在实际游戏中，这里应该调用Dungeon的方法来警报敌人
        self.notify("刺耳的警报声响起！附近的敌人都被吸引了！".to_string());
    }

    /// 随机传送
    fn random_teleport(&mut self) -> Result<(), String> {
        // 在实际游戏中，这里应该调用Dungeon的方法寻找随机安全位置
        self.notify("你被神秘力量传送了！".to_string());
        Ok(())
    }

    /// 麻痹效果
    fn apply_paralysis(&mut self, duration: i32) {
        self.add_effect(
            Effect::new(
                EffectType::Paralysis,
                duration,
                None,
                "你被麻痹了，无法移动！".to_string(),
            )
        );
    }

    /// 在指定位置周围召唤敌人
    fn summon_enemies_around(&mut self, pos: (i32, i32)) -> Result<(), String> {
        // 在实际游戏中，这里应该调用Dungeon的方法生成敌人
        self.notify("周围出现了敌对的召唤生物！".to_string());
        Ok(())
    }

    /// 点燃效果
    fn set_on_fire(&mut self, damage: i32) {
        self.add_effect(
            Effect::new(
                EffectType::Burning,
                6, // 默认燃烧3回合
                Some(damage / 3), // 每回合伤害
                "你被点燃了！".to_string(),
            )
        );
        self.take_damage(damage);
    }

    /// 掉落至下层
    fn fall_to_lower_level(&mut self) -> Result<(), String> {
        // 在实际游戏中，这里应该调用Dungeon的方法下降一层
        self.notify("你掉入了陷坑，落到了下层！".to_string());
        self.take_damage((self.max_hp / 5).max(1)); // 掉落伤害
        Ok(())
    }

    /// 束缚效果
    fn apply_gripping(&mut self, duration: i32) {
        self.add_effect(
            Effect::new(
                EffectType::Rooted,
                duration,
                None,
                "你被粘性物质束缚住了！".to_string(),
            )
        );
    }

    /// 解除本层所有陷阱
    fn disarm_all_traps_on_level(&mut self) {
        // 在实际游戏中，这里应该调用Dungeon的方法解除所有陷阱
        self.notify("一道能量波解除了本层所有陷阱！".to_string());
    }
    /// 添加新的效果到英雄身上
    pub fn add_effect(&mut self, effect: Effect) {
        if let Some(ref mut effects) = self.effect {
            // 检查是否已有相同类型的效果
            if let Some(existing) = effects.iter_mut().find(|e| e.effect_type == effect.effect_type) {
                // 更新持续时间（取最大值）
                existing.duration = existing.duration.max(effect.duration);
                // 更新伤害值（如果有）
                if let Some(dmg) = effect.damage {
                    existing.damage = Some(existing.damage.unwrap_or(0).max(dmg));
                }
            } else {
                effects.push(effect);
            }
        } else {
            self.effect = Some(vec![effect]);
        }
    }

    /// 移除指定类型的效果
    pub fn remove_effect(&mut self, effect_type: EffectType) {
        if let Some(ref mut effects) = self.effect {
            effects.retain(|e| e.effect_type != effect_type);
            if effects.is_empty() {
                self.effect = None;
            }
        }
    }

    /// 检查是否有指定类型的效果
    pub fn has_effect(&self, effect_type: EffectType) -> bool {
        self.effect.as_ref().map_or(false, |effects| 
            effects.iter().any(|e| e.effect_type == effect_type)
        )
    }

    /// 更新所有效果（每回合调用）
    pub fn update_effects(&mut self) {
        if let Some(ref mut effects) = self.effect {
            // 处理每个效果的持续时间和伤害
            for effect in effects.iter_mut() {
                effect.duration -= 1;
                
                // 应用效果伤害
                if let Some(damage) = effect.damage {
                    self.take_damage(damage);
                }
            }

            // 移除已过期的效果
            effects.retain(|e| e.duration > 0);
            
            if effects.is_empty() {
                self.effect = None;
            }
        }
    }

    /// 获取特定类型的效果（如果有）
    pub fn get_effect(&self, effect_type: EffectType) -> Option<&Effect> {
        self.effect.as_ref().and_then(|effects| 
            effects.iter().find(|e| e.effect_type() == effect_type)
        )
    }

    /// 清除所有效果
    pub fn clear_effects(&mut self) {
        self.effect = None;
    }

    /// 检查英雄是否被麻痹/束缚等无法移动的效果影响
    pub fn is_immobilized(&self) -> bool {
        self.has_effect(EffectType::Paralysis) || 
        self.has_effect(EffectType::Rooted)
    }

    /// 检查英雄是否有视觉增强效果（如灵视）
    pub fn has_vision_enhancement(&self) -> bool {
        self.has_effect(EffectType::MindVision)
    }
}

impl Default for Hero {
    fn default() -> Self {
        Self::new(Class::default())
    }
}

// 战斗系统实现（根据SPD战斗公式）
impl Combatant for Hero {
    fn hp(&self) -> i32 {
        self.hp
    }

    fn max_hp(&self) -> i32 {
        self.max_hp
    }

    fn attack_power(&self) -> i32 {
        let weapon_bonus = self
            .bag
            .equipment()
            .weapon
            .as_ref()
            .map_or(0, |w| w.damage_bonus() as i32);

        (self.base_attack + weapon_bonus) * (100 + self.level) / 100 // 等级加成
    }

    fn defense(&self) -> i32 {
        let armor_bonus = self
            .bag
            .equipment()
            .armor
            .as_ref()
            .map_or(0, |a| a.defense() as i32);

        self.base_defense + armor_bonus
    }

    fn accuracy(&self) -> i32 {
        let weapon_bonus = self
            .bag
            .equipment()
            .weapon
            .as_ref()
            .map_or(0, |w| w.accuracy_bonus() as i32);

        80 + (self.level * 2) + weapon_bonus // SPD基础精度
    }

    fn evasion(&self) -> i32 {
        let armor_penalty = self
            .bag
            .equipment()
            .armor()
            .map_or(0, |a| a.evasion_penalty());

        (self.level * 3) - armor_penalty // 护甲降低闪避
    }

    fn crit_bonus(&self) -> f32 {
        let class_bonus = match self.class {
            Class::Warrior => 0.05,
            Class::Mage => 0.0,
            Class::Rogue => 0.15,
            Class::Huntress => 0.07,
        };

        let weapon_bonus = self.weapon().map_or(0.0, |w| w.crit_bonus());

        0.1 + class_bonus + weapon_bonus
    }

    fn weapon(&self) -> Option<&Weapon> {
        self.bag.equipment().weapon()
    }

    fn is_alive(&self) -> bool {
        self.alive && self.hp > 0
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn attack_distance(&self) -> i32 {
        self.weapon().map_or(1, |w| w.range() as i32)
    }

    fn take_damage(&mut self, amount: i32) -> bool {
        let actual_damage = (amount - self.defense()).max(1);
        self.hp = (self.hp - actual_damage).max(0);
        self.alive = self.hp > 0;

        if !self.alive {
            self.notify("你死了...".into());
        }
        self.is_alive()
    }

    fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }
}
