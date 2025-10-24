// src/combat/src/lib.rs

use rand::Rng; // 添加这个导入以修复 random_bool 和 random_range 错误

pub mod boss;
pub mod combat_manager;
pub mod combatant;
pub mod effect;
pub mod enemy;
pub mod status_effect;
#[cfg(test)]
mod tests;
pub mod vision;

pub use crate::boss::{Boss, BossLoot, BossPhase, BossSkill, BossType, SkillCooldowns};
pub use crate::combatant::Combatant;
pub use crate::effect::*;
pub use crate::enemy::Enemy;

/// 打包一次攻击所需的参数，减少长参数列表
pub struct AttackParams<'a, T: Combatant, U: Combatant> {
    pub attacker: &'a mut T,
    pub attacker_id: u32,
    pub attacker_x: i32,
    pub attacker_y: i32,
    pub defender: &'a mut U,
    pub defender_id: u32,
    pub defender_x: i32,
    pub defender_y: i32,
    pub is_blocked: &'a dyn Fn(i32, i32) -> bool,
    pub attacker_fov_range: u32,
}

/// Handles combat interactions between entities
pub struct Combat;

/// Combat configuration constants (balanced to match Shattered PD values)
mod constants {
    pub const BASE_HIT_CHANCE: f32 = 0.8; // Base hit chance
    pub const MIN_HIT_CHANCE: f32 = 0.05; // Minimum possible hit chance
    pub const MAX_HIT_CHANCE: f32 = 0.95; // Maximum possible hit chance
    pub const CRIT_MULTIPLIER: f32 = 1.5; // Critical damage multiplier
    pub const BASE_CRIT_CHANCE: f32 = 0.1; // Base critical chance
    pub const DEFENSE_CAP: f32 = 0.8; // Maximum damage reduction from defense
    pub const MIN_DAMAGE: u32 = 1; // Minimum damage dealt
    pub const SURPRISE_ATTACK_MODIFIER: f32 = 2.0; // Damage bonus for surprise attacks (2x damage)
}

impl Combat {
    /// Engage in combat between two combatants (player vs enemy or enemy vs player)
    pub fn engage<T: Combatant, U: Combatant>(
        attacker: &mut T,
        defender: &mut U,
        is_ambush: bool, // Whether this is an ambush attack
    ) -> CombatResult {
        Self::engage_with_ids(attacker, attacker.id(), defender, defender.id(), is_ambush)
    }

    /// 使用参数包执行一次考虑潜行的攻击，减少长参数列表
    pub fn perform_attack_with_ambush<T: Combatant, U: Combatant>(
        params: &mut AttackParams<T, U>,
    ) -> CombatResult {
        // Check if attacker can ambush defender
        let is_ambush = vision::VisionSystem::can_ambush(
            params.attacker,
            params.attacker_x,
            params.attacker_y,
            params.defender,
            params.defender_x,
            params.defender_y,
            params.is_blocked,
            params.attacker_fov_range,
        );

        Self::engage_with_ids(
            params.attacker,
            params.attacker_id,
            params.defender,
            params.defender_id,
            is_ambush,
        )
    }

    /// Engage in combat between two combatants with explicit IDs (for event bus purposes)
    pub fn engage_with_ids<T: Combatant, U: Combatant>(
        attacker: &mut T,
        attacker_id: u32,
        defender: &mut U,
        defender_id: u32,
        is_ambush: bool, // Whether this is an ambush attack
    ) -> CombatResult {
        let mut result = CombatResult::new();

        // 发布战斗开始事件
        result.add_event(CombatEvent::CombatStarted {
            attacker: attacker_id,
            defender: defender_id,
        });

        // Attacker's turn (with potential ambush bonus)
        if is_ambush {
            result.log(format!("Ambush by {}!", attacker.name()));
            // 发布潜行攻击事件
            result.add_event(CombatEvent::Ambush {
                attacker: attacker_id,
                defender: defender_id,
            });
        }
        let attack_result =
            Self::resolve_attack_with_ids(attacker, attacker_id, defender, defender_id, is_ambush);
        result.combine(attack_result);

        // Defender's counterattack if alive (no ambush bonus since they know attacker is there)
        if defender.is_alive() {
            let counter_result =
                Self::resolve_attack_with_ids(defender, defender_id, attacker, attacker_id, false);
            result.combine(counter_result);
        }

        result
    }

    /// Calculate hit chance (SPD-style formula)
    pub fn calculate_hit_chance<T: Combatant, U: Combatant>(attacker: &T, defender: &U) -> f32 {
        let accuracy = attacker.accuracy() as f32;
        let evasion = defender.evasion() as f32;

        // SPD formula: base + (accuracy - evasion)/20
        let hit_chance = constants::BASE_HIT_CHANCE + (accuracy - evasion) / 20.0;
        hit_chance.clamp(constants::MIN_HIT_CHANCE, constants::MAX_HIT_CHANCE)
    }

