# Turn System Refactor: Energy-Driven State Machine

## Overview

The turn system has been refactored from a simple player/AI alternation model into a comprehensive energy-driven state machine with explicit phases, centralized cost management, and detailed turn tracking.

## Architecture

### State Machine Phases

The turn system now operates through four explicit phases:

1. **Input** - Collecting input from player and AI entities
2. **IntentQueue** - Building a priority queue of action intents from entities with sufficient energy
3. **Resolution** - Resolving queued actions in priority order
4. **Aftermath** - Post-action cleanup: energy regeneration, status effect ticks, turn advancement

```
┌─────────┐
│  Input  │ ◄─────────────────────┐
└────┬────┘                       │
     │                            │
     ▼                            │
┌────────────┐                    │
│IntentQueue │                    │
└─────┬──────┘                    │
      │                           │
      ▼                           │
┌────────────┐                    │
│ Resolution │                    │
└─────┬──────┘                    │
      │                           │
      ▼                           │
┌───────────┐                     │
│ Aftermath │─────────────────────┘
└───────────┘
```

### Turn Metadata (`TurnMeta`)

Tracks comprehensive turn state:

```rust
pub struct TurnMeta {
    pub global_turn: u32,        // Full turn cycle counter
    pub sub_turn: u32,           // Action counter within current turn
    pub last_actor: Option<Entity>, // Last entity that took an action
    pub phase: TurnPhase,        // Current state machine phase
    pub legacy_state: TurnState, // Backward compatibility
}
```

**Key Operations:**
- `advance_sub_turn()` - Increments after each action
- `advance_global_turn()` - Increments after full turn cycle, resets sub_turn
- `set_last_actor(entity)` - Records which entity acted
- `set_phase(phase)` - Transitions between state machine phases

### Energy Cost Table

All action costs are centralized in `turn_system::energy_costs`:

```rust
pub const FULL_ACTION: u32 = 100;  // Standard action cost
pub const WAIT: u32 = 50;          // Wait action (half cost)
pub const FREE: u32 = 0;           // Menu/system actions

pub fn player_action_cost(action: &PlayerAction) -> u32;
pub fn ai_action_cost(action: &AIIntent) -> u32;
```

**Player Action Costs:**
- Move, Attack, UseItem, DropItem, Descend, Ascend → 100 energy
- Wait → 50 energy
- Menu actions (OpenInventory, CloseMenu, etc.) → 0 energy
- Quit → 0 energy

**AI Action Costs:**
- Move, Attack, Flee, UseSkill → 100 energy
- Wait → 50 energy

### Action Intent System

Actions are queued with priority for deterministic resolution:

```rust
pub struct ActionIntent {
    pub entity: Entity,
    pub action: Action,           // Player or AI action
    pub energy_cost: u32,
    pub priority: u32,            // Higher = acts first
}
```

**Priority Queue Ordering:**
1. Higher priority value acts first
2. On tie, lower energy cost acts first
3. Ensures player always acts before AI when both have sufficient energy

## Turn Processing Flow

### 1. Player Turn Phase

```
Input Phase → Player takes action → Energy consumed
    ↓
Sub-turn incremented → Last actor recorded
    ↓
If energy spent → Switch to AI Turn
```

### 2. AI Turn Phase

```
IntentQueue Phase → Collect AI entities with energy ≥ 100
    ↓
Resolution Phase → Each AI acts, energy consumed, sub-turn incremented
    ↓
Loop until player energy reaches maximum
    ↓
Aftermath Phase → Regenerate all energy
    ↓
Advance player progress (turn counter)
    ↓
Advance global turn → Sync to clock.turn_count
    ↓
Return to Player Turn
```

### 3. Energy Regeneration

**Timing:** After all AI actions complete (Aftermath phase)

**Process:**
```rust
for each entity with Energy component:
    energy.current = min(energy.current + energy.regeneration_rate, energy.max)
```

