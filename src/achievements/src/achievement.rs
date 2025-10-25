//! Achievement definitions and types

use serde::{Deserialize, Serialize};

/// Unique identifier for an achievement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum AchievementId {
    // Kill-based achievements
    FirstBlood,        // Kill your first enemy
    SlayerI,          // Kill 10 enemies
    SlayerII,         // Kill 50 enemies
    SlayerIII,        // Kill 100 enemies
    BossSlayer,       // Defeat a boss
    
    // Exploration achievements
    DeepDiver,        // Reach depth 5
    Spelunker,        // Reach depth 10
    MasterExplorer,   // Reach depth 20
    
    // Item collection achievements
    Hoarder,          // Collect 10 items
    Collector,        // Collect 50 items
    TreasureHunter,   // Collect 100 items
    
    // Survival achievements
    Survivor,         // Survive 100 turns
    Veteran,          // Survive 500 turns
    Legend,           // Survive 1000 turns
    
    // Miscellaneous achievements
    Lucky,            // Find a rare item
    Wealthy,          // Collect 1000 gold
}

/// Criteria required to unlock an achievement
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum AchievementCriteria {
    /// Kill a certain number of enemies
    KillCount(u32),
    /// Reach a certain depth
    DepthReached(usize),
    /// Collect a certain number of items
    ItemsCollected(u32),
    /// Survive a certain number of turns
    TurnsSurvived(u32),
    /// Defeat a boss
    BossDefeated,
    /// Collect a certain amount of gold
    GoldCollected(u32),
}

/// An achievement definition
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Achievement {
    pub id: AchievementId,
    pub name: String,
    pub description: String,
    pub criteria: AchievementCriteria,
    pub unlocked: bool,
}

impl Achievement {
    pub fn new(
        id: AchievementId,
        name: impl Into<String>,
        description: impl Into<String>,
        criteria: AchievementCriteria,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            criteria,
            unlocked: false,
        }
    }

    /// Check if this achievement is unlocked based on current progress
    pub fn check_unlock(&self, progress: &crate::criteria::AchievementProgress) -> bool {
        if self.unlocked {
            return false; // Already unlocked
        }

        match &self.criteria {
            AchievementCriteria::KillCount(required) => progress.kills >= *required,
            AchievementCriteria::DepthReached(required) => progress.max_depth >= *required,
            AchievementCriteria::ItemsCollected(required) => progress.items_collected >= *required,
            AchievementCriteria::TurnsSurvived(required) => progress.turns_survived >= *required,
            AchievementCriteria::BossDefeated => progress.bosses_defeated > 0,
            AchievementCriteria::GoldCollected(required) => progress.gold_collected >= *required,
        }
    }
}

/// Get all achievement definitions
pub fn all_achievements() -> Vec<Achievement> {
    vec![
        // Kill-based achievements
        Achievement::new(
            AchievementId::FirstBlood,
            "First Blood",
            "Defeat your first enemy",
            AchievementCriteria::KillCount(1),
        ),
        Achievement::new(
            AchievementId::SlayerI,
            "Slayer I",
            "Defeat 10 enemies",
            AchievementCriteria::KillCount(10),
        ),
        Achievement::new(
            AchievementId::SlayerII,
            "Slayer II",
            "Defeat 50 enemies",
            AchievementCriteria::KillCount(50),
        ),
        Achievement::new(
            AchievementId::SlayerIII,
            "Slayer III",
            "Defeat 100 enemies",
            AchievementCriteria::KillCount(100),
        ),
        Achievement::new(
            AchievementId::BossSlayer,
            "Boss Slayer",
            "Defeat a boss",
            AchievementCriteria::BossDefeated,
        ),
        // Exploration achievements
        Achievement::new(
            AchievementId::DeepDiver,
            "Deep Diver",
            "Reach depth 5",
            AchievementCriteria::DepthReached(5),
        ),
        Achievement::new(
            AchievementId::Spelunker,
            "Spelunker",
            "Reach depth 10",
            AchievementCriteria::DepthReached(10),
        ),
        Achievement::new(
            AchievementId::MasterExplorer,
            "Master Explorer",
            "Reach depth 20",
            AchievementCriteria::DepthReached(20),
        ),
        // Item collection achievements
        Achievement::new(
            AchievementId::Hoarder,
            "Hoarder",
            "Collect 10 items",
            AchievementCriteria::ItemsCollected(10),
        ),
        Achievement::new(
            AchievementId::Collector,
            "Collector",
            "Collect 50 items",
            AchievementCriteria::ItemsCollected(50),
        ),
        Achievement::new(
            AchievementId::TreasureHunter,
            "Treasure Hunter",
            "Collect 100 items",
            AchievementCriteria::ItemsCollected(100),
        ),
        // Survival achievements
        Achievement::new(
            AchievementId::Survivor,
            "Survivor",
            "Survive 100 turns",
            AchievementCriteria::TurnsSurvived(100),
        ),
        Achievement::new(
            AchievementId::Veteran,
            "Veteran",
            "Survive 500 turns",
            AchievementCriteria::TurnsSurvived(500),
        ),
        Achievement::new(
            AchievementId::Legend,
            "Legend",
            "Survive 1000 turns",
            AchievementCriteria::TurnsSurvived(1000),
        ),
        // Miscellaneous achievements
        Achievement::new(
            AchievementId::Wealthy,
            "Wealthy",
            "Collect 1000 gold",
            AchievementCriteria::GoldCollected(1000),
        ),
    ]
}
