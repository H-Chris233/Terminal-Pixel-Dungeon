# AI Decision Making with Energy-Driven Turns

## Overview

This document describes the integration of AI decision making with the energy-driven turn system in Terminal Pixel Dungeon. The implementation follows a split intent-generation and execution model that respects energy costs and emits events for logging and UI feedback.

## Architecture

### Intent Generation vs. Execution

The AI system is now split into two distinct phases:

1. **Intent Generation Phase** (`generate_intent`)
   - Evaluates current world state (positions, status effects, hunger)
   - Considers AI type (Aggressive, Passive, Neutral, Patrol)
   - Considers AI state (Idle, Chasing, Fleeing, Patrolling, Attacking)
   - Generates an `AIActionIntent` with context (reason, target, position)
   - **No mutations** - reads world state without modifying it

2. **Execution Phase** (`execute_intent`)
   - Takes the generated intent and executes it
   - Updates entity positions, states, and targets
   - Consumes energy based on action type
   - Emits events via the event bus
   - **Mutations only** - modifies world state based on intent

### Energy Integration

AI actions respect the centralized energy cost system:

- **FULL_ACTION (100 energy)**: Move, Attack, Flee, UseSkill
- **WAIT (50 energy)**: Wait action when target out of range or impaired
- **FREE (0 energy)**: No free actions for AI currently

AI entities are only processed when they have sufficient energy (`current >= FULL_ACTION`). After taking an action, energy is deducted in `execute_intent`.

### World State Snapshots

The AI system reads fresh world state at the beginning of each AI turn:

```rust
// Get fresh world state snapshot
let player_positions: Vec<(Entity, Position, Option<u32>)> = world
    .world
    .query::<(&Position, &Player)>()
    .iter()
    .map(|(entity, (pos, _))| {
        let hp = world.world.get::<&Stats>(entity).map(|s| s.hp).ok();
        (entity, pos.clone(), hp)
    })
    .collect();
```

This ensures AI decisions are based on the latest game state, including:
- Player positions and health
- Entity positions
- Status effects
- Energy levels

### AI Behaviors

#### Aggressive AI

1. Checks for status impairments (Paralysis, Frost, Rooted)
2. If impaired, generates `Wait` intent
3. Finds closest player within range
4. If player is adjacent (distance <= 1.5), generates `Attack` intent
5. If player is in range but not adjacent, generates `Move` intent (chasing)
6. If no player in range, generates `Wait` intent

#### Passive AI

Always generates `Wait` intent. Never moves or attacks.

#### Neutral AI

1. Checks if AI has a target (has been provoked)
2. If provoked, acts like Aggressive AI
3. If not provoked, generates `Wait` intent

#### Patrol AI

1. If patrol path is empty, generates `Wait` intent
2. Finds closest patrol point
3. If already at patrol point, generates `Wait` intent
4. Otherwise, generates `Patrol` intent with target position
5. Moves towards the next patrol point

### AI State Machine

States are updated based on the intent being executed:

- `Idle`: Default state, no current action
- `Patrolling`: Following patrol path
- `Chasing`: Moving towards a target
- `Fleeing`: Moving away from a threat
- `Attacking`: In combat with a target

Transitions:
```rust
AIIntentType::Attack(_) => AIState::Attacking
AIIntentType::Move(_) (Aggressive) => AIState::Chasing
AIIntentType::Flee(_) => AIState::Fleeing
AIIntentType::Patrol(_) => AIState::Patrolling
AIIntentType::Wait => AIState::Idle (unless already Patrolling)
```

### Event Bus Integration

The AI system emits two types of events:

#### AIDecisionMade

Emitted for every AI decision with a human-readable reason:

```rust
GameEvent::AIDecisionMade {
    entity: ai_entity.id(),
    decision: "Chasing target at distance 3.2".to_string(),
}
```

#### AITargetChanged

Emitted when an AI's target entity changes:

```rust
GameEvent::AITargetChanged {
    entity: ai_entity.id(),
    old_target: Some(1234),
    new_target: Some(5678),
}
```

These events enable:
- Debug logging
- UI feedback ("The goblin spots you!")
- Statistics tracking
- Replay systems

## Implementation Details