**Default Rates:**
- Player: 1 energy per turn (100 turns to full recharge)
- AI entities: 1 energy per turn

### 4. Player Progress Advancement

**Timing:** After energy regeneration, before global turn increment

**Process:**
```rust
if player entity exists:
    player_progress.advance_turn()  // Increments player_progress.turns
```

**Synced with:**
- `Resources.clock.turn_count` ← Updated from `TurnSystem.meta.global_turn`
- Event bus: `GameEvent::TurnEnded { turn: global_turn }`

## API Reference

### Core Methods

#### `consume_player_energy(world: &mut World, action: &PlayerAction) -> Result<()>`

Deducts energy for completed player actions. Updates sub-turn counter and records last actor.

**Preconditions:**
- Action must be in `completed_actions` buffer (not `pending_actions`)
- Prevents double-charging

**Side Effects:**
- Decrements player energy by action cost
- Sets `player_action_taken = true` if energy was consumed
- Increments `meta.sub_turn`
- Records `meta.last_actor`

#### `process_ai_turns(world: &mut World, resources: &mut Resources) -> Result<()>`

Processes all AI actions until player energy is full.

**Algorithm:**
1. Collect AI entities with energy ≥ FULL_ACTION
2. Each AI consumes 100 energy per action
3. Sub-turn incremented per action
4. Loop until no AI can act OR player energy == max
5. Switch state back to PlayerTurn

#### `process_turn_cycle(world: &mut World, resources: &mut Resources) -> Result<()>`

Main turn processing entry point. Handles state machine transitions.

**States:**
- `PlayerTurn` → Process player actions, switch to AITurn if action taken
- `AITurn` → Process AI actions, regenerate energy, advance turn, return to PlayerTurn

**Event Emission:**
Caller should emit events based on state transitions:
- `PlayerTurnStarted` when entering PlayerTurn
- `AITurnStarted` when entering AITurn  
- `TurnEnded { turn }` when global turn increments

#### `get_turn_order(world: &World) -> Vec<(Entity, u32)>`

Returns entities sorted by current energy (descending).

**Use Cases:**
- Debugging turn order
- UI display of action queue
- Priority visualization

### Helper Functions

#### `find_player(world: &World) -> Option<Entity>`

Locates player entity by Faction::Player in Actor component.

#### Energy Cost Functions

```rust
energy_costs::player_action_cost(action: &PlayerAction) -> u32
energy_costs::ai_action_cost(action: &AIIntent) -> u32
```

Centralized lookup for all action energy costs.

## Testing

### Unit Tests

Located in `src/turn_system.rs`:

```rust
test_energy_cost_lookup()              // Validates cost table
test_turn_meta_advancement()           // Sub-turn and global turn tracking
test_action_intent_ordering()          // Priority queue behavior
test_turn_system_initialization()      // Default state verification
test_energy_consumption()              // Player energy deduction
test_energy_regeneration()             // After-turn regen
test_turn_order_by_energy()           // Entity ordering
test_player_progress_advancement()    // Turn counter increment
test_full_turn_cycle()                // Complete player → AI → player flow
test_wait_action_costs_less()         // Wait costs 50 vs 100
```

### Integration Tests

Located in `tests/save_turn_state_integration.rs`:

```rust
test_save_and_restore_turn_state()    // Persistence of turn state
test_backward_compatibility_v1_saves() // Migration support
test_save_mid_combat_with_enemies()   // Combat state preservation
```

## Integration Points

### GameLoop Integration

```rust
// In game_loop.rs
turn_system.process_turn_cycle(&mut world, &mut resources)?;

// Sync clock
resources.clock.turn_count = turn_system.meta.global_turn;

// Emit events based on state changes
if turn_system.state != prev_state {
    match turn_system.state {
        TurnState::PlayerTurn => event_bus.publish(GameEvent::PlayerTurnStarted),
        TurnState::AITurn => event_bus.publish(GameEvent::AITurnStarted),
    }
}
```

