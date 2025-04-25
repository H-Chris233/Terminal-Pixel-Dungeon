// src/hero/combat.rs
use super::core::{Hero, HeroError};
use crate::class::Class;
use crate::BagError;

pub use combat::{effect::EffectType, Combat, Combatant};
use items::{armor::Armor, ring::Ring, weapon::Weapon};
use rand::Rng;

/// 战斗系统实现（确定性版本）
impl Combatant for Hero {
    fn hp(&self) -> u32 {
        self.hp
    }

    fn max_hp(&self) -> u32 {
        self.max_hp
    }

    /// 计算攻击力（包含武器加成和等级加成）
    fn attack_power(&self) -> u32 {
        let weapon_bonus = self
            .bag
            .equipment()
            .weapon
            .as_ref()
            .map_or(0, |w| w.damage_bonus() as u32);

        (self.base_attack + weapon_bonus) * (100 + self.level) / 100
    }

    /// 计算防御力（包含护甲加成）
    fn defense(&self) -> u32 {
        let armor_bonus = self.bag.armor().map_or(0, |a| a.defense() as u32);

        self.base_defense + armor_bonus
    }

    /// 计算命中率（包含武器加成）
    fn accuracy(&self) -> u32 {
        let weapon_bonus = self.bag.weapon().map_or(0, |w| w.accuracy_bonus() as u32);

        // SPD基础精度80 + 每级2点 + 武器加成
        80 + (self.level * 2) + weapon_bonus
    }

    /// 计算闪避率（受护甲惩罚）
    fn evasion(&self) -> u32 {
        let armor_penalty = self.bag.armor().map_or(0, |a| a.evasion_penalty());

        // 每级3点 - 护甲惩罚
        (self.level * 3).saturating_sub(armor_penalty)
    }

    /// 暴击率计算（确定性版本）
    fn crit_bonus(&self) -> f32 {
        let class_bonus = match self.class {
            Class::Warrior => 0.05,
            Class::Mage => 0.0,
            Class::Rogue => 0.15,
            Class::Huntress => 0.07,
        };

        let weapon_bonus = self.bag.weapon().map_or(0.0, |w| w.crit_bonus());

        let ring_bonus: f32 = self.bag.rings().iter().map(|r| r.crit_bonus()).sum();

        // 基础10% + 职业加成 + 装备加成
        0.1 + class_bonus + weapon_bonus + ring_bonus
    }

    fn weapon(&self) -> Option<&Weapon> {
        self.bag.weapon()
    }

    fn is_alive(&self) -> bool {
        self.alive && self.hp > 0
    }

    fn name(&self) -> &str {
        &self.name
    }

    /// 攻击距离（由武器决定）
    fn attack_distance(&self) -> u32 {
        self.weapon().map_or(1, |w| w.range() as u32)
    }

    /// 承受伤害（SPD防御公式）
    fn take_damage(&mut self, amount: u32) -> bool {
        // SPD防御公式：防御力 × (0.7~1.3随机系数)
        let defense_roll = self.defense() as f32 * (0.7 + self.rng.gen_range(0.0..0.6));

        // 实际伤害 = 攻击力 - 防御roll值（至少1点）
        let actual_damage = (amount as f32 - defense_roll).max(1.0) as u32;

        self.hp = self.hp.saturating_sub(actual_damage);
        self.alive = self.hp > 0;

        if !self.alive {
            self.notify("你死了...".into());
        }
        self.is_alive()
    }

    /// 治疗（不超过最大HP）
    fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }
}

/// 英雄特有的战斗扩展方法
impl Hero {
    /// 执行攻击（返回是否暴击和实际伤害）
    pub fn perform_attack(&mut self, target: &mut dyn Combatant) -> (bool, u32) {
        let is_crit = self.is_critical();
        let base_damage = self.attack_power();
        let final_damage = if is_crit {
            (base_damage as f32 * 1.5) as u32 // 暴击伤害150%
        } else {
            base_damage
        };

        let target_alive = target.take_damage(final_damage);
        if !target_alive {
            self.gain_exp(target.exp_value());
        }

        (is_crit, final_damage)
    }

    /// 计算命中概率（0.0-1.0）
    pub fn hit_probability(&self, target: &dyn Combatant) -> f32 {
        let accuracy = self.accuracy() as f32;
        let evasion = target.evasion() as f32;

        // SPD命中公式：min(0.9, max(0.1, acc/(acc + eva)))
        let raw_prob = accuracy / (accuracy + evasion);
        raw_prob.clamp(0.1, 0.9)
    }

    /// 尝试闪避攻击（返回是否闪避成功）
    pub fn try_evade(&mut self, attacker: &dyn Combatant) -> bool {
        let hit_prob = attacker.hit_probability(self);
        !self.rng.gen_bool(hit_prob as f64)
    }

    /// 反击机制（盗贼专属）
    pub fn counter_attack(&mut self, attacker: &mut dyn Combatant) -> Option<(bool, u32)> {
        if self.class == Class::Rogue && self.rng.gen_bool(0.3) {
            Some(self.perform_attack(attacker))
        } else {
            None
        }
    }

