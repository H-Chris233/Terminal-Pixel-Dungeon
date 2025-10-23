//! Combat manager for handling turn-based combat mechanics
use crate::vision::VisionSystem;
use crate::{AttackParams, Combat, CombatResult, Combatant};

/// Manages combat rounds and turns
pub struct CombatManager;

impl CombatManager {
    /// Process a full combat round between two combatants
    pub fn process_combat_round<T: Combatant, U: Combatant>(
        params: &mut AttackParams<T, U>,
    ) -> CombatResult {
        Combat::perform_attack_with_ambush(params)
    }

    /// Process initiative-based combat where combatants act in turn order
    pub fn process_initiative_combat<T: Combatant, U: Combatant>(
        params: &mut AttackParams<T, U>,
    ) -> CombatResult {
        let mut result = CombatResult::new();

        let attack_result = Combat::perform_attack_with_ambush(params);
        result.combine(attack_result);

        if params.defender.is_alive() {
            let defender_result = Combat::engage_with_ids(
                params.defender,
                params.defender_id,
                params.attacker,
                params.attacker_id,
                false,
            );
            result.combine(defender_result);
        }

        result
    }

    /// Process ranged combat with distance considerations
    pub fn process_ranged_combat<T: Combatant, U: Combatant>(
        params: &mut AttackParams<T, U>,
    ) -> CombatResult {
        let distance = calculate_distance(
            params.attacker_x,
            params.attacker_y,
            params.defender_x,
            params.defender_y,
        );

        if distance > params.attacker.attack_distance() as f32 {
            let mut result = CombatResult::new();
            result.log(format!(
                "{} is out of range for {}",
                params.defender.name(),
                params.attacker.name()
            ));
            return result;
        }

        let has_los = VisionSystem::is_visible(
            params.attacker_x,
            params.attacker_y,
            params.defender_x,
            params.defender_y,
            params.is_blocked,
        );
        if !has_los {
            let mut result = CombatResult::new();
            result.log(format!("No line of sight to {}", params.defender.name()));
            return result;
        }

        Combat::perform_attack_with_ambush(params)
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
        fn id(&self) -> u32 { 0 }
        fn hp(&self) -> u32 { self.hp }
        fn max_hp(&self) -> u32 { self.max_hp }
        fn attack_power(&self) -> u32 { self.attack }
        fn defense(&self) -> u32 { self.defense }
        fn accuracy(&self) -> u32 { self.accuracy }
        fn evasion(&self) -> u32 { self.evasion }
        fn crit_bonus(&self) -> f32 { self.crit_bonus }
        fn weapon(&self) -> Option<&items::Weapon> { None }
        fn is_alive(&self) -> bool { self.hp > 0 }
        fn name(&self) -> &str { &self.name }
        fn attack_distance(&self) -> u32 { self.attack_dist }
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

        let mut params = AttackParams {
            attacker: &mut attacker,
            attacker_id: 0,
            attacker_x: 0,
            attacker_y: 0,
            defender: &mut defender,
            defender_id: 1,
            defender_x: 1,
            defender_y: 1,
            is_blocked: &is_blocked,
            attacker_fov_range: 5,
        };

        let result = CombatManager::process_combat_round(&mut params);
        assert!(!result.logs.is_empty());
        let has_hit_or_miss = result
            .logs
            .iter()
            .any(|log| log.contains("hit") || log.contains("miss"));
        assert!(
            has_hit_or_miss,
            "Expected combat log to contain hit/miss info, logs: {:?}",
            result.logs
        );
    }

    #[test]
    fn test_ambush_combat() {
        let mut attacker = TestCombatant::new("Attacker");
        let mut defender = TestCombatant::new("Defender");

        // Wall blocking line of sight to enable ambush
        let is_blocked = |x: i32, y: i32| -> bool {
            x == 0 && y == 1 // Wall between attacker at (0,0) and defender at (0,2)
        };

        let mut params = AttackParams {
            attacker: &mut attacker,
            attacker_id: 0,
            attacker_x: 0,
            attacker_y: 0,
            defender: &mut defender,
            defender_id: 1,
            defender_x: 0,
            defender_y: 2,
            is_blocked: &is_blocked,
            attacker_fov_range: 5,
        };

        let result = CombatManager::process_combat_round(&mut params);
        assert!(!result.logs.is_empty());
        // Should contain ambush message if ambush worked
        let is_ambush = result.logs.iter().any(|log| log.contains("Ambush"));
        assert!(is_ambush);
    }
}
