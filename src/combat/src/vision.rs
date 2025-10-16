//! Vision and ambush system for combat mechanics
use crate::combatant::Combatant;
use std::collections::HashSet;

/// Represents a field of view for determining ambush opportunities
pub struct VisionSystem;

impl VisionSystem {
    /// Calculate the field of view for a combatant
    /// Returns a set of coordinates that the combatant can see
    pub fn calculate_fov(
        x: i32,
        y: i32,
        range: u32,
        is_blocked: &dyn Fn(i32, i32) -> bool,
    ) -> HashSet<(i32, i32)> {
        let mut visible_tiles = HashSet::new();
        let range_sq = (range * range) as i32;

        // Add the starting position
        visible_tiles.insert((x, y));

        // Check all tiles within the square that bounds the circular range
        let range_i32 = range as i32;
        for dx in (-range_i32)..=range_i32 {
            for dy in (-range_i32)..=range_i32 {
                // Skip if outside the circular range
                if dx * dx + dy * dy > range_sq {
                    continue;
                }

                let target_x = x + dx;
                let target_y = y + dy;

                // Use raycasting to check if the tile is visible
                if Self::is_visible(x, y, target_x, target_y, is_blocked) {
                    visible_tiles.insert((target_x, target_y));
                }
            }
        }

        visible_tiles
    }

    /// Check if a target is visible from a source using raycasting
    pub fn is_visible(
        src_x: i32,
        src_y: i32,
        target_x: i32,
        target_y: i32,
        is_blocked: &dyn Fn(i32, i32) -> bool,
    ) -> bool {
        // Use Bresenham's line algorithm to trace a line between source and target
        let dx = (target_x - src_x).abs();
        let dy = (target_y - src_y).abs();
        let sx = if src_x < target_x { 1 } else { -1 };
        let sy = if src_y < target_y { 1 } else { -1 };

        let mut err = dx - dy;
        let mut current_x = src_x;
        let mut current_y = src_y;

        loop {
            // If we've reached the target, it's visible
            if current_x == target_x && current_y == target_y {
                return true;
            }

            // If this tile is blocked, the target is not visible
            if is_blocked(current_x, current_y) {
                return false;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                current_x += sx;
            }
            if e2 < dx {
                err += dx;
                current_y += sy;
            }
        }
    }

    /// Determine if an attacker can ambush a defender based on visibility
    pub fn can_ambush<T: Combatant, U: Combatant>(
        _attacker: &T,
        attacker_x: i32,
        attacker_y: i32,
        _defender: &U,
        defender_x: i32,
        defender_y: i32,
        is_blocked: &dyn Fn(i32, i32) -> bool,
        attacker_fov_range: u32,
    ) -> bool {
        // Calculate the attacker's field of view
        let attacker_fov =
            Self::calculate_fov(attacker_x, attacker_y, attacker_fov_range, is_blocked);

        // Check if the defender is within the attacker's field of view
        let defender_in_view = attacker_fov.contains(&(defender_x, defender_y));

        // For an ambush to occur, the defender must NOT be visible to the attacker
        // (i.e., the attacker is approaching from behind or from a hidden position)
        !defender_in_view
    }

    /// Determine if an attacker is vulnerable to a counter-ambush from defender
    pub fn is_vulnerable_to_ambush<T: Combatant, U: Combatant>(
        _attacker: &T,
        attacker_x: i32,
        attacker_y: i32,
        _defender: &U,
        defender_x: i32,
        defender_y: i32,
        is_blocked: &dyn Fn(i32, i32) -> bool,
        defender_fov_range: u32,
    ) -> bool {
        // Calculate the defender's field of view
        let defender_fov =
            Self::calculate_fov(defender_x, defender_y, defender_fov_range, is_blocked);

        // Check if the attacker is within the defender's field of view
        let attacker_in_view = defender_fov.contains(&(attacker_x, attacker_y));

        // The attacker is vulnerable if they are NOT in the defender's field of view
        // (i.e., they're attacking from behind or from a hidden position)
        !attacker_in_view
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCombatant {
        name: String,
        fov_range: u32,
    }

    impl MockCombatant {
        fn new(name: &str, fov_range: u32) -> Self {
            Self {
                name: name.to_string(),
                fov_range,
            }
        }
    }

    impl Combatant for MockCombatant {
        fn id(&self) -> u32 {
            0
        } // 添加缺失的id方法
        fn hp(&self) -> u32 {
            100
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
            30
        }
        fn crit_bonus(&self) -> f32 {
            0.1
        }
        fn weapon(&self) -> Option<&items::Weapon> {
            None
        }
        fn is_alive(&self) -> bool {
            true
        }
        fn name(&self) -> &str {
            &self.name
        }
        fn attack_distance(&self) -> u32 {
            1
        }
        fn take_damage(&mut self, _amount: u32) -> bool {
            true
        }
        fn heal(&mut self, _amount: u32) {}
    }

    #[test]
    fn test_ambush_detection() {
        let attacker = MockCombatant::new("Attacker", 5);
        let defender = MockCombatant::new("Defender", 4);

        // Create a simple map with walls
        let is_blocked = |x: i32, y: i32| -> bool {
            // Block the path between attacker and defender
            x == 5 && y == 5
        };

        // Attacker is at (0,0), defender is at (10,10)
        // With a wall at (5,5) blocking the line of sight
        let can_ambush =
            VisionSystem::can_ambush(&attacker, 0, 0, &defender, 10, 10, &is_blocked, 5);

        // Should be true because the defender is not in view of the attacker
        assert!(can_ambush);
    }

    #[test]
    fn test_normal_attack() {
        let attacker = MockCombatant::new("Attacker", 5);
        let defender = MockCombatant::new("Defender", 4);

        // Create an open map (no walls)
        let is_blocked = |_x: i32, _y: i32| -> bool { false };

        // Attacker is at (0,0), defender is at (2,2)
        let can_ambush = VisionSystem::can_ambush(&attacker, 0, 0, &defender, 2, 2, &is_blocked, 5);

        // Should be false because the defender is visible to the attacker
        assert!(!can_ambush);
    }
}
