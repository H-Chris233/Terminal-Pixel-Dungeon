// src/combat/src/combatant.rs

use crate::enemy::Enemy;
use crate::enemy::EnemyKind;
use items::weapon::Weapon;

/// 表示可以参加战斗的活体
pub trait Combatant {
    /// 获取当前生命值
    fn hp(&self) -> i32;

    /// 获取最大生命值
    fn max_hp(&self) -> i32;

    /// 获取基础攻击力
    fn attack_power(&self) -> i32;

    /// 获取防御力
    fn defense(&self) -> i32;

    /// 获取命中率
    fn accuracy(&self) -> i32;

    /// 获取闪避率
    fn evasion(&self) -> i32;

    /// 获取暴击加成
    fn crit_bonus(&self) -> f32;

    /// 获取武器引用
    fn weapon(&self) -> Option<&Weapon>;

    /// 是否存活
    fn is_alive(&self) -> bool;

    /// 获取名称
    fn name(&self) -> &str;

    /// 获取攻击距离
    fn attack_distance(&self) -> i32;

    /// 获取击败后提供的经验值
    fn experience_value(&self) -> Option<i32> {
        None // 默认不提供经验值
    }

    /// 是否为远程战斗者
    fn is_ranged(&self) -> bool {
        self.attack_distance() > 1
    }

    /// 造成伤害
    fn take_damage(&mut self, amount: i32) -> bool;

    /// 治疗
    fn heal(&mut self, amount: i32);
}

// 为Enemy实现Combatant
impl Combatant for Enemy {
    fn hp(&self) -> i32 {
        self.hp
    }

    fn max_hp(&self) -> i32 {
        self.max_hp
    }

    fn attack_power(&self) -> i32 {
        self.attack + self.weapon.as_ref().map_or(0, |w| w.damage_bonus() as i32)
    }

    fn defense(&self) -> i32 {
        self.defense
    }

    fn accuracy(&self) -> i32 {
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
        base + self
            .weapon
            .as_ref()
            .map_or(0, |w| w.accuracy_bonus() as i32)
    }

    fn evasion(&self) -> i32 {
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

    fn crit_bonus(&self) -> f32 {
        self.crit_bonus
    }

    fn weapon(&self) -> Option<&Weapon> {
        self.weapon.as_ref()
    }

    fn is_alive(&self) -> bool {
        self.hp > 0
    }

    fn name(&self) -> &str {
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

    fn attack_distance(&self) -> i32 {
        self.weapon
            .as_ref()
            .map_or(self.attack_range, |w| w.range() as i32)
    }

    fn take_damage(&mut self, amount: i32) -> bool {
        let actual_damage = (amount - self.defense).max(1);
        self.hp = (self.hp - actual_damage).max(0);
        self.is_alive()
    }

    fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }
    fn experience_value(&self) -> Option<i32> {
        Some(self.exp_value)
    }
}
