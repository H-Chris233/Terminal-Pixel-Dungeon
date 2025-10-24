//! Comprehensive tests for the achievements system

use crate::*;

#[test]
fn test_serialization_achievement() {
    let achievement = Achievement::new(
        AchievementId::FirstBlood,
        "First Blood",
        "Defeat your first enemy",
        AchievementCriteria::KillCount(1),
    );

    // Test JSON serialization
    let json = serde_json::to_string(&achievement).expect("Failed to serialize to JSON");
    let deserialized: Achievement =
        serde_json::from_str(&json).expect("Failed to deserialize from JSON");

    assert_eq!(achievement.id, deserialized.id);
    assert_eq!(achievement.name, deserialized.name);
    assert_eq!(achievement.description, deserialized.description);
    assert_eq!(achievement.unlocked, deserialized.unlocked);
}

#[test]
fn test_serialization_achievement_progress() {
    let mut progress = AchievementProgress::new();
    progress.add_kill();
    progress.add_kill();
    progress.update_depth(5);
    progress.add_item();
    progress.update_turns(100);
    progress.add_boss_defeat();
    progress.add_gold(500);

    // Test JSON serialization
    let json = serde_json::to_string(&progress).expect("Failed to serialize to JSON");
    let deserialized: AchievementProgress =
        serde_json::from_str(&json).expect("Failed to deserialize from JSON");

    assert_eq!(progress.kills, deserialized.kills);
    assert_eq!(progress.max_depth, deserialized.max_depth);
    assert_eq!(progress.items_collected, deserialized.items_collected);
    assert_eq!(progress.turns_survived, deserialized.turns_survived);
    assert_eq!(progress.bosses_defeated, deserialized.bosses_defeated);
    assert_eq!(progress.gold_collected, deserialized.gold_collected);
}

#[test]
fn test_serialization_achievements_manager() {
    let mut manager = AchievementsManager::new();

    // Unlock some achievements
    manager.on_kill();
    manager.on_level_change(5);
    manager.on_item_pickup();

    // Test JSON serialization
    let json = serde_json::to_string(&manager).expect("Failed to serialize to JSON");
    let deserialized: AchievementsManager =
        serde_json::from_str(&json).expect("Failed to deserialize from JSON");

    assert_eq!(manager.progress.kills, deserialized.progress.kills);
    assert_eq!(manager.progress.max_depth, deserialized.progress.max_depth);
    assert_eq!(
        manager.progress.items_collected,
        deserialized.progress.items_collected
    );

    // Check unlocked achievements match
    assert_eq!(
        manager.is_unlocked(AchievementId::FirstBlood),
        deserialized.is_unlocked(AchievementId::FirstBlood)
    );
    assert_eq!(
        manager.is_unlocked(AchievementId::DeepDiver),
        deserialized.is_unlocked(AchievementId::DeepDiver)
    );
}

#[test]
fn test_bincode_serialization_achievement() {
    let achievement = Achievement::new(
        AchievementId::SlayerII,
        "Slayer II",
        "Defeat 50 enemies",
        AchievementCriteria::KillCount(50),
    );

    // Test bincode serialization
    let encoded = bincode::encode_to_vec(&achievement, bincode::config::standard())
        .expect("Failed to serialize with bincode");
    let (decoded, _): (Achievement, _) = bincode::decode_from_slice(&encoded, bincode::config::standard())
        .expect("Failed to deserialize with bincode");

    assert_eq!(achievement.id, decoded.id);
    assert_eq!(achievement.name, decoded.name);
    assert_eq!(achievement.description, decoded.description);
    assert_eq!(achievement.unlocked, decoded.unlocked);
}

