# Achievements Crate

A comprehensive achievements tracking system for Terminal Pixel Dungeon.

## Overview

This crate provides a complete achievements system that integrates with the game's event bus to track player progress and unlock achievements. It supports serialization via both JSON (serde) and binary (bincode) formats for save game persistence.

## Core Data Structures

### AchievementId

An enum representing unique identifiers for all achievements in the game:

- **Kill-based**: `FirstBlood`, `SlayerI`, `SlayerII`, `SlayerIII`, `BossSlayer`
- **Exploration**: `DeepDiver`, `Spelunker`, `MasterExplorer`
- **Collection**: `Hoarder`, `Collector`, `TreasureHunter`
- **Survival**: `Survivor`, `Veteran`, `Legend`
- **Miscellaneous**: `Lucky`, `Wealthy`

### AchievementCriteria

Defines the conditions required to unlock an achievement:

```rust
pub enum AchievementCriteria {
    KillCount(u32),           // Kill a certain number of enemies
    DepthReached(usize),      // Reach a certain depth
    ItemsCollected(u32),      // Collect a certain number of items
    TurnsSurvived(u32),       // Survive a certain number of turns
    BossDefeated,             // Defeat a boss
    GoldCollected(u32),       // Collect a certain amount of gold
}
```

### Achievement

Represents a single achievement with its metadata and unlock status:

```rust
pub struct Achievement {
    pub id: AchievementId,
    pub name: String,
    pub description: String,
    pub criteria: AchievementCriteria,
    pub unlocked: bool,
}
```

### AchievementProgress

Tracks the player's current progress toward all achievements:

```rust
pub struct AchievementProgress {
    pub kills: u32,
    pub max_depth: usize,
    pub items_collected: u32,
    pub turns_survived: u32,
    pub bosses_defeated: u32,
    pub gold_collected: u32,
}
```

## AchievementsManager

The main interface for managing achievements. It provides:

### Registration and Initialization

```rust
let manager = AchievementsManager::new();  // Auto-registers all achievements
```

### Event-Driven Updates

The manager provides event handlers for game events:

```rust
// Update progress and check for newly unlocked achievements
let unlocked = manager.on_kill();           // Enemy killed
let unlocked = manager.on_level_change(10); // Reached depth 10
let unlocked = manager.on_item_pickup();    // Item collected
let unlocked = manager.on_turn_end(100);    // Turn counter updated
let unlocked = manager.on_boss_defeat();    // Boss defeated
let unlocked = manager.on_gold_collected(500); // Gold collected
```

Each handler returns a `Vec<AchievementId>` containing newly unlocked achievements.

### Querying Achievements

```rust
// Check if an achievement is unlocked
if manager.is_unlocked(AchievementId::FirstBlood) {
    // ...
}

// Get all unlocked achievements
let unlocked = manager.unlocked_achievements();

// Get all locked achievements
let locked = manager.locked_achievements();

// Get a specific achievement
if let Some(achievement) = manager.get_achievement(AchievementId::SlayerI) {
    println!("{}: {}", achievement.name, achievement.description);
}

// Get unlock percentage (0.0 to 1.0)
let percentage = manager.unlock_percentage();
```

### Notification System

The manager tracks newly unlocked achievements for UI notifications:

```rust
// Peek at newly unlocked without clearing
let new_achievements = manager.peek_newly_unlocked();

// Get and clear newly unlocked
let new_achievements = manager.drain_newly_unlocked();
```

### Progress Access

```rust
// Get current progress (read-only)
let progress = manager.progress();
println!("Total kills: {}", progress.kills);

// Get mutable progress (for direct manipulation if needed)
let progress_mut = manager.progress_mut();
progress_mut.add_kill();
```

## Serialization Support

All core types support both serde (JSON) and bincode (binary) serialization:

```rust
// JSON serialization
let json = serde_json::to_string(&manager)?;
let manager: AchievementsManager = serde_json::from_str(&json)?;

// Bincode serialization (for save games)
let encoded = bincode::encode_to_vec(&manager, bincode::config::standard())?;
let (manager, _): (AchievementsManager, _) = 
    bincode::decode_from_slice(&encoded, bincode::config::standard())?;
```

**Note**: The `newly_unlocked` field is marked with `#[serde(skip_serializing)]` to avoid persisting transient notification state. Remember to call `drain_newly_unlocked()` before saving to ensure clean serialization.

## Achievement Tiers

Many achievements have multiple tiers (e.g., Slayer I/II/III). These are implemented as separate achievements with different criteria:

- **Slayer I**: 10 kills
- **Slayer II**: 50 kills
- **Slayer III**: 100 kills

All tiers can be unlocked simultaneously if criteria are met (e.g., reaching 100 kills unlocks all three Slayer achievements at once if they weren't previously unlocked).

## Integration with Event Bus

To integrate with the game's event bus:

```rust
// In your event handler
match event {
    GameEvent::EntityDied { .. } => {
        let unlocked = achievements_manager.on_kill();
        for achievement_id in unlocked {
            // Display notification
        }
    }
    GameEvent::LevelChanged { new_level } => {
        let unlocked = achievements_manager.on_level_change(new_level);
        // Handle notifications...
    }
    // ... other events
}
```

## Examples

### Basic Usage

```rust
use achievements::{AchievementsManager, AchievementId};

let mut manager = AchievementsManager::new();

// Simulate gameplay
for _ in 0..10 {
    let unlocked = manager.on_kill();
    for id in unlocked {
        println!("Achievement unlocked: {:?}", id);
    }
}
// Unlocks: FirstBlood, SlayerI

manager.on_level_change(5);
// Unlocks: DeepDiver

println!("Progress: {}%", manager.unlock_percentage() * 100.0);
```

### Save/Load Scenario

```rust
// Before saving
let mut manager = AchievementsManager::new();
// ... play game ...

// Clear notification queue before saving
manager.drain_newly_unlocked();

// Save
let save_data = bincode::encode_to_vec(&manager, bincode::config::standard())?;

// Later: Load
let (loaded_manager, _): (AchievementsManager, _) = 
    bincode::decode_from_slice(&save_data, bincode::config::standard())?;

// Continue playing with loaded manager
```

## Testing

The crate includes comprehensive unit tests covering:

- Serialization (both JSON and bincode)
- Progress tracking accuracy
- Achievement unlock logic
- Multi-tier achievements
- Event-driven updates
- Persistence simulation

Run tests with:

```bash
cargo test -p achievements
```

## Dependencies

- `serde` 1.0.219 - JSON serialization
- `bincode` 2.0.1 - Binary serialization
- `serde_json` 1.0.140 (dev) - Testing JSON serialization

## Future Extensions

Potential enhancements:

- Custom achievement definitions loaded from config files
- Time-based achievements (e.g., complete in under 30 minutes)
- Hidden achievements (revealed only when unlocked)
- Achievement rewards (unlock special items/abilities)
- Platform-specific achievement integration (Steam, etc.)
