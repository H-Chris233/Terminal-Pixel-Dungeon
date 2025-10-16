// src/combat/src/enemy/enemy.rs

use bincode::{Decode, Encode};
use rand::Rng;
use rand::prelude::IndexedRandom;
use serde::{Deserialize, Serialize};

use crate::effect::Effect;
use items::Weapon;

/// 敌人实体，包含战斗属性和位置信息
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Enemy {
    pub effects: Vec<Effect>,
    pub kind: EnemyKind,
    pub hp: u32,
    pub max_hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub exp_value: u32,
    pub x: i32,
    pub y: i32,
    pub state: EnemyState,
    pub attack_range: u32,
    pub detection_range: u32,
    pub symbol: char,
    pub color: (u8, u8, u8),
    pub is_surprised: bool,
    pub weapon: Option<Weapon>,
    pub crit_bonus: f32,
    pub entity_id: Option<u32>, // 添加entity_id用于事件总线
}

impl Enemy {
    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}

/// 敌人种类，影响基础属性和行为
#[derive(Clone, Debug, Default, Encode, Decode, Serialize, Deserialize, PartialEq)]
pub enum EnemyKind {
    #[default]
    Rat,
    Snake,
    Gnoll,
    Crab,
    Bat,
    Scorpion,
    Guard,
    Warlock,
    Golem,
}

/// 敌人当前状态
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize, PartialEq)]
pub enum EnemyState {
    Idle,
    Alert,
    Hostile,
    Fleeing,
    Sleeping,
    Passive,
}

/// 掉落物品类型
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub enum DropItem {
    Gold(u32),
    HealthPotion,
    Weapon(String), // 武器类型
    Armor(String),  // 护甲类型
    Scroll(String), // 卷轴类型
    Key,            // 钥匙
    Artifact,       // 神器
}

impl Enemy {
    /// 创建新敌人实例，属性根据破碎的像素地牢原作平衡
    pub fn new(kind: EnemyKind, x: i32, y: i32) -> Self {
        let (hp, attack, defense, exp_value, range, detection, symbol, color) = match kind {
            EnemyKind::Rat => (10, 4, 2, 2, 1, 5, 'r', (255, 150, 150)),
            EnemyKind::Snake => (12, 6, 3, 4, 1, 6, 's', (150, 255, 150)),
            EnemyKind::Gnoll => (20, 8, 5, 6, 1, 6, 'g', (200, 200, 100)),
            EnemyKind::Crab => (25, 5, 10, 5, 1, 4, 'c', (255, 100, 100)),
            EnemyKind::Bat => (15, 10, 4, 3, 1, 8, 'b', (200, 150, 255)),
            EnemyKind::Scorpion => (22, 12, 8, 8, 1, 5, 'S', (255, 100, 0)),
            EnemyKind::Guard => (30, 12, 10, 10, 1, 7, 'G', (100, 100, 255)),
            EnemyKind::Warlock => (18, 15, 5, 12, 3, 8, 'W', (255, 0, 255)),
            EnemyKind::Golem => (50, 18, 15, 15, 1, 4, 'M', (150, 150, 150)),
        };

        Self {
            kind,
            hp,
            max_hp: hp,
            attack,
            defense,
            exp_value,
            x,
            y,
            state: EnemyState::Idle,
            attack_range: range,
            detection_range: detection,
            symbol,
            color,
            is_surprised: false,
            weapon: None,
            crit_bonus: 0.0,
            effects: Vec::new(),
            entity_id: None, // 添加entity_id字段
        }
    }

    /// 受到伤害
    pub fn take_damage(&mut self, amount: u32) -> bool {
        let actual_damage = (amount - self.defense).max(0);
        self.hp = (self.hp - actual_damage).max(0);
        self.is_alive()
    }

    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// 重置敌人状态
    pub fn reset(&mut self) {
        self.state = EnemyState::Idle;
        self.is_surprised = false;
    }

    /// 状态转换方法
    pub fn set_state(&mut self, new_state: EnemyState) {
        if new_state == EnemyState::Hostile {
            self.is_surprised = true;
        }
        self.state = new_state;
    }

    /// 转换为敌对状态
    pub fn make_hostile(&mut self) {
        self.set_state(EnemyState::Hostile);
    }

    /// 转换为逃跑状态
    pub fn start_fleeing(&mut self) {
        self.set_state(EnemyState::Fleeing);
    }

    /// 转换为警戒状态
    pub fn alert(&mut self) {
        self.set_state(EnemyState::Alert);
    }

    /// 使用改进的寻路算法计算下一步移动方向
    pub fn calculate_move(
        &self,
        target_x: i32,
        target_y: i32,
        obstacles: &[(i32, i32)],
    ) -> Option<(i32, i32)> {
        // 简单实现：优先减少x或y距离
        let dx = (target_x - self.x).signum();
        let dy = (target_y - self.y).signum();

        // 检查移动是否有效(不穿过障碍物)
        let new_x = self.x + dx;
        let new_y = self.y + dy;

        if !obstacles.contains(&(new_x, new_y)) {
            Some((dx, dy))
        } else {
            // 尝试只移动x或y方向
            if !obstacles.contains(&(self.x + dx, self.y)) {
                Some((dx, 0))
            } else if !obstacles.contains(&(self.x, self.y + dy)) {
                Some((0, dy))
            } else {
                // 随机选择一个方向尝试
                let mut rng = rand::rng();
                let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
                for dir in directions.choose_multiple(&mut rng, 4) {
                    let test_x = self.x + dir.0;
                    let test_y = self.y + dir.1;
                    if !obstacles.contains(&(test_x, test_y)) {
                        return Some(*dir);
                    }
                }
                None
            }
        }
    }

