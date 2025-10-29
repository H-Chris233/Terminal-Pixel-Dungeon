# Changelog: Event Bus Taxonomy and Turn-Phase Integration

## Summary

Expanded the event bus system with categorized events, phase-aware processing, and priority-based event queues integrated with the turn system.

## Changes

### 1. Event Taxonomy (`src/event_bus.rs`)

#### New Enums

- **`EventCategory`**: Categorizes events into Combat, Movement, Status, Items, AI, Environment, UI, System, TurnPhase, and Action
- **`TurnPhase`**: Defines turn phases (Input, IntentQueue, Resolution, Aftermath, Any)

#### New Event Variants

**Action Intent/Result Events (4 new)**
- `ActionIntended` - Action queued
- `ActionCompleted` - Action completed with success flag
- `ActionFailed` - Action failed with reason
- `ActionCancelled` - Action cancelled

**Advanced Combat Outcomes (7 new)**
- `CombatBlocked` - Attack blocked
- `CombatParried` - Attack parried with counter
- `CombatDodged` - Attack dodged completely
- `CombatGrazed` - Partial damage attack
- `CombatLifesteal` - Lifesteal healing
- `CombatReflected` - Reflected damage
- `CombatShieldAbsorbed` - Shield absorption

**Status Lifecycle Events (6 new)**
- `StatusStacked` - Status intensity increased
- `StatusRefreshed` - Duration refreshed
- `StatusResisted` - Resisted application
- `StatusImmune` - Immune to effect
- `StatusTransferred` - Transferred between entities
- `StatusSpread` - Spread to multiple targets

**Environment Triggers (8 new)**
- `DoorOpened` / `DoorClosed` - Door interactions
- `SecretDiscovered` - Hidden secrets
- `ChestOpened` - Chest looting
- `ShrineActivated` - Shrine interaction
- `TrapDisarmed` - Trap disarming
- `TerrainChanged` - Terrain transformation
- `ExplosionTriggered` - Explosion effects

**UI Cues (5 new)**
- `UINotification` - General notifications
- `UIAlert` - Important alerts
- `TooltipRequested` - Tooltip display
- `HighlightRequested` - Entity/position highlighting
- `AnimationRequested` - Animation playback

**Total**: 30 new event variants

### 2. Event Bus Infrastructure

#### Priority Event Queue System

- **`PriorityEventEntry`**: Wraps events with priority and sequence number
- **Phase Queues**: `HashMap<TurnPhase, BinaryHeap<PriorityEventEntry>>`
- **Event Ordering**: Priority → Sequence (FIFO within same priority)

#### Recursive Publishing Protection

- **Publish Depth Tracking**: Limits recursion to 10 levels by default
- **Batch Buffer**: Collects events published beyond max depth
- **Automatic Flush**: Processes buffered events when depth returns to 0

#### Phase-Aware Processing

- **`set_current_phase()`**: Track current turn phase
- **`publish_to_phase()`**: Publish to specific phase with priority
- **`drain_phase()`**: Drain events for specific phase
- **`process_phase_events()`**: Process all events for a phase

### 3. Handler Enhancements

#### Phase-Aware Handler Trait

```rust
pub trait EventHandler {
    // Existing methods...
    
    /// NEW: Declare which phases handler should run in
    fn run_in_phases(&self) -> Vec<TurnPhase> {
        vec![TurnPhase::Any]
    }
}
```

#### Registration Methods

- **`subscribe_for_phase()`**: Register handler for specific phase
- **`subscribe_with_phases()`**: Register using handler's phase declaration
- **Phase Handlers Map**: `HashMap<TurnPhase, Vec<HandlerEntry>>`

### 4. Event Categorization

#### `GameEvent` Methods

- **`category(&self) -> EventCategory`**: Returns event's category
- **`event_type(&self) -> &'static str`**: Updated for all 30 new events

### 5. ECS Integration (`src/ecs.rs`)

#### Enhanced `handle_core_event()`

- Added handlers for all 30 new event types
- Graceful handling of unknown events (backward compatibility)
- Comprehensive message log updates
- No panic on legacy saves