### Handling Borrowing Issues

When reading and writing Position components, we must be careful about borrows:

```rust
// WRONG: Holds immutable borrow while trying to mutably borrow
if let Ok(current_pos) = world.get::<&Position>(entity) {
    // Still holding borrow here...
    if let Ok(mut pos) = world.get::<&mut Position>(entity) {
        // ERROR: Already borrowed!
    }
}

// CORRECT: Clone data and release borrow
let (old_pos, new_pos) = {
    if let Ok(current_pos) = world.get::<&Position>(entity) {
        let old = Position::new(current_pos.x, current_pos.y, current_pos.z);
        let new = Position::new(current_pos.x + dx, current_pos.y + dy, current_pos.z);
        (Some(old), Some(new))
    } else {
        (None, None)
    }
}; // Borrow released here

// Now we can mutably borrow
if let (Some(old_pos), Some(new_pos)) = (old_pos, new_pos) {
    if let Ok(mut pos) = world.get::<&mut Position>(entity) {
        *pos = new_pos;
    }
}
```

### Integration with Turn System

The turn system's `process_ai_turns` method now only checks if the player has enough energy to act again and if any AI can still act. The actual AI decision-making and energy consumption happens in `AISystem::run_with_events`, which is called from the game loop during AI turns.

### Direction Enum

Added `PartialEq` and `Eq` derives to the `Direction` enum to support pattern matching and comparisons in AI intent execution.

## Testing

### Test Suite

Comprehensive test coverage in `tests/ai_energy_integration_test.rs`:

1. **test_multi_enemy_turn_ordering**: Verifies multiple enemies take turns based on energy
2. **test_ai_waiting_when_no_target**: Ensures AI waits when targets are out of range
3. **test_ai_energy_regeneration**: Verifies energy regenerates correctly
4. **test_ai_impaired_by_status_effects**: Tests AI behavior under status effects
5. **test_ai_passive_behavior**: Validates passive AI never moves
6. **test_ai_neutral_becomes_aggressive**: Tests neutral AI provocation
7. **test_ai_patrol_behavior**: Validates patrol path following
8. **test_ai_decision_events_emitted**: Ensures events are emitted
9. **test_ai_state_transitions**: Tests state machine transitions
10. **test_multiple_ai_actors_take_turns**: Verifies multiple AI actors act in order
11. **test_ai_respects_energy_costs**: Ensures energy is consumed correctly

All tests pass ✓

### Helper Functions

Test helpers for creating test worlds and spawning enemies:

```rust
fn setup_test_world() -> ECSWorld {
    // Creates world with player and floor tiles
}

fn spawn_enemy(world: &mut World, x: i32, y: i32, name: &str, 
               energy: u32, ai_type: AIType) -> Entity {
    // Spawns enemy with specified parameters
}
```

## Future Enhancements

Potential improvements to the AI system:

1. **Pathfinding**: Implement A* or similar for smarter navigation around obstacles
2. **Group AI**: Coordinate multiple AI entities for flanking, formations
3. **Skill Usage**: Expand `UseSkill` intent with actual skill selection logic
4. **Flee Behavior**: Implement proper flee behavior when health is low
5. **Dynamic Difficulty**: Adjust AI aggression based on player performance
6. **Learning AI**: AI that adapts to player strategies
7. **Environmental Awareness**: Use terrain for cover, traps, etc.
8. **Communication**: AI entities sharing information about player location

## Code Locations

- **AI System**: `src/systems.rs` (lines 185-709)
- **Turn System Integration**: `src/turn_system.rs` (lines 327-365)
- **Game Loop Integration**: `src/game_loop.rs` (lines 521-534)
- **AI Components**: `src/ecs.rs` (lines 1015-1050)
- **Tests**: `tests/ai_energy_integration_test.rs`
- **Event Definitions**: `src/event_bus.rs` (lines 91-98)

## Migration Notes

For existing save games, the AI system changes should be backwards compatible since:
- AI component structure hasn't changed
- Energy component structure hasn't changed
- Only the execution logic has been refactored

However, AI behavior may differ slightly:
- AI now waits when targets are out of range (instead of moving randomly)
- AI respects status effects for impairment
- Energy consumption is more precise and consistent