#[test]
fn test_bincode_serialization_progress() {
    let mut progress = AchievementProgress::new();
    progress.kills = 42;
    progress.max_depth = 15;
    progress.items_collected = 23;
    progress.turns_survived = 500;
    progress.bosses_defeated = 2;
    progress.gold_collected = 1500;

    // Test bincode serialization
    let encoded = bincode::encode_to_vec(&progress, bincode::config::standard())
        .expect("Failed to serialize with bincode");
    let (decoded, _): (AchievementProgress, _) = bincode::decode_from_slice(&encoded, bincode::config::standard())
        .expect("Failed to deserialize with bincode");

    assert_eq!(progress.kills, decoded.kills);
    assert_eq!(progress.max_depth, decoded.max_depth);
    assert_eq!(progress.items_collected, decoded.items_collected);
    assert_eq!(progress.turns_survived, decoded.turns_survived);
    assert_eq!(progress.bosses_defeated, decoded.bosses_defeated);
    assert_eq!(progress.gold_collected, decoded.gold_collected);
}

#[test]
fn test_bincode_serialization_manager() {
    let mut manager = AchievementsManager::new();

    // Set up some state
    for _ in 0..50 {
        manager.on_kill();
    }
    manager.on_level_change(10);
    manager.on_boss_defeat();

    // Clear newly_unlocked before serializing (as would happen in a real save scenario)
    manager.drain_newly_unlocked();

    // Test bincode serialization
    let encoded = bincode::encode_to_vec(&manager, bincode::config::standard())
        .expect("Failed to serialize with bincode");
    let (decoded, _): (AchievementsManager, _) = bincode::decode_from_slice(&encoded, bincode::config::standard())
        .expect("Failed to deserialize with bincode");

    // Verify progress
    assert_eq!(manager.progress.kills, decoded.progress.kills);
    assert_eq!(manager.progress.max_depth, decoded.progress.max_depth);
    assert_eq!(manager.progress.bosses_defeated, decoded.progress.bosses_defeated);

    // Verify unlocked achievements
    assert!(decoded.is_unlocked(AchievementId::FirstBlood));
    assert!(decoded.is_unlocked(AchievementId::SlayerI));
    assert!(decoded.is_unlocked(AchievementId::SlayerII));
    assert!(decoded.is_unlocked(AchievementId::Spelunker));
    assert!(decoded.is_unlocked(AchievementId::BossSlayer));

    // Newly unlocked should be empty since we drained it before serializing
    assert_eq!(decoded.peek_newly_unlocked().len(), 0);
}

#[test]
fn test_bincode_roundtrip_empty_manager() {
    let manager = AchievementsManager::new();

    let encoded = bincode::encode_to_vec(&manager, bincode::config::standard())
        .expect("Failed to serialize with bincode");
    let (decoded, _): (AchievementsManager, _) = bincode::decode_from_slice(&encoded, bincode::config::standard())
        .expect("Failed to deserialize with bincode");

    assert_eq!(manager.progress.kills, decoded.progress.kills);
    assert_eq!(manager.progress.max_depth, decoded.progress.max_depth);
    assert_eq!(manager.achievements.len(), decoded.achievements.len());
}

#[test]
fn test_achievement_criteria_variants() {
    let criteria_variants = vec![
        AchievementCriteria::KillCount(10),
        AchievementCriteria::DepthReached(5),
        AchievementCriteria::ItemsCollected(20),
        AchievementCriteria::TurnsSurvived(100),
        AchievementCriteria::BossDefeated,
        AchievementCriteria::GoldCollected(1000),
    ];

    for criteria in criteria_variants {
        // Test JSON serialization
        let json = serde_json::to_string(&criteria).expect("Failed to serialize criteria to JSON");
        let _: AchievementCriteria =
            serde_json::from_str(&json).expect("Failed to deserialize criteria from JSON");

        // Test bincode serialization
        let encoded = bincode::encode_to_vec(&criteria, bincode::config::standard())
            .expect("Failed to serialize criteria with bincode");
        let (_decoded, _): (AchievementCriteria, _) = bincode::decode_from_slice(&encoded, bincode::config::standard())
            .expect("Failed to deserialize criteria with bincode");
    }
}