    /// 范围攻击（法师专属）
    pub fn area_attack(&mut self, targets: &mut Vec<&mut dyn Combatant>) -> Vec<(bool, u32)> {
        if self.class != Class::Mage {
            return Vec::new();
        }

        targets
            .iter_mut()
            .map(|t| self.perform_attack(*t))
            .collect()
    }

    /// 远程攻击（女猎手专属）
    pub fn ranged_attack(&mut self, target: &mut dyn Combatant) -> Option<(bool, u32)> {
        if self.class == Class::Huntress && self.attack_distance() > 1 {
            Some(self.perform_attack(target))
        } else {
            None
        }
    }
    /// 武器升级（需要添加到Hero实现中）
    pub fn upgrade_weapon(&mut self) -> Result<(), HeroError> {
        self.bag.upgrade_weapon().map_err(|e| match e {
            BagError::NoWeaponEquipped => HeroError::Underpowered,
            BagError::NoUpgradeScroll => HeroError::IdentifyFailed,
            _ => HeroError::from(e),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{Weapon, WeaponKind};

    struct MockCombatant {
        hp: u32,
        evasion: u32,
        exp_value: u32,
    }

    impl Combatant for MockCombatant {
        fn hp(&self) -> u32 {
            self.hp
        }
        fn max_hp(&self) -> u32 {
            100
        }
        fn attack_power(&self) -> u32 {
            10
        }
        fn defense(&self) -> u32 {
            5
        }
        fn accuracy(&self) -> u32 {
            80
        }
        fn evasion(&self) -> u32 {
            self.evasion
        }
        fn crit_bonus(&self) -> f32 {
            0.0
        }
        fn is_critical(&mut self) -> bool {
            false
        }
        fn weapon(&self) -> Option<&Weapon> {
            None
        }
        fn is_alive(&self) -> bool {
            self.hp > 0
        }
        fn name(&self) -> &str {
            "Mock"
        }
        fn take_damage(&mut self, amount: u32) -> bool {
            self.hp = self.hp.saturating_sub(amount);
            self.is_alive()
        }
        fn heal(&mut self, _: u32) {}
        fn exp_value(&self) -> u32 {
            self.exp_value
        }
    }

    #[test]
    fn test_warrior_attack() {
        let mut hero = Hero::with_seed(Class::Warrior, 123);
        hero.level = 5;
        hero.base_attack = 15;
        hero.bag
            .equip_weapon(Weapon::new(WeaponKind::Sword, 1))
            .unwrap();

        let mut target = MockCombatant {
            hp: 100,
            evasion: 10,
            exp_value: 50,
        };

        let (is_crit, damage) = hero.perform_attack(&mut target);
        assert!(damage >= 15);
        assert_eq!(is_crit, false); // 种子123的第一击不暴击
        assert_eq!(hero.experience, 0); // 目标未死亡

        target.hp = 1;
        hero.perform_attack(&mut target);
        assert_eq!(hero.experience, 50); // 目标死亡获得经验
    }

    #[test]
    fn test_rogue_counter() {
        let mut hero = Hero::with_seed(Class::Rogue, 456);
        let mut attacker = MockCombatant {
            hp: 100,
            evasion: 20,
            exp_value: 0,
        };

        // 测试30%概率的反击
        let mut counter_count = 0;
        for _ in 0..1000 {
            if hero.counter_attack(&mut attacker).is_some() {
                counter_count += 1;
            }
        }
        assert!(counter_count > 250 && counter_count < 350); // ~30%
    }

    #[test]
    fn test_mage_area_attack() {
        let mut hero = Hero::with_seed(Class::Mage, 789);
        let mut targets = vec![
            &mut MockCombatant {
                hp: 50,
                evasion: 5,
                exp_value: 0,
            } as &mut dyn Combatant,
            &mut MockCombatant {
                hp: 50,
                evasion: 5,
                exp_value: 0,
            },
        ];

        let results = hero.area_attack(&mut targets);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_hit_probability() {
        let hero = Hero::with_seed(Class::Huntress, 999);
        let target = MockCombatant {
            hp: 100,
            evasion: 20,
            exp_value: 0,
        };

        let prob = hero.hit_probability(&target);
        assert!(prob >= 0.1 && prob <= 0.9);
    }

    #[test]
    fn test_defense_calculation() {
        let mut hero = Hero::with_seed(Class::Warrior, 111);
        hero.base_defense = 10;
        hero.bag.equip_armor(Armor::new("chainmail", 6)).unwrap();

        let defense = hero.defense();
        assert_eq!(defense, 16); // 10基础 + 6护甲
    }
    #[test]
    fn test_weapon_upgrade() {
        let mut hero = Hero::with_seed(Class::Warrior, 555);
        hero.bag
            .equip_weapon(Weapon::new(WeaponKind::Sword, 0))
            .unwrap();
        hero.bag
            .add_item(Item::from(Scroll::new(ScrollKind::Upgrade)))
            .unwrap();

        let initial_damage = hero.attack_power();
        hero.upgrade_weapon().unwrap();
        assert!(hero.attack_power() > initial_damage);
    }
}
