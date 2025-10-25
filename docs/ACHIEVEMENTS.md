# Achievements System

## Overview

The achievements system provides a complete tracking and reward mechanism for player progress in Terminal Pixel Dungeon. It is fully integrated with the event bus and automatically tracks player actions without requiring manual intervention.

## Architecture

The achievements system is implemented as a standalone crate (`src/achievements`) that integrates with the game through the event bus. It consists of three main components:

1. **Achievement Definitions** (`achievement.rs`): Defines all available achievements, their criteria, and metadata
2. **Progress Tracking** (`criteria.rs`): Tracks player progress toward achievement goals
3. **Manager** (`lib.rs`): Coordinates tracking and unlocking of achievements

## Integration

### ECS Integration

The `AchievementsManager` is stored as part of the `Resources` struct in the ECS:

```rust
pub struct Resources {
    // ... other fields ...
    pub achievements: AchievementsManager,
}
```

### Event Bus Integration

The achievements system listens to game events via `ECSWorld::handle_achievement_event()`:

- **EntityDied**: Tracks enemy kills
- **LevelChanged**: Tracks dungeon depth reached
- **ItemPickedUp**: Tracks items collected
- **TurnEnded**: Tracks turns survived
- **BossDefeated**: Tracks boss defeats

When an achievement is unlocked, a `LogMessage` event is published with the achievement notification.

## Available Achievements

### Kill-Based Achievements
- **First Blood**: Defeat your first enemy (1 kill)
- **Slayer I**: Defeat 10 enemies
- **Slayer II**: Defeat 50 enemies
- **Slayer III**: Defeat 100 enemies
- **Boss Slayer**: Defeat a boss

### Exploration Achievements
- **Deep Diver**: Reach depth 5
- **Spelunker**: Reach depth 10
- **Master Explorer**: Reach depth 20

### Collection Achievements
- **Hoarder**: Collect 10 items
- **Collector**: Collect 50 items
- **Treasure Hunter**: Collect 100 items

### Survival Achievements
- **Survivor**: Survive 100 turns
- **Veteran**: Survive 500 turns
- **Legend**: Survive 1000 turns

### Miscellaneous Achievements
- **Wealthy**: Collect 1000 gold

## Event Emission

### LevelChanged Events

The `DungeonSystem` now emits `LevelChanged` events when the player uses stairs:

```rust
// When descending
ecs_world.publish_event(GameEvent::LevelChanged {
    old_level,
    new_level,
});
```

This is handled via `DungeonSystem::run_with_events()` which has access to the event bus.

### EntityDied Events

Already emitted by the `CombatSystem` when enemies are defeated.

### ItemPickedUp Events

Already emitted by the hero module's event handling.

### TurnEnded Events

Already emitted by the game loop's turn system.

### BossDefeated Events

Already defined in the event system and will be emitted by boss encounters.

## Usage

### Accessing Achievement Data

```rust
// Get all unlocked achievements
let unlocked = ecs_world.resources.achievements.unlocked_achievements();

// Check if specific achievement is unlocked
if ecs_world.resources.achievements.is_unlocked(AchievementId::FirstBlood) {
    // Achievement is unlocked
}

// Get unlock percentage
let percentage = ecs_world.resources.achievements.unlock_percentage();
```

### Getting Recent Unlocks

```rust
// Peek at newly unlocked achievements (without clearing)
let newly_unlocked = ecs_world.resources.achievements.peek_newly_unlocked();

// Drain newly unlocked achievements (clears the list)
let newly_unlocked = ecs_world.resources.achievements.drain_newly_unlocked();
```

### Getting Progress

```rust
// Access current progress
let progress = ecs_world.resources.achievements.progress();
println!("Kills: {}", progress.kills);
println!("Max Depth: {}", progress.max_depth);
println!("Items Collected: {}", progress.items_collected);
```

## Notification System

When an achievement is unlocked, a notification is automatically displayed to the player via the message log:

```
🏆 成就解锁: First Blood - Defeat your first enemy
```

The notification is published as a `LogMessage` event with `LogLevel::Info`.

## Testing

The achievements system includes comprehensive tests:

- **Unit Tests**: Test individual achievement criteria and progress tracking
- **Integration Tests**: Test achievement unlocking through event sequences
- **Progression Tests**: Test multi-level achievement progression

Run tests with:
```bash
cargo test -p achievements
```

All tests are in `src/achievements/src/lib.rs` under the `#[cfg(test)] mod tests` section.

## Serialization

The `AchievementsManager` and its components implement `Serialize` and `Deserialize`, allowing achievement progress to be saved and loaded with the game state.

```rust
// The manager automatically serializes with the Resources struct
// when the game is saved
```

## Future Enhancements

Potential future improvements:

1. **Hidden Achievements**: Add achievements that are not revealed until unlocked
2. **Achievement Categories**: Group achievements by type
3. **Achievement Rewards**: Tie achievements to gameplay rewards
4. **Steam Integration**: Export achievements to Steam API
5. **Statistics Tracking**: More detailed statistics beyond achievement criteria
6. **Rare Achievements**: Add low-percentage achievements for skilled players
7. **Challenge Achievements**: Add achievements for special challenges (no damage runs, speed runs, etc.)

## Code Organization

```
src/achievements/
├── Cargo.toml                  # Crate definition
└── src/
    ├── lib.rs                  # Main module with AchievementsManager
    ├── achievement.rs          # Achievement definitions and criteria
    └── criteria.rs             # Progress tracking
```

## Related Files

- `src/ecs.rs`: Integration point for achievements in ECS Resources
- `src/game_loop.rs`: Uses `DungeonSystem::run_with_events()`
- `src/systems.rs`: Includes `DungeonSystem::run_with_events()` for event emission
- `src/event_bus.rs`: Defines all game events

## Dependencies

The achievements crate has minimal dependencies:
- `serde`: For serialization/deserialization

It does not depend on other game crates, maintaining good separation of concerns.