    /// 执行移动
    pub fn perform_move(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
        self.is_surprised = false; // 移动后不再处于惊讶状态
    }

    /// 敌人死亡时掉落物品
    pub fn drop_items(&self) -> Vec<DropItem> {
        let mut drops = Vec::new();
        let mut rng = rand::rng();

        // 基础掉落：金币
        let gold_amount = match self.kind {
            EnemyKind::Rat => rng.random_range(1..5),
            EnemyKind::Snake => rng.random_range(2..6),
            EnemyKind::Gnoll => rng.random_range(3..8),
            EnemyKind::Crab => rng.random_range(2..7),
            EnemyKind::Bat => rng.random_range(1..4),
            EnemyKind::Scorpion => rng.random_range(4..10),
            EnemyKind::Guard => rng.random_range(5..12),
            EnemyKind::Warlock => rng.random_range(8..15),
            EnemyKind::Golem => rng.random_range(10..20),
        };

        drops.push(DropItem::Gold(gold_amount));

        // 稀有掉落
        let rare_drop_chance = match self.kind {
            EnemyKind::Rat => 0.05,
            EnemyKind::Snake => 0.08,
            EnemyKind::Gnoll => 0.1,
            EnemyKind::Crab => 0.07,
            EnemyKind::Bat => 0.15,
            EnemyKind::Scorpion => 0.12,
            EnemyKind::Guard => 0.2,
            EnemyKind::Warlock => 0.25,
            EnemyKind::Golem => 0.3,
        };

        if rng.random::<f32>() < rare_drop_chance {
            drops.push(match rng.random_range(0..6) {
                0 => DropItem::HealthPotion,
                1 => DropItem::Weapon("Dagger".to_string()),
                2 => DropItem::Armor("Leather".to_string()),
                3 => DropItem::Scroll("Identify".to_string()),
                4 => DropItem::Key,
                _ => DropItem::Gold(5),
            });
        }

        // 特殊敌人掉落
        match self.kind {
            EnemyKind::Guard => {
                if rng.random::<f32>() < 0.1 {
                    drops.push(DropItem::Key);
                }
            }
            EnemyKind::Warlock => {
                if rng.random::<f32>() < 0.15 {
                    drops.push(DropItem::Scroll("Magic Mapping".to_string()));
                }
            }
            _ => {}
        }

        drops
    }

    /// 获取攻击力（考虑武器加成）
    pub fn attack_power(&self) -> u32 {
        let base = self.attack;
        self.weapon
            .as_ref()
            .map_or(base, |w| base + w.damage_bonus())
    }

    /// 获取防御力
    pub fn defense(&self) -> u32 {
        self.defense
    }

    /// 获取命中率（考虑武器加成）
    pub fn accuracy(&self) -> i32 {
        let base = match self.kind {
            EnemyKind::Rat => 8,
            EnemyKind::Snake => 10,
            EnemyKind::Gnoll => 12,
            EnemyKind::Crab => 9,
            EnemyKind::Bat => 15,
            EnemyKind::Scorpion => 13,
            EnemyKind::Guard => 14,
            EnemyKind::Warlock => 16,
            EnemyKind::Golem => 10,
        };
        self.weapon
            .as_ref()
            .map_or(base, |w| base + w.accuracy_bonus())
    }

    /// 获取闪避率
    pub fn evasion(&self) -> u32 {
        match self.kind {
            EnemyKind::Rat => 6,
            EnemyKind::Snake => 12,
            EnemyKind::Gnoll => 8,
            EnemyKind::Crab => 5,
            EnemyKind::Bat => 18,
            EnemyKind::Scorpion => 10,
            EnemyKind::Guard => 9,
            EnemyKind::Warlock => 14,
            EnemyKind::Golem => 4,
        }
    }

    /// 获取暴击加成
    pub fn crit_bonus(&self) -> f32 {
        self.crit_bonus
    }

    /// 设置武器
    pub fn with_weapon(mut self, weapon: Weapon) -> Self {
        self.weapon = Some(weapon);
        self
    }

    /// 设置暴击加成
    pub fn with_crit_bonus(mut self, bonus: f32) -> Self {
        self.crit_bonus = bonus;
        self
    }

    /// 获取武器引用
    pub fn weapon(&self) -> Option<&Weapon> {
        self.weapon.as_ref()
    }

    /// 获取敌人名称
    pub fn name(&self) -> &str {
        match self.kind {
            EnemyKind::Rat => "Rat",
            EnemyKind::Snake => "Snake",
            EnemyKind::Gnoll => "Gnoll",
            EnemyKind::Crab => "Crab",
            EnemyKind::Bat => "Bat",
            EnemyKind::Scorpion => "Scorpion",
            EnemyKind::Guard => "Guard",
            EnemyKind::Warlock => "Warlock",
            EnemyKind::Golem => "Golem",
        }
    }

    /// 计算攻击伤害（考虑惊讶状态和武器）
    pub fn calculate_attack(&self) -> u32 {
        let base_damage = self.attack_power();
        let damage = if self.is_surprised {
            base_damage / 2 // 惊讶状态下伤害减半
        } else {
            base_damage
        };

        // 添加随机波动 (80%-120%)
        let mut rng = rand::rng();
        let damage_var = 0.8 + rng.random_range(0.0..0.4);
        (damage as f32 * damage_var) as u32
    }

    /// 治疗指定数值
    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// 获取攻击距离（优先使用武器距离）
    pub fn attack_distance(&self) -> u32 {
        self.weapon
            .as_ref()
            .map_or(self.attack_range, |w| w.range())
    }

    /// 判断是否为远程敌人
    pub fn is_ranged(&self) -> bool {
        self.attack_distance() > 1
    }
}
