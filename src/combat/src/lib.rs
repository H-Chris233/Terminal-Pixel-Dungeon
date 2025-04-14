
// src/combat/src/lib.rs
#![allow(dead_code)]
#![allow(unused)]

use rand::Rng;
pub mod enemy;

use crate::enemy::Enemy;
use items::Weapon;

/// Handles combat interactions between entities
pub struct Combat;

/// Combat configuration constants
mod constants {
    pub const BASE_HIT_CHANCE: f32 = 0.8;
    pub const MIN_HIT_CHANCE: f32 = 0.05;
    pub const MAX_HIT_CHANCE: f32 = 0.95;
    pub const CRIT_MULTIPLIER: f32 = 1.5;
    pub const BASE_CRIT_CHANCE: f32 = 0.1;
    pub const DEFENSE_CAP: f32 = 0.8;
    pub const MIN_DAMAGE: i32 = 1;
    pub const RANGED_PENALTY_PER_TILE: f32 = 0.15; // 15% penalty per tile closer than max
}

impl Combat {
    /// Engage in combat between two entities
    pub fn engage(attacker: &mut Enemy, defender: &mut Enemy, distance: i32) -> CombatResult {
        let mut result = CombatResult::new();
        
        // Check if attacker can reach defender
        let max_range = attacker.weapon().map_or(1, |w| w.hit_distance);
        if distance > max_range.into() {
            result.log(format!("{} is out of range! ({} > {})", 
                defender.name(), distance, max_range));
            return result;
        }

        // Attacker's turn
        let attack_result = Self::resolve_attack(attacker, defender, distance);
        result.combine(attack_result);
        
        // Defender's counterattack if alive and in range
        if defender.is_alive() {
            let defender_range = defender.weapon().map_or(1, |w| w.hit_distance);
            if distance <= defender_range as i32 {
                let counter_result = Self::resolve_attack(defender, attacker, distance);
                result.combine(counter_result);
            }
        }
        
        result
    }

    /// Calculate hit chance (SPD-style)
    pub fn calculate_hit_chance(attacker: &Enemy, defender: &Enemy) -> f32 {
        let accuracy = attacker.accuracy() as f32;
        let evasion = defender.evasion() as f32;
        let hit_chance = constants::BASE_HIT_CHANCE + (accuracy - evasion) / 20.0;
        hit_chance.clamp(constants::MIN_HIT_CHANCE, constants::MAX_HIT_CHANCE)
    }

    /// Check for critical hit
    pub fn is_critical(attacker: &Enemy) -> bool {
        let crit_chance = constants::BASE_CRIT_CHANCE + attacker.crit_bonus();
        rand::rng().random_bool(crit_chance as f64)
    }

    /// Calculate damage with distance penalty (closer = higher penalty)
    pub fn calculate_damage(attacker: &Enemy, defender: &Enemy, distance: i32) -> i32 {
        let base_damage = attacker.attack_power() as f32;
        let defense = defender.defense() as f32;
        
        // Damage variation (80%-120%)
        let damage_var = 0.8 + rand::rng().random_range(0.0..0.4);
        let mut raw_damage = base_damage * damage_var;
        
        // Apply critical hit
        if Self::is_critical(attacker) {
            raw_damage *= constants::CRIT_MULTIPLIER;
        }
        
        // Apply distance penalty for ranged weapons
        if let Some(weapon) = attacker.weapon() {
            if weapon.hit_distance > 1 {
                let max_range = weapon.hit_distance;
                let tiles_closer = max_range as i32 - distance;
                let penalty = 1.0 - tiles_closer as f32 * constants::RANGED_PENALTY_PER_TILE;
                raw_damage *= penalty.max(0.1); // Minimum 10% damage
            }
        }
        
        // Defense reduces damage by a percentage
        let defense_factor = (defense / (defense + 5.0)).min(constants::DEFENSE_CAP);
        let mitigated_damage = raw_damage * (1.0 - defense_factor);
        
        mitigated_damage.max(constants::MIN_DAMAGE as f32) as i32
    }

