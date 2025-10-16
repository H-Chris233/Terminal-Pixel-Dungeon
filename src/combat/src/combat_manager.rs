//! Combat manager for handling turn-based combat mechanics
use crate::vision::VisionSystem;
use crate::{Combat, CombatResult, Combatant};

/// Manages combat rounds and turns
pub struct CombatManager;

impl CombatManager {
    /// Process a full combat round between two combatants
    pub fn process_combat_round<T: Combatant, U: Combatant>(
        attacker: &mut T,
        attacker_x: i32,
        attacker_y: i32,
        defender: &mut U,
        defender_x: i32,
        defender_y: i32,
        is_blocked: &dyn Fn(i32, i32) -> bool,
        attacker_fov_range: u32,
    ) -> CombatResult {
        // Perform the attack with ambush consideration
        Combat::perform_attack_with_ambush(
            attacker,
            0, // attacker_id - 现在需要提供ID
            attacker_x,
            attacker_y,
            defender,
            0, // defender_id - 现在需要提供ID
            defender_x,
            defender_y,
            is_blocked,
            attacker_fov_range,
        )
    }

    /// Process initiative-based combat where combatants act in turn order
    pub fn process_initiative_combat<T: Combatant, U: Combatant>(
        attacker: &mut T,
        attacker_x: i32,
        attacker_y: i32,
        defender: &mut U,
        defender_x: i32,
        defender_y: i32,
        is_blocked: &dyn Fn(i32, i32) -> bool,
        attacker_fov_range: u32,
    ) -> CombatResult {
        let mut result = CombatResult::new();

        // Determine initiative (simplified as attacker always goes first)
        // In a full implementation, this would be based on agility, weapon speed, etc.

        // Attacker's turn
        let attack_result = Combat::perform_attack_with_ambush(
            attacker,
            0, // attacker_id
            attacker_x,
            attacker_y,
            defender,
            0, // defender_id
            defender_x,
            defender_y,
            is_blocked,
            attacker_fov_range,
        );

        result.combine(attack_result);

        // Defender knows about the attacker now, so no ambush possible
        if defender.is_alive() {
            let defender_result = Combat::engage_with_ids(defender, 0, attacker, 0, false);
            result.combine(defender_result);
        }

        result
    }

    /// Process ranged combat with distance considerations
    pub fn process_ranged_combat<T: Combatant, U: Combatant>(
        attacker: &mut T,
        attacker_x: i32,
        attacker_y: i32,
        defender: &mut U,
        defender_x: i32,
        defender_y: i32,
        is_blocked: &dyn Fn(i32, i32) -> bool,
        attacker_fov_range: u32,
    ) -> CombatResult {
        let distance = calculate_distance(attacker_x, attacker_y, defender_x, defender_y);

        // Check if target is within attack range
        if distance > attacker.attack_distance() as f32 {
            let mut result = CombatResult::new();
            result.log(format!(
                "{} is out of range for {}",
                defender.name(),
                attacker.name()
            ));
            return result;
        }

        // Check if there's a clear line of sight
        let has_los =
            VisionSystem::is_visible(attacker_x, attacker_y, defender_x, defender_y, is_blocked);

        if !has_los {
            let mut result = CombatResult::new();
            result.log(format!("No line of sight to {}", defender.name()));
            return result;
        }

        // Perform the ranged attack
        Combat::perform_attack_with_ambush(
            attacker,
            0, // attacker_id
            attacker_x,
            attacker_y,
            defender,
            0, // defender_id
            defender_x,
            defender_y,
            is_blocked,
            attacker_fov_range,
        )
    }
}

/// Calculate distance between two points
fn calculate_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {
    let dx = (x1 - x2) as f32;
    let dy = (y1 - y2) as f32;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemy::Enemy;
    use crate::enemy::EnemyKind;

    struct TestCombatant {
        name: String,
        hp: u32,
        max_hp: u32,
        attack: u32,
        defense: u32,
        accuracy: u32,
        evasion: u32,
        crit_bonus: f32,
        attack_dist: u32,
    }

    impl TestCombatant {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                hp: 100,
                max_hp: 100,
                attack: 10,
                defense: 5,
                accuracy: 80,
                evasion: 20,
                crit_bonus: 0.1,
                attack_dist: 1,
            }
        }
    }

    impl Combatant for TestCombatant {
        fn id(&self) -> u32 {
            0
        } // 添加缺失的id方法
        fn hp(&self) -> u32 {
            self.hp
        }
        fn max_hp(&self) -> u32 {
            self.max_hp
        }
        fn attack_power(&self) -> u32 {
            self.attack
        }
        fn defense(&self) -> u32 {
            self.defense
        }
        fn accuracy(&self) -> u32 {
            self.accuracy
        }
        fn evasion(&self) -> u32 {
            self.evasion
        }
        fn crit_bonus(&self) -> f32 {
            self.crit_bonus
        }
        fn weapon(&self) -> Option<&items::Weapon> {
            None
        }
        fn is_alive(&self) -> bool {
            self.hp > 0
        }
        fn name(&self) -> &str {
            &self.name
        }
        fn attack_distance(&self) -> u32 {
            self.attack_dist
        }
        fn take_damage(&mut self, amount: u32) -> bool {
            self.hp = self.hp.saturating_sub(amount);
            self.is_alive()
        }
        fn heal(&mut self, amount: u32) {
            self.hp = std::cmp::min(self.max_hp, self.hp + amount);
        }
    }

    #[test]
    fn test_combat_round() {
        let mut attacker = TestCombatant::new("Attacker");
        let mut defender = TestCombatant::new("Defender");

        let is_blocked = |_x: i32, _y: i32| -> bool { false };

        let result = CombatManager::process_combat_round(
            &mut attacker,
            0,
            0,
            &mut defender,
            1,
            1,
            &is_blocked,
            5,
        );

        assert!(!result.logs.is_empty());
        assert!(result.logs[0].contains("hits") || result.logs[0].contains("misses"));
    }

    #[test]
    fn test_ambush_combat() {
        let mut attacker = TestCombatant::new("Attacker");
        let mut defender = TestCombatant::new("Defender");

        // Wall blocking line of sight to enable ambush
        let is_blocked = |x: i32, y: i32| -> bool {
            x == 0 && y == 1 // Wall between attacker at (0,0) and defender at (0,2)
        };

        let result = CombatManager::process_combat_round(
            &mut attacker,
            0,
            0,
            &mut defender,
            0,
            2, // Across the wall
            &is_blocked,
            5,
        );

        assert!(!result.logs.is_empty());
        // Should contain ambush message if ambush worked
        let is_ambush = result.logs.iter().any(|log| log.contains("Ambush"));
        assert!(is_ambush);
    }
}