### 6. Testing

#### New Test Suite (`tests/event_bus_taxonomy_tests.rs`)

12 comprehensive tests covering:
- ✅ Event categorization (all 30 new events)
- ✅ Priority ordering within phases
- ✅ Phase-aware event queues
- ✅ Phase-aware handler execution
- ✅ Middleware short-circuit behavior
- ✅ Recursive publishing protection
- ✅ FIFO ordering within same priority
- ✅ Current phase tracking
- ✅ Event history with new events
- ✅ Backward compatibility
- ✅ Multiple phase queues
- ✅ Full turn cycle integration

All 12 tests pass ✓

### 7. Documentation

#### New Documentation Files

- **`docs/EVENT_BUS_TAXONOMY.md`**: Comprehensive guide
  - Event categories overview
  - Turn phase integration
  - Phase-aware handlers
  - API reference
  - Migration guide
  - Performance considerations
  - Example usage

## API Changes

### Backward Compatible

All existing APIs continue to work unchanged:
- `event_bus.publish(event)` ✓
- `event_bus.drain()` ✓
- `event_bus.subscribe_all(handler)` ✓
- Existing event handlers ✓

### New APIs

```rust
// Phase-aware publishing
event_bus.publish_to_phase(event, Priority::High, TurnPhase::Resolution);

// Phase management
event_bus.set_current_phase(TurnPhase::Resolution);
let phase = event_bus.get_current_phase();

// Phase-specific draining
let events = event_bus.drain_phase(TurnPhase::Resolution);

// Phase-aware handlers
event_bus.subscribe_for_phase(TurnPhase::Resolution, handler);
event_bus.process_phase_events(TurnPhase::Resolution);

// Event categorization
let category = event.category(); // EventCategory::Combat
```

## Performance Impact

- **Priority Queue**: O(log n) operations for insertion/extraction
- **Phase Separation**: Only processes relevant events per phase
- **No Overhead**: Traditional mode has zero overhead
- **Memory**: Minimal - one BinaryHeap per phase

## Migration Guide

### No Changes Required

Existing code continues to work without modification.

### To Use New Features

1. **Use categorized events**:
   ```rust
   if event.category() == EventCategory::Combat {
       // Handle combat events
   }
   ```

2. **Phase-aware publishing**:
   ```rust
   event_bus.publish_to_phase(event, Priority::High, phase);
   ```

3. **Phase-aware handlers**:
   ```rust
   impl EventHandler for MyHandler {
       fn run_in_phases(&self) -> Vec<TurnPhase> {
           vec![TurnPhase::Resolution]
       }
   }
   ```

## Testing Results

```
Running tests/event_bus_taxonomy_tests.rs
running 12 tests
test test_backward_compatibility ... ok
test test_current_phase_tracking ... ok
test test_event_categories ... ok
test test_event_history_with_new_events ... ok
test test_fifo_ordering_same_priority ... ok
test test_full_turn_cycle_event_flow ... ok
test test_multiple_phase_queues ... ok
test test_middleware_short_circuit ... ok
test test_phase_aware_event_queues ... ok
test test_phase_aware_handlers ... ok
test test_priority_event_ordering ... ok
test test_recursive_publishing_protection ... ok

test result: ok. 12 passed; 0 failed
```

All existing tests also pass (56 lib tests + workspace tests).

## Benefits

1. **Better Organization**: Events grouped by category
2. **Deterministic Processing**: Priority and phase-based ordering
3. **Flexible Handlers**: Handlers can target specific phases
4. **Recursion Safety**: Protection against infinite event loops
5. **Rich Event Set**: 30 new event types for detailed interactions
6. **Backward Compatible**: No breaking changes
7. **Well Tested**: Comprehensive test coverage
8. **Documented**: Complete documentation and examples

## Future Enhancements

Potential areas for future work:
- Event filtering by category
- Event recording and replay
- Async event processing
- Event compression/merging
- Event analytics and metrics

## Authors

Enhanced by Claude (Anthropic) as part of the Terminal Pixel Dungeon development.

## Date

December 2024