#[test]
fn test_achievement_id_variants() {
    let ids = vec![
        AchievementId::FirstBlood,
        AchievementId::SlayerI,
        AchievementId::SlayerII,
        AchievementId::SlayerIII,
        AchievementId::BossSlayer,
        AchievementId::DeepDiver,
        AchievementId::Spelunker,
        AchievementId::MasterExplorer,
        AchievementId::Hoarder,
        AchievementId::Collector,
        AchievementId::TreasureHunter,
        AchievementId::Survivor,
        AchievementId::Veteran,
        AchievementId::Legend,
        AchievementId::Lucky,
        AchievementId::Wealthy,
    ];

    for id in ids {
        // Test JSON serialization
        let json = serde_json::to_string(&id).expect("Failed to serialize id to JSON");
        let deserialized: AchievementId =
            serde_json::from_str(&json).expect("Failed to deserialize id from JSON");
        assert_eq!(id, deserialized);

        // Test bincode serialization
        let encoded = bincode::encode_to_vec(&id, bincode::config::standard())
            .expect("Failed to serialize id with bincode");
        let (decoded, _): (AchievementId, _) = bincode::decode_from_slice(&encoded, bincode::config::standard())
            .expect("Failed to deserialize id with bincode");
        assert_eq!(id, decoded);
    }
}

#[test]
fn test_manager_register_definitions() {
    let manager = AchievementsManager::new();

    // Verify all achievements are registered
    assert!(manager.get_achievement(AchievementId::FirstBlood).is_some());
    assert!(manager.get_achievement(AchievementId::SlayerI).is_some());
    assert!(manager.get_achievement(AchievementId::DeepDiver).is_some());
    assert!(manager.get_achievement(AchievementId::Hoarder).is_some());
    assert!(manager.get_achievement(AchievementId::Survivor).is_some());

    // Verify achievement properties
    let first_blood = manager.get_achievement(AchievementId::FirstBlood).unwrap();
    assert_eq!(first_blood.name, "First Blood");
    assert_eq!(first_blood.description, "Defeat your first enemy");
    assert!(!first_blood.unlocked);
}

#[test]
fn test_manager_update_progress() {
    let mut manager = AchievementsManager::new();

    // Test kill updates
    assert_eq!(manager.progress().kills, 0);
    manager.on_kill();
    assert_eq!(manager.progress().kills, 1);

    // Test depth updates
    assert_eq!(manager.progress().max_depth, 0);
    manager.on_level_change(5);
    assert_eq!(manager.progress().max_depth, 5);

    // Test item updates
    assert_eq!(manager.progress().items_collected, 0);
    manager.on_item_pickup();
    assert_eq!(manager.progress().items_collected, 1);

    // Test turn updates
    assert_eq!(manager.progress().turns_survived, 0);
    manager.on_turn_end(50);
    assert_eq!(manager.progress().turns_survived, 50);

    // Test boss defeat updates
    assert_eq!(manager.progress().bosses_defeated, 0);
    manager.on_boss_defeat();
    assert_eq!(manager.progress().bosses_defeated, 1);

    // Test gold updates
    assert_eq!(manager.progress().gold_collected, 0);
    manager.on_gold_collected(100);
    assert_eq!(manager.progress().gold_collected, 100);
}

#[test]
fn test_manager_query_unlocked() {
    let mut manager = AchievementsManager::new();

    // Initially nothing should be unlocked
    assert_eq!(manager.unlocked_achievements().len(), 0);
    assert_eq!(manager.locked_achievements().len(), manager.achievements().len());

    // Unlock some achievements
    manager.on_kill();
    manager.on_level_change(5);

    // Now some should be unlocked
    assert!(manager.is_unlocked(AchievementId::FirstBlood));
    assert!(manager.is_unlocked(AchievementId::DeepDiver));
    assert!(!manager.is_unlocked(AchievementId::SlayerI));

    let unlocked = manager.unlocked_achievements();
    assert_eq!(unlocked.len(), 2);

    let locked = manager.locked_achievements();
    assert_eq!(locked.len(), manager.achievements().len() - 2);
}

#[test]
fn test_progress_tracking_accuracy() {
    let mut progress = AchievementProgress::new();

    // Test multiple operations
    for _ in 0..100 {
        progress.add_kill();
    }
    assert_eq!(progress.kills, 100);

    // Test depth only updates to maximum
    progress.update_depth(10);
    progress.update_depth(5);
    progress.update_depth(15);
    assert_eq!(progress.max_depth, 15);

    // Test gold accumulation
    progress.add_gold(100);
    progress.add_gold(250);
    progress.add_gold(650);
    assert_eq!(progress.gold_collected, 1000);
}