    /// Check for critical hit (based on attacker's crit bonus)
    pub fn is_critical<T: Combatant>(attacker: &T) -> bool {
        let crit_chance = constants::BASE_CRIT_CHANCE + attacker.crit_bonus();
        rand::rng().random_bool(crit_chance as f64)
    }

    /// Calculate damage with all modifiers (SPD-style)
    pub fn calculate_damage<T: Combatant, U: Combatant>(
        attacker: &T,
        defender: &U,
        is_ambush: bool,
    ) -> u32 {
        // Base damage with weapon variation (80-120%)
        let base_damage = attacker.attack_power() as f32;
        let damage_var = 0.8 + rand::rng().random_range(0.0..0.4);
        let mut raw_damage = base_damage * damage_var;

        // Apply critical hit
        if Self::is_critical(attacker) {
            raw_damage *= constants::CRIT_MULTIPLIER;
        }

        // Apply ambush attack modifier (2x damage for unaware targets)
        if is_ambush {
            raw_damage *= constants::SURPRISE_ATTACK_MODIFIER;
        }

        // Defense reduces damage by percentage (capped at DEFENSE_CAP)
        let defense = defender.defense() as f32;
        let defense_factor = (defense / (defense + 5.0)).min(constants::DEFENSE_CAP);
        let mitigated_damage = raw_damage * (1.0 - defense_factor);

        // Ensure minimum damage is dealt
        mitigated_damage.max(constants::MIN_DAMAGE as f32) as u32
    }

    /// Resolve a single attack with combat logs
    pub fn resolve_attack<T: Combatant, U: Combatant>(
        attacker: &mut T,
        defender: &mut U,
        is_ambush: bool,
    ) -> CombatResult {
        Self::resolve_attack_with_ids(attacker, attacker.id(), defender, defender.id(), is_ambush)
    }

    /// Resolve a single attack with combat logs and explicit IDs
    fn resolve_attack_with_ids<T: Combatant, U: Combatant>(
        attacker: &mut T,
        attacker_id: u32,
        defender: &mut U,
        defender_id: u32,
        is_ambush: bool,
    ) -> CombatResult {
        let mut result = CombatResult::new();

        if Self::does_attack_hit(attacker, defender) {
            let damage = Self::calculate_damage(attacker, defender, is_ambush);
            let is_crit = Self::is_critical(attacker);

            // Apply damage and check for death
            defender.take_damage(damage);

            // 发布伤害事件
            result.add_event(CombatEvent::DamageDealt {
                attacker: attacker_id,
                victim: defender_id,
                damage,
                is_critical: is_crit,
            });

            // Build combat message
            let mut damage_msg = if is_crit {
                format!("Critical hit! {} deals {} damage", attacker.name(), damage)
            } else if is_ambush {
                format!(
                    "Ambush! {} deals {} damage (2x damage bonus)",
                    attacker.name(),
                    damage
                )
            } else {
                format!("{} hits for {} damage", attacker.name(), damage)
            };

            damage_msg += "!";
            result.log(damage_msg);

            // Check for death
            if !defender.is_alive() {
                result.log(format!("{} defeated {}!", attacker.name(), defender.name()));
                result.defeated = true;
                result.experience = defender.exp_value();

                // 发布实体死亡事件
                result.add_event(CombatEvent::EntityDied {
                    entity: defender_id,
                    entity_name: defender.name().to_string(),
                });
            }
        } else {
            result.log(format!("{} misses {}!", attacker.name(), defender.name()));
        }

        result
    }

    /// Determine if an attack hits (wrapper for hit chance calculation)
    pub fn does_attack_hit<T: Combatant, U: Combatant>(attacker: &T, defender: &U) -> bool {
        let hit_chance = Self::calculate_hit_chance(attacker, defender);
        rand::rng().random_bool(hit_chance as f64)
    }
}

/// Combat result with detailed logs and event information
#[derive(Debug, Clone, Default)]
pub struct CombatResult {
    pub logs: Vec<String>,        // Combat messages for UI
    pub defeated: bool,           // Whether target was defeated
    pub experience: u32,          // Experience gained (if any)
    pub events: Vec<CombatEvent>, // Events to be published to event bus
}

/// Combat events that can be published to the event bus
#[derive(Debug, Clone)]
pub enum CombatEvent {
    CombatStarted {
        attacker: u32,
        defender: u32,
    },
    DamageDealt {
        attacker: u32,
        victim: u32,
        damage: u32,
        is_critical: bool,
    },
    EntityDied {
        entity: u32,
        entity_name: String,
    },
    Ambush {
        attacker: u32,
        defender: u32,
    },
}

impl CombatResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log(&mut self, message: String) {
        self.logs.push(message);
    }

    pub fn add_event(&mut self, event: CombatEvent) {
        self.events.push(event);
    }

    pub fn combine(&mut self, other: CombatResult) {
        self.logs.extend(other.logs);
        self.defeated = self.defeated || other.defeated;
        self.experience += other.experience;
        self.events.extend(other.events);
    }
}
