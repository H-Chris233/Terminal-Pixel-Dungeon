//! Achievement progress tracking

use serde::{Deserialize, Serialize};

/// Tracks progress toward all achievement criteria
#[derive(Debug, Clone, Serialize, Deserialize, Default, bincode::Encode, bincode::Decode)]
pub struct AchievementProgress {
    /// Total number of enemies killed
    pub kills: u32,
    /// Maximum depth reached
    pub max_depth: usize,
    /// Total number of items collected (picked up)
    pub items_collected: u32,
    /// Total number of turns survived
    pub turns_survived: u32,
    /// Number of bosses defeated
    pub bosses_defeated: u32,
    /// Total gold collected
    pub gold_collected: u32,
}

impl AchievementProgress {
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment kill count
    pub fn add_kill(&mut self) {
        self.kills = self.kills.saturating_add(1);
    }

    /// Update maximum depth reached
    pub fn update_depth(&mut self, depth: usize) {
        if depth > self.max_depth {
            self.max_depth = depth;
        }
    }

    /// Increment items collected
    pub fn add_item(&mut self) {
        self.items_collected = self.items_collected.saturating_add(1);
    }

    /// Update turns survived
    pub fn update_turns(&mut self, turns: u32) {
        self.turns_survived = turns;
    }

    /// Increment boss defeats
    pub fn add_boss_defeat(&mut self) {
        self.bosses_defeated = self.bosses_defeated.saturating_add(1);
    }

    /// Add gold collected
    pub fn add_gold(&mut self, amount: u32) {
        self.gold_collected = self.gold_collected.saturating_add(amount);
    }

    /// Reset all progress
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_kill() {
        let mut progress = AchievementProgress::new();
        assert_eq!(progress.kills, 0);

        progress.add_kill();
        assert_eq!(progress.kills, 1);

        progress.add_kill();
        assert_eq!(progress.kills, 2);
    }

    #[test]
    fn test_update_depth() {
        let mut progress = AchievementProgress::new();
        assert_eq!(progress.max_depth, 0);

        progress.update_depth(5);
        assert_eq!(progress.max_depth, 5);

        // Should not decrease
        progress.update_depth(3);
        assert_eq!(progress.max_depth, 5);

        // Should increase
        progress.update_depth(10);
        assert_eq!(progress.max_depth, 10);
    }

    #[test]
    fn test_add_item() {
        let mut progress = AchievementProgress::new();
        assert_eq!(progress.items_collected, 0);

        progress.add_item();
        assert_eq!(progress.items_collected, 1);

        for _ in 0..9 {
            progress.add_item();
        }
        assert_eq!(progress.items_collected, 10);
    }

    #[test]
    fn test_saturating_arithmetic() {
        let mut progress = AchievementProgress::new();
        progress.kills = u32::MAX;

        // Should not overflow
        progress.add_kill();
        assert_eq!(progress.kills, u32::MAX);
    }

    #[test]
    fn test_reset() {
        let mut progress = AchievementProgress::new();
        progress.add_kill();
        progress.update_depth(10);
        progress.add_item();

        progress.reset();

        assert_eq!(progress.kills, 0);
        assert_eq!(progress.max_depth, 0);
        assert_eq!(progress.items_collected, 0);
    }
}
