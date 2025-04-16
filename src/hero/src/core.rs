
// src/hero/core.rs
use crate::{
    bag::Bag,
    class::Class,
    effects::{Effect, EffectManager, EffectType},
    rng::HeroRng,
};
use combat::{Combatant, Trap};
use dungeon::Dungeon;
use items::{ItemCategory, potion::PotionKind};
use thiserror::Error;

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
    #[error("饥饿过度")]
    Starvation,
    #[error(transparent)]
    BagError(#[from] crate::bag::BagError),
}

/// 英雄核心数据结构
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
    pub bag: Bag,
    pub effects: EffectManager,
    pub rng: HeroRng,
}

impl Hero {
    /// 创建新英雄（随机种子）
    pub fn new(class: Class) -> Self {
        Self::with_seed(class, rand::random())
    }

    /// 使用指定种子创建英雄
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
            bag: Bag::new(),
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

    /// 每回合更新状态
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
        self.effects.update(self);

        Ok(())
    }

    /// 升级系统
    pub fn level_up(&mut self) {
        self.level += 1;
        self.max_hp += self.class.hp_per_level();
        self.hp = self.max_hp;
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

    /// 移动英雄位置
    pub fn move_to(&mut self, dx: i32, dy: i32, dungeon: &mut Dungeon) -> Result<(), String> {
        if self.is_immobilized() {
            return Err("你被控制效果影响，无法移动".into());
        }

        let new_x = self.x.saturating_add(dx);
        let new_y = self.y.saturating_add(dy);

        if !dungeon.current_level().in_bounds(new_x, new_y) {
            return Err("超出地图边界".into());
        }

        if !dungeon.current_level().is_passable(new_x, new_y) {
            return Err("路径被阻挡".into());
        }

        self.x = new_x;
        self.y = new_y;
        self.explore_current_tile(dungeon);
        Ok(())
    }

    /// 探索当前位置
    fn explore_current_tile(&mut self, dungeon: &mut Dungeon) {
        let level = dungeon.current_level_mut();

        // 敌人遭遇战
        if let Some(enemy) = level.enemy_at(self.x, self.y) {
            combat::engage(self, enemy);
        }

        // 物品拾取
        if let Some(item) = level.take_item(self.x, self.y) {
            match self.bag.add_item(item) {
                Ok(_) => self.notify(format!("拾取了: {}", item.name())),
                Err(crate::bag::BagError::InventoryFull) => {
                    level.drop_item(self.x, self.y, item);
                    self.notify("背包已满，无法拾取".into());
                }
                _ => {}
            }
        }

        // 陷阱检测
        if level.has_trap(self.x, self.y) && !self.has_vision_enhancement() {
            if let Some(trap) = level.get_trap(self.x, self.y) {
                self.trigger_trap(trap).unwrap_or_else(|e| self.notify(e));
            }
        }
    }

    /// 使用物品
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

    /// 获取经验值
    pub fn gain_exp(&mut self, exp: u32) {
        self.experience += exp;

        while self.experience >= self.level * 100 {
            self.experience -= self.level * 100;
            self.level_up();
            self.notify(format!("升级到 {} 级！", self.level));
        }
    }

    /// 显示消息
    pub fn notify(&self, message: String) {
        println!("[英雄] {}", message);
    }
    
    /// 药水使用逻辑
    fn use_potion(&mut self, index: usize) -> Result<(), HeroError> {
        let potion = self.bag.potions().get(index).ok_or(HeroError::InvalidIndex)?;

        if !potion.identified {
            self.notify("你喝下了未知的药水...".into());
            if self.rng.gen_bool(0.5) {
                return Err(HeroError::IdentifyFailed);
            }
        }

        match potion.kind {
            PotionKind::Healing => self.heal(self.max_hp / 3),
            PotionKind::Strength => self.strength += 1,
            PotionKind::MindVision => self.effects.add(
                Effect::new(EffectType::MindVision, 20, "获得灵视效果".into())
            ),
            PotionKind::ToxicGas => {
                // 对周围敌人造成中毒效果
                dungeon::affect_adjacent_enemies(self.x, self.y, |e| {
                    e.add_effect(Effect::poison(2, 10));
                });
            },
            PotionKind::Frost => {
                // 冰冻周围敌人
                dungeon::affect_adjacent_enemies(self.x, self.y, |e| {
                    e.add_effect(Effect::new(EffectType::Frozen, 5, "被冰冻".into()));
                });
            }
        }

        self.bag.remove_potion(index)?;
        Ok(())
    }

    /// 使用卷轴
    fn use_scroll(&mut self, index: usize) -> Result<(), HeroError> {
        let scroll = self.bag.scrolls().get(index).ok_or(HeroError::InvalidIndex)?;
        
        if !scroll.identified {
            self.notify("你阅读了未知的卷轴...".into());
            if self.rng.gen_bool(0.5) {
                scroll.identify(&mut self.rng);
            } else {
                return Err(HeroError::IdentifyFailed);
            }
        }

        match scroll.kind {
            ScrollKind::Upgrade => {
                if let Some(weapon) = self.bag.equipment().weapon.as_mut() {
                    weapon.upgrade();
                    self.notify(format!("你的{}变得更锋利了！", weapon.name));
                } else {
                    return Err(HeroError::UnusableItem);
                }
            },
            ScrollKind::RemoveCurse => {
                self.bag.remove_cursed_items();
                self.notify("一股净化之力扫过你的装备".into());
            },
            ScrollKind::MagicMapping => {
                dungeon::reveal_current_level(self.x, self.y);
                self.notify("你的脑海中浮现出这一层的地图".into());
            }
        }

        self.bag.remove_scroll(index)?;
        Ok(())
    }

    /// 装备武器
    fn equip_weapon(&mut self, index: usize) -> Result<(), HeroError> {
        let weapon = self.bag.weapons().get(index).ok_or(HeroError::InvalidIndex)?;

        if weapon.str_requirement > self.strength {
            return Err(HeroError::Underpowered);
        }

        let old_weapon = self.bag.equip_weapon(index)?;
        if let Some(w) = old_weapon {
            self.bag.add_weapon(w).map_err(|_| HeroError::InventoryFull)?;
        }

        Ok(())
    }

    /// 装备护甲
    fn equip_armor(&mut self, index: usize) -> Result<(), HeroError> {
        let armor = self.bag.armors().get(index).ok_or(HeroError::InvalidIndex)?;

        if armor.str_requirement > self.strength {
            return Err(HeroError::Underpowered);
        }

        let old_armor = self.bag.equip_armor(index)?;
        if let Some(a) = old_armor {
            self.bag.add_armor(a).map_err(|_| HeroError::InventoryFull)?;
        }

        Ok(())
    }

    /// 装备戒指
    fn equip_ring(&mut self, index: usize) -> Result<(), HeroError> {
        let ring = self.bag.rings().get(index).ok_or(HeroError::InvalidIndex)?;
        
        if self.bag.equipment().rings.iter().filter(|r| r.is_some()).count() >= 2 {
            return Err(HeroError::InventoryFull); // 戒指槽已满
        }

        let old_ring = self.bag.equip_ring(index)?;
        if let Some(r) = old_ring {
            self.bag.add_ring(r).map_err(|_| HeroError::InventoryFull)?;
        }

        Ok(())
    }

    /// 触发陷阱
    pub fn trigger_trap(&mut self, trap: &mut Trap) -> Result<(), String> {
        if !trap.is_active() {
            return Err("陷阱已失效".into());
        }

        let effect = trap.trigger(&mut self.rng);
        match effect {
            TrapEffect::Damage(amount) => {
                self.take_damage(amount);
                self.notify(format!("受到{}点伤害！", amount));
            },
            TrapEffect::Poison(dmg, turns) => {
                self.effects.add(Effect::poison(dmg, turns));
                self.notify("你中毒了！".into());
            },
            TrapEffect::Alarm => {
                dungeon::alert_nearby_enemies(self.x, self.y);
                self.notify("警报响起！敌人被吸引过来了！".into());
            },
            TrapEffect::Teleport => {
                dungeon::random_teleport(self);
                self.notify("你被随机传送了！".into());
            },
            TrapEffect::Paralyze(duration) => {
                self.effects.add(Effect::new(
                    EffectType::Paralysis, 
                    duration,
                    "你被麻痹了，无法移动！".into()
                ));
            },
            _ => {}
        }
        
        Ok(())
    }

    /// 受到伤害
    pub fn take_damage(&mut self, amount: u32) -> bool {
        // 计算实际伤害（考虑防御随机性）
        let defense_roll = self.defense() as f32 * (0.7 + self.rng.gen_range(0.0..0.6));
        let actual_damage = (amount as f32 - defense_roll).max(1.0) as u32;
        
        self.hp = self.hp.saturating_sub(actual_damage);
        self.alive = self.hp > 0;

        if !self.alive {
            self.notify("你死了...".into());
        }
        self.alive
    }

    /// 治疗
    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
        self.notify(format!("恢复了{}点生命值", amount));
    }

    /// 获取当前防御力
    pub fn defense(&self) -> u32 {
        self.base_defense + self.bag.equipment().armor_defense()
    }

    /// 检查是否被控制效果影响
    pub fn is_immobilized(&self) -> bool {
        self.effects.has(EffectType::Paralysis) || 
        self.effects.has(EffectType::Rooted)
    }

    /// 检查是否有视觉增强
    pub fn has_vision_enhancement(&self) -> bool {
        self.effects.has(EffectType::MindVision)
    }

    /// 重置RNG状态
    pub fn reset_rng(&mut self) {
        self.rng.reset();
    }

    /// 获取当前种子
    pub fn seed(&self) -> u64 {
        self.rng.seed()
    }

    /// 重新设定种子
    pub fn reseed(&mut self, new_seed: u64) {
        self.rng.reseed(new_seed);
    }
}

impl Default for Hero {
    fn default() -> Self {
        Self::new(Class::default())
    }
}

// 实现Combatant trait
impl Combatant for Hero {
    fn hp(&self) -> u32 {
        self.hp
    }

    fn max_hp(&self) -> u32 {
        self.max_hp
    }

    fn attack_power(&self) -> u32 {
        let weapon_bonus = self.bag.equipment()
            .weapon
            .as_ref()
            .map_or(0, |w| w.damage_bonus() as u32);
        
        (self.base_attack + weapon_bonus) * (100 + self.level) / 100
    }

    fn defense(&self) -> u32 {
        self.defense()
    }

    fn take_damage(&mut self, amount: u32) -> bool {
        self.take_damage(amount)
    }

    fn heal(&mut self, amount: u32) {
        self.heal(amount)
    }

    fn is_alive(&self) -> bool {
        self.alive
    }

    fn name(&self) -> &str {
        &self.name
    }
}