    /// Resolve a single attack
    pub fn resolve_attack(attacker: &mut Enemy, defender: &mut Enemy, distance: i32) -> CombatResult {
        let mut result = CombatResult::new();
        
        if Self::does_attack_hit(attacker, defender) {
            let damage = Self::calculate_damage(attacker, defender, distance);
            let is_crit = Self::is_critical(attacker);
            
            defender.take_damage(damage);
            
            // Log damage with distance info if ranged
            let mut damage_msg = if is_crit {
                format!("Critical hit! {} deals {} damage", attacker.name(), damage)
            } else {
                format!("{} hits for {} damage", attacker.name(), damage)
            };
            
            if let Some(weapon) = attacker.weapon() {
                if weapon.hit_distance > 1 {
                    damage_msg += &format!(" from {} tiles", distance);
                }
            }
            
            damage_msg += "!";
            result.log(damage_msg);
            
            if !defender.is_alive() {
                result.log(format!("{} defeated {}!", attacker.name(), defender.name()));
                result.defeated = true;
                result.experience = defender.experience_value();
            }
        } else {
            result.log(format!("{} misses {}!", attacker.name(), defender.name()));
        }
        
        result
    }

    /// Determine if an attack hits
    pub fn does_attack_hit(attacker: &Enemy, defender: &Enemy) -> bool {
        let hit_chance = Self::calculate_hit_chance(attacker, defender);
        rand::rng().random_bool(hit_chance as f64)
    }
}

/// Combat result with logs and rewards
#[derive(Debug, Clone, Default)]
pub struct CombatResult {
    pub logs: Vec<String>,
    pub defeated: bool,
    pub experience: i32,
}

impl CombatResult {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn log(&mut self, message: String) {
        self.logs.push(message);
    }
    
    pub fn combine(&mut self, other: CombatResult) {
        self.logs.extend(other.logs);
        self.defeated = self.defeated || other.defeated;
        self.experience += other.experience;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemy::EnemyKind;
    use crate::items::weapon::{Weapon, WeaponType};

    #[test]
    fn test_ranged_distance_penalty() {
        let weapon = Weapon::new(WeaponType::Bow, 1).with_hit_distance(4);
        let mut attacker = Enemy::new(EnemyKind::Archer, 0, 0)
            .with_weapon(weapon)
            .with_attack_power(100); // High damage for clear testing
        
        let defender = Enemy::new(EnemyKind::Rat, 0, 0);
        
        // Max range (4 tiles) - no penalty
        let damage_at_max = Combat::calculate_damage(&attacker, &defender, 4);
        
        // 3 tiles (15% penalty)
        let damage_at_3 = Combat::calculate_damage(&attacker, &defender, 3);
        assert!(damage_at_3 < damage_at_max);
        
        // 2 tiles (30% penalty)
        let damage_at_2 = Combat::calculate_damage(&attacker, &defender, 2);
        assert!(damage_at_2 < damage_at_3);
        
        // 1 tile (45% penalty)
        let damage_at_1 = Combat::calculate_damage(&attacker, &defender, 1);
        assert!(damage_at_1 < damage_at_2);
        
        // Minimum damage check
        assert!(damage_at_1 >= constants::MIN_DAMAGE);
    }
    
    #[test]
    fn test_melee_no_distance_penalty() {
        let weapon = Weapon::new(WeaponType::Sword, 1).with_hit_distance(1);
        let mut attacker = Enemy::new(EnemyKind::Warrior, 0, 0)
            .with_weapon(weapon)
            .with_attack_power(100);
        
        let defender = Enemy::new(EnemyKind::Rat, 0, 0);
        
        let damage = Combat::calculate_damage(&attacker, &defender, 1);
        let expected_damage = Combat::calculate_damage(&attacker, &defender, 1);
        assert_eq!(damage, expected_damage); // No penalty at any distance
    }
    
    #[test]
    fn test_out_of_range() {
        let weapon = Weapon::new(WeaponType::Bow, 1).with_hit_distance(3);
        let mut attacker = Enemy::new(EnemyKind::Archer, 0, 0)
            .with_weapon(weapon);
        let mut defender = Enemy::new(EnemyKind::Rat, 0, 0);
        
        let result = Combat::engage(&mut attacker, &mut defender, 4);
        assert!(result.logs[0].contains("out of range"));
    }
}
