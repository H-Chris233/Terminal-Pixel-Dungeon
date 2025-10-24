//! Achievements tracking system
//! 
//! This module provides a complete achievements system that integrates with the game's
//! event bus to track player progress and unlock achievements.

pub mod achievement;
pub mod criteria;

pub use achievement::{Achievement, AchievementCriteria, AchievementId, all_achievements};
pub use criteria::AchievementProgress;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The main achievements manager that tracks progress and unlocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementsManager {
    /// All achievement definitions
    achievements: HashMap<AchievementId, Achievement>,
    /// Current progress toward all achievements
    progress: AchievementProgress,
    /// Newly unlocked achievements this session (for notifications)
    #[serde(skip)]
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
        self.achievements
            .values()
            .filter(|a| a.unlocked)
            .collect()
    }

    /// Get all locked achievements
    pub fn locked_achievements(&self) -> Vec<&Achievement> {
        self.achievements
            .values()
            .filter(|a| !a.unlocked)
            .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager() {
        let manager = AchievementsManager::new();
        assert!(manager.achievements.len() > 0);
        assert_eq!(manager.progress.kills, 0);
        assert_eq!(manager.newly_unlocked.len(), 0);
    }

    #[test]
    fn test_first_blood_achievement() {
        let mut manager = AchievementsManager::new();
        
        // First Blood should not be unlocked
        assert!(!manager.is_unlocked(AchievementId::FirstBlood));
        
        // Kill one enemy
        let unlocked = manager.on_kill();
        
        // First Blood should now be unlocked
        assert!(manager.is_unlocked(AchievementId::FirstBlood));
        assert_eq!(unlocked.len(), 1);
        assert_eq!(unlocked[0], AchievementId::FirstBlood);
    }

    #[test]
    fn test_slayer_progression() {
        let mut manager = AchievementsManager::new();
        
        // Kill 10 enemies
        for _ in 0..10 {
            manager.on_kill();
        }
        
        // FirstBlood and SlayerI should be unlocked
        assert!(manager.is_unlocked(AchievementId::FirstBlood));
        assert!(manager.is_unlocked(AchievementId::SlayerI));
        assert!(!manager.is_unlocked(AchievementId::SlayerII));
    }

    #[test]
    fn test_depth_achievements() {
        let mut manager = AchievementsManager::new();
        
        // Reach depth 5
        let unlocked = manager.on_level_change(5);
        
        assert!(manager.is_unlocked(AchievementId::DeepDiver));
        assert!(!manager.is_unlocked(AchievementId::Spelunker));
        assert!(unlocked.contains(&AchievementId::DeepDiver));
    }

    #[test]
    fn test_item_collection() {
        let mut manager = AchievementsManager::new();
        
        // Collect 10 items
        for _ in 0..10 {
            manager.on_item_pickup();
        }
        
        assert!(manager.is_unlocked(AchievementId::Hoarder));
        assert!(!manager.is_unlocked(AchievementId::Collector));
    }

    #[test]
    fn test_newly_unlocked() {
        let mut manager = AchievementsManager::new();
        
        // Kill one enemy
        manager.on_kill();
        
        // Should have one newly unlocked
        assert_eq!(manager.peek_newly_unlocked().len(), 1);
        
        // Drain should return and clear
        let unlocked = manager.drain_newly_unlocked();
        assert_eq!(unlocked.len(), 1);
        assert_eq!(manager.peek_newly_unlocked().len(), 0);
    }

    #[test]
    fn test_unlock_percentage() {
        let mut manager = AchievementsManager::new();
        let total = manager.achievements.len();
        
        // No achievements unlocked
        assert_eq!(manager.unlock_percentage(), 0.0);
        
        // Unlock one achievement
        manager.on_kill();
        let expected = 1.0 / total as f32;
        assert!((manager.unlock_percentage() - expected).abs() < 0.01);
    }

    #[test]
    fn test_reset() {
        let mut manager = AchievementsManager::new();
        
        // Unlock some achievements
        for _ in 0..10 {
            manager.on_kill();
        }
        manager.on_level_change(5);
        
        assert!(manager.unlock_percentage() > 0.0);
        
        // Reset
        manager.reset();
        
        assert_eq!(manager.unlock_percentage(), 0.0);
        assert_eq!(manager.progress.kills, 0);
        assert_eq!(manager.progress.max_depth, 0);
    }

    #[test]
    fn test_boss_defeat() {
        let mut manager = AchievementsManager::new();
        
        assert!(!manager.is_unlocked(AchievementId::BossSlayer));
        
        let unlocked = manager.on_boss_defeat();
        
        assert!(manager.is_unlocked(AchievementId::BossSlayer));
        assert!(unlocked.contains(&AchievementId::BossSlayer));
    }

    #[test]
    fn test_turn_survival() {
        let mut manager = AchievementsManager::new();
        
        // Survive 100 turns
        let unlocked = manager.on_turn_end(100);
        
        assert!(manager.is_unlocked(AchievementId::Survivor));
        assert!(!manager.is_unlocked(AchievementId::Veteran));
        assert!(unlocked.contains(&AchievementId::Survivor));
    }

    #[test]
    fn test_event_sequence_unlocks_multiple_achievements() {
        let mut manager = AchievementsManager::new();

        // Simulate a game session
        // Player kills 10 enemies (should unlock FirstBlood and SlayerI)
        for _ in 0..10 {
            manager.on_kill();
        }

        assert!(manager.is_unlocked(AchievementId::FirstBlood));
        assert!(manager.is_unlocked(AchievementId::SlayerI));
        assert!(!manager.is_unlocked(AchievementId::SlayerII));

        // Player reaches level 5 (should unlock DeepDiver)
        manager.on_level_change(5);
        assert!(manager.is_unlocked(AchievementId::DeepDiver));
        assert!(!manager.is_unlocked(AchievementId::Spelunker));

        // Player collects 10 items (should unlock Hoarder)
        for _ in 0..10 {
            manager.on_item_pickup();
        }
        assert!(manager.is_unlocked(AchievementId::Hoarder));

        // Player survives 100 turns (should unlock Survivor)
        manager.on_turn_end(100);
        assert!(manager.is_unlocked(AchievementId::Survivor));

        // Verify unlock percentage
        let total_unlocked = manager.unlocked_achievements().len();
        assert_eq!(total_unlocked, 5); // FirstBlood, SlayerI, DeepDiver, Hoarder, Survivor
    }

    #[test]
    fn test_level_progression() {
        let mut manager = AchievementsManager::new();

        // Level 5 -> unlock DeepDiver
        manager.on_level_change(5);
        assert!(manager.is_unlocked(AchievementId::DeepDiver));

        // Level 10 -> unlock Spelunker
        manager.on_level_change(10);
        assert!(manager.is_unlocked(AchievementId::Spelunker));

        // Level 20 -> unlock MasterExplorer
        manager.on_level_change(20);
        assert!(manager.is_unlocked(AchievementId::MasterExplorer));

        // All depth achievements should be unlocked
        assert!(manager.is_unlocked(AchievementId::DeepDiver));
        assert!(manager.is_unlocked(AchievementId::Spelunker));
        assert!(manager.is_unlocked(AchievementId::MasterExplorer));
    }

    #[test]
    fn test_notification_queue() {
        let mut manager = AchievementsManager::new();

        // Unlock first achievement
        manager.on_kill();
        let newly_unlocked = manager.peek_newly_unlocked();
        assert_eq!(newly_unlocked.len(), 1);
        assert_eq!(newly_unlocked[0], AchievementId::FirstBlood);

        // Drain should clear the queue
        let drained = manager.drain_newly_unlocked();
        assert_eq!(drained.len(), 1);
        assert_eq!(manager.peek_newly_unlocked().len(), 0);

        // Unlock multiple achievements at once
        for _ in 0..9 {
            manager.on_kill();
        }
        let newly_unlocked = manager.peek_newly_unlocked();
        assert_eq!(newly_unlocked.len(), 1); // SlayerI
        assert_eq!(newly_unlocked[0], AchievementId::SlayerI);
    }
}