### Save System Integration

`TurnMeta` state is persisted through:
- `turn_state.current_phase` → Maps TurnState to save format
- `clock_state.turn_count` → Stores `meta.global_turn`
- `player_progress.turns` → Player-specific turn counter

On load:
```rust
let (restored_turn_state, action_taken) = ecs_world.from_save_data(save_data)?;
turn_system.set_state(restored_turn_state, action_taken);
resources.clock.turn_count = save_data.clock_state.turn_count;
```

### Event Bus Integration

**Published Events:**
- `PlayerTurnStarted` - When switching to player input phase
- `AITurnStarted` - When AI begins processing
- `TurnEnded { turn: u32 }` - After global turn increments

**Event Timing:**
- Turn events emitted by GameLoop based on `TurnSystem.state` changes
- `TurnEnded` uses `meta.global_turn` value

## Migration Guide

### From Old System

**Before:**
```rust
// Energy costs scattered inline
energy.current = energy.current.saturating_sub(100); // Magic number
```

**After:**
```rust
// Centralized costs
let cost = energy_costs::player_action_cost(&action);
turn_system.consume_player_energy(&mut world, &action)?;
```

### Accessing Turn Metadata

**Before:**
```rust
resources.clock.turn_count  // Only global turn available
```

**After:**
```rust
let meta = turn_system.get_meta();
meta.global_turn    // Same as clock.turn_count (synced)
meta.sub_turn       // Actions within current turn
meta.last_actor     // Who acted last
meta.phase          // Current state machine phase
```

### Custom Action Costs

To add new actions with different costs:

1. Add variant to `PlayerAction` or `AIIntent`
2. Add case to `player_action_cost()` or `ai_action_cost()`
3. Document cost in this file
4. Add test case to `test_energy_cost_lookup()`

## Performance Considerations

### Energy Priority Queue

- Uses `BinaryHeap<ActionIntent>` for O(log n) insertions
- Currently populated but not actively used (reserved for future simultaneous action support)
- Ordering ensures deterministic resolution when needed

### Turn Order Calculation

- `get_turn_order()` creates temporary vector and sorts
- O(n log n) where n = entity count
- Called only when needed (UI display, debugging)
- Not on critical path during normal turn processing

### Energy Regeneration

- Linear scan of all Energy components: O(n)
- Happens once per complete turn cycle
- Minimal overhead: simple addition and min operation

## Future Enhancements

### Potential Features

1. **Simultaneous Actions**: Multiple entities acting in same sub-turn based on priority
2. **Variable Energy Costs**: Skills/abilities with custom energy requirements
3. **Energy Overflow**: Carry over excess energy to next turn
4. **Speed Modifiers**: Entities with different regeneration rates
5. **Action Interrupts**: Higher priority entities interrupting lower priority actions

### Extension Points

- `ActionIntent.priority` - Currently constant (1000 for player, 100 for AI)
- `build_intent_queue()` - Stub for future simultaneous action support
- `resolve_intents()` - Framework for priority-based resolution
- `energy.regeneration_rate` - Per-entity customization ready

## Backward Compatibility

### Legacy Support

- `TurnState` enum preserved for save compatibility
- `TurnMeta.legacy_state` synced with modern phases
- Old save format migration handled by save system
- Tests verify v1 → v2 migration path

### Breaking Changes

None. All existing code using `TurnSystem` continues to work:
- `is_player_turn()` / `is_ai_turn()` unchanged
- `process_turn_cycle()` signature unchanged
- `consume_player_energy()` signature unchanged
- Save/load interface unchanged

## References

- **Implementation**: `src/turn_system.rs`
- **Tests**: `src/turn_system.rs` (unit), `tests/save_turn_state_integration.rs` (integration)
- **Usage**: `src/game_loop.rs` (GameLoop integration)
- **Components**: `src/ecs.rs` (Energy, PlayerProgress, etc.)