#[test]
fn test_achievement_tiers() {
    let mut manager = AchievementsManager::new();

    // Test slayer tiers
    manager.on_kill(); // 1 kill -> FirstBlood
    assert!(manager.is_unlocked(AchievementId::FirstBlood));
    assert!(!manager.is_unlocked(AchievementId::SlayerI));

    for _ in 0..9 {
        manager.on_kill();
    }
    // 10 kills -> SlayerI
    assert!(manager.is_unlocked(AchievementId::SlayerI));
    assert!(!manager.is_unlocked(AchievementId::SlayerII));

    for _ in 0..40 {
        manager.on_kill();
    }
    // 50 kills -> SlayerII
    assert!(manager.is_unlocked(AchievementId::SlayerII));
    assert!(!manager.is_unlocked(AchievementId::SlayerIII));

    for _ in 0..50 {
        manager.on_kill();
    }
    // 100 kills -> SlayerIII
    assert!(manager.is_unlocked(AchievementId::SlayerIII));
}

#[test]
fn test_multiple_criteria_independence() {
    let mut manager = AchievementsManager::new();

    // Test that different criteria types don't interfere
    manager.on_kill(); // Unlocks FirstBlood
    manager.on_level_change(5); // Unlocks DeepDiver
    manager.on_item_pickup(); // Just increments count
    manager.on_boss_defeat(); // Unlocks BossSlayer

    assert!(manager.is_unlocked(AchievementId::FirstBlood));
    assert!(manager.is_unlocked(AchievementId::DeepDiver));
    assert!(manager.is_unlocked(AchievementId::BossSlayer));
    assert!(!manager.is_unlocked(AchievementId::Hoarder)); // Needs 10 items
}

#[test]
fn test_event_driven_updates() {
    let mut manager = AchievementsManager::new();

    // Simulate event-driven updates
    let unlocked_1 = manager.on_kill();
    assert_eq!(unlocked_1.len(), 1);
    assert_eq!(unlocked_1[0], AchievementId::FirstBlood);

    let unlocked_2 = manager.on_level_change(5);
    assert_eq!(unlocked_2.len(), 1);
    assert_eq!(unlocked_2[0], AchievementId::DeepDiver);

    // No new unlocks
    let unlocked_3 = manager.on_kill();
    assert_eq!(unlocked_3.len(), 0);

    // Kill more enemies to reach SlayerI threshold
    // We have 2 kills so far, need 8 more to get to 10
    for _ in 0..7 {
        manager.on_kill();
    }
    // This should be the 10th kill, unlocking SlayerI
    let unlocked_4 = manager.on_kill();
    assert_eq!(unlocked_4.len(), 1);
    assert_eq!(unlocked_4[0], AchievementId::SlayerI);
}

#[test]
fn test_persistence_simulation() {
    // Simulate saving and loading achievements between sessions
    let mut manager1 = AchievementsManager::new();

    // Session 1: Play and unlock some achievements
    for _ in 0..25 {
        manager1.on_kill();
    }
    manager1.on_level_change(10);
    manager1.on_gold_collected(500);

    // Save to bincode
    let saved_data = bincode::encode_to_vec(&manager1, bincode::config::standard())
        .expect("Failed to save");

    // Session 2: Load and continue
    let (mut manager2, _): (AchievementsManager, _) = bincode::decode_from_slice(&saved_data, bincode::config::standard())
        .expect("Failed to load");

    // Verify progress was preserved
    assert_eq!(manager2.progress().kills, 25);
    assert_eq!(manager2.progress().max_depth, 10);
    assert_eq!(manager2.progress().gold_collected, 500);

    // Verify unlocked achievements were preserved
    assert!(manager2.is_unlocked(AchievementId::FirstBlood));
    assert!(manager2.is_unlocked(AchievementId::SlayerI));
    assert!(manager2.is_unlocked(AchievementId::Spelunker));

    // Continue playing
    for _ in 0..25 {
        manager2.on_kill();
    }
    assert!(manager2.is_unlocked(AchievementId::SlayerII));
}
