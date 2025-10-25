//! Basic usage example for the achievements system

use achievements::{AchievementsManager, AchievementId};

fn main() {
    println!("=== Terminal Pixel Dungeon Achievements Demo ===\n");

    // Create a new achievements manager
    let mut manager = AchievementsManager::new();
    
    println!("Total achievements: {}", manager.achievements().len());
    println!("Starting unlock percentage: {:.1}%\n", manager.unlock_percentage() * 100.0);

    // Simulate some gameplay events
    println!("--- Player kills their first enemy ---");
    let unlocked = manager.on_kill();
    display_unlocked(&manager, &unlocked);

    println!("\n--- Player kills 9 more enemies (10 total) ---");
    for _ in 0..9 {
        manager.on_kill();
    }
    display_progress(&manager);

    println!("\n--- Player reaches depth 5 ---");
    let unlocked = manager.on_level_change(5);
    display_unlocked(&manager, &unlocked);

    println!("\n--- Player collects 10 items ---");
    for _ in 0..10 {
        let unlocked = manager.on_item_pickup();
        if !unlocked.is_empty() {
            display_unlocked(&manager, &unlocked);
        }
    }

    println!("\n--- Player defeats a boss ---");
    let unlocked = manager.on_boss_defeat();
    display_unlocked(&manager, &unlocked);

    println!("\n--- Player collects 1000 gold ---");
    let unlocked = manager.on_gold_collected(1000);
    display_unlocked(&manager, &unlocked);

    println!("\n=== Final Statistics ===");
    display_progress(&manager);
    
    println!("\n=== All Unlocked Achievements ===");
    for achievement in manager.unlocked_achievements() {
        println!("✓ {}: {}", achievement.name, achievement.description);
    }

    println!("\n=== Remaining Achievements ===");
    for achievement in manager.locked_achievements() {
        println!("✗ {}: {}", achievement.name, achievement.description);
    }

    // Demonstrate serialization
    println!("\n=== Testing Serialization ===");
    let encoded = bincode::encode_to_vec(&manager, bincode::config::standard())
        .expect("Failed to serialize");
    println!("Serialized size: {} bytes", encoded.len());
    
    let (loaded, _): (AchievementsManager, _) = 
        bincode::decode_from_slice(&encoded, bincode::config::standard())
        .expect("Failed to deserialize");
    println!("Successfully loaded from save!");
    println!("Loaded achievements: {}/{}", 
        loaded.unlocked_achievements().len(),
        loaded.achievements().len()
    );
}

fn display_unlocked(manager: &AchievementsManager, unlocked: &[AchievementId]) {
    if unlocked.is_empty() {
        println!("No new achievements unlocked");
    } else {
        for id in unlocked {
            if let Some(achievement) = manager.get_achievement(*id) {
                println!("🏆 ACHIEVEMENT UNLOCKED: {}", achievement.name);
                println!("   {}", achievement.description);
            }
        }
    }
}

fn display_progress(manager: &AchievementsManager) {
    let progress = manager.progress();
    println!("Current Progress:");
    println!("  Enemies killed: {}", progress.kills);
    println!("  Max depth: {}", progress.max_depth);
    println!("  Items collected: {}", progress.items_collected);
    println!("  Turns survived: {}", progress.turns_survived);
    println!("  Bosses defeated: {}", progress.bosses_defeated);
    println!("  Gold collected: {}", progress.gold_collected);
    println!("  Achievements unlocked: {}/{} ({:.1}%)", 
        manager.unlocked_achievements().len(),
        manager.achievements().len(),
        manager.unlock_percentage() * 100.0
    );
}
