//! Achievements tracking system
//!
//! This module provides a complete achievements system that integrates with the game's
//! event bus to track player progress and unlock achievements.

pub mod achievement;
pub mod criteria;

#[cfg(test)]
mod tests;

pub use achievement::{Achievement, AchievementCriteria, AchievementId, all_achievements};
pub use criteria::AchievementProgress;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The main achievements manager that tracks progress and unlocks
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct AchievementsManager {
    /// All achievement definitions
    achievements: HashMap<AchievementId, Achievement>,
    /// Current progress toward all achievements
    progress: AchievementProgress,
    /// Newly unlocked achievements this session (for notifications)
    #[serde(default, skip_serializing)]
    newly_unlocked: Vec<AchievementId>,
}

impl AchievementsManager {
    /// Create a new achievements manager with all default achievements
    pub fn new() -> Self {
        let mut achievements = HashMap::new();
        for achievement in all_achievements() {
            achievements.insert(achievement.id, achievement);
        }

        Self {
            achievements,
            progress: AchievementProgress::new(),
            newly_unlocked: Vec::new(),
        }
    }

    /// Get current progress
    pub fn progress(&self) -> &AchievementProgress {
        &self.progress
    }

    /// Get mutable progress (for direct manipulation if needed)
    pub fn progress_mut(&mut self) -> &mut AchievementProgress {
        &mut self.progress
    }

    /// Get all achievements
    pub fn achievements(&self) -> &HashMap<AchievementId, Achievement> {
        &self.achievements
    }

    /// Get a specific achievement
    pub fn get_achievement(&self, id: AchievementId) -> Option<&Achievement> {
        self.achievements.get(&id)
    }

    /// Check if an achievement is unlocked
    pub fn is_unlocked(&self, id: AchievementId) -> bool {
        self.achievements
            .get(&id)
            .map(|a| a.unlocked)
            .unwrap_or(false)
    }

    /// Get all unlocked achievements
    pub fn unlocked_achievements(&self) -> Vec<&Achievement> {
        self.achievements.values().filter(|a| a.unlocked).collect()
    }

    /// Get all locked achievements
    pub fn locked_achievements(&self) -> Vec<&Achievement> {
        self.achievements.values().filter(|a| !a.unlocked).collect()
    }

    /// Get newly unlocked achievements since last check and clear the list
    pub fn drain_newly_unlocked(&mut self) -> Vec<AchievementId> {
        std::mem::take(&mut self.newly_unlocked)
    }

    /// Get newly unlocked achievements without clearing
    pub fn peek_newly_unlocked(&self) -> &[AchievementId] {
        &self.newly_unlocked
    }

    /// Check all achievements and unlock any that meet their criteria
    /// Returns the list of newly unlocked achievement IDs
    pub fn check_and_unlock(&mut self) -> Vec<AchievementId> {
        let mut unlocked = Vec::new();

        for (id, achievement) in self.achievements.iter_mut() {
            if !achievement.unlocked && achievement.check_unlock(&self.progress) {
                achievement.unlocked = true;
                unlocked.push(*id);
                self.newly_unlocked.push(*id);
            }
        }

        unlocked
    }

    /// Handle a kill event
    pub fn on_kill(&mut self) -> Vec<AchievementId> {
        self.progress.add_kill();
        self.check_and_unlock()
    }

    /// Handle a level change event
    pub fn on_level_change(&mut self, new_level: usize) -> Vec<AchievementId> {
        self.progress.update_depth(new_level);
        self.check_and_unlock()
    }

    /// Handle an item pickup event
    pub fn on_item_pickup(&mut self) -> Vec<AchievementId> {
        self.progress.add_item();
        self.check_and_unlock()
    }

    /// Handle a turn end event
    pub fn on_turn_end(&mut self, turn: u32) -> Vec<AchievementId> {
        self.progress.update_turns(turn);
        self.check_and_unlock()
    }

    /// Handle a boss defeat event
    pub fn on_boss_defeat(&mut self) -> Vec<AchievementId> {
        self.progress.add_boss_defeat();
        self.check_and_unlock()
    }

    /// Handle gold collection
    pub fn on_gold_collected(&mut self, amount: u32) -> Vec<AchievementId> {
        self.progress.add_gold(amount);
        self.check_and_unlock()
    }

    /// Reset all achievements and progress
    pub fn reset(&mut self) {
        self.progress.reset();
        for achievement in self.achievements.values_mut() {
            achievement.unlocked = false;
        }
        self.newly_unlocked.clear();
    }

    /// Get unlock percentage (0.0 to 1.0)
    pub fn unlock_percentage(&self) -> f32 {
        let total = self.achievements.len();
        if total == 0 {
            return 0.0;
        }
        let unlocked = self.unlocked_achievements().len();
        unlocked as f32 / total as f32
    }
}

impl Default for AchievementsManager {
    fn default() -> Self {
        Self::new()
    }
}
