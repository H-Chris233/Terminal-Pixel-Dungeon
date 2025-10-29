# Event Bus Taxonomy and Turn-Phase Integration

## Overview

This document describes the enhanced event bus system that supports categorized events, phase-aware processing, and priority-based event queues integrated with the turn system.

## Event Categories

Events are now organized into the following categories for better filtering and processing:

### `EventCategory` Enum

```rust
pub enum EventCategory {
    Combat,      // Combat-related events
    Movement,    // Entity movement
    Status,      // Status effects lifecycle
    Items,       // Item interactions
    AI,          // AI decisions
    Environment, // Environmental interactions
    UI,          // UI notifications and alerts
    System,      // System-level events
    TurnPhase,   // Turn phase transitions
    Action,      // Action intent and results
}
```

Every `GameEvent` now implements a `category()` method that returns its category.

## New Event Types

### Action Intent/Result Events

- `ActionIntended` - When an action is queued
- `ActionCompleted` - When an action succeeds or fails
- `ActionFailed` - When an action fails with a reason
- `ActionCancelled` - When an action is cancelled

### Advanced Combat Outcomes

- `CombatBlocked` - Attack blocked by shield/defense
- `CombatParried` - Attack parried with counter damage
- `CombatDodged` - Attack completely dodged
- `CombatGrazed` - Partial damage attack
- `CombatLifesteal` - Lifesteal healing
- `CombatReflected` - Damage reflected back
- `CombatShieldAbsorbed` - Shield absorbs damage

### Status Lifecycle Events

- `StatusStacked` - Status effect intensity increased
- `StatusRefreshed` - Status effect duration refreshed
- `StatusResisted` - Entity resists status effect
- `StatusImmune` - Entity is immune to status effect
- `StatusTransferred` - Status transferred to another entity
- `StatusSpread` - Status spreads to multiple entities

### Environment Triggers

- `DoorOpened` / `DoorClosed` - Door interactions
- `SecretDiscovered` - Hidden secrets found
- `ChestOpened` - Chest looted
- `ShrineActivated` - Shrine interaction
- `TrapDisarmed` - Trap successfully disarmed
- `TerrainChanged` - Terrain transformation
- `ExplosionTriggered` - Explosion effect

### UI Cues

- `UINotification` - General UI notification
- `UIAlert` - Important UI alert
- `TooltipRequested` - Request for tooltip display
- `HighlightRequested` - Request to highlight entities/positions
- `AnimationRequested` - Request for animation playback

## Turn Phase Integration

### `TurnPhase` Enum

```rust
pub enum TurnPhase {
    Input,       // Collecting player/AI input
    IntentQueue, // Building action queue
    Resolution,  // Executing actions
    Aftermath,   // Post-action cleanup (energy, status effects)
    Any,         // Handler runs in all phases
}
```

### Phase-Aware Event Processing

Events can now be published to specific turn phases with priorities:

```rust
event_bus.publish_to_phase(
    GameEvent::DamageDealt { ... },
    Priority::High,
    TurnPhase::Resolution,
);
```

Events are processed in order of:
1. **Turn Phase** - Events are grouped by phase
2. **Priority** - Within a phase, higher priority events process first
3. **Sequence** - Same priority events process FIFO

### Priority Levels

```rust
pub enum Priority {
    Critical = 0,  // Crash handling, emergency saves
    High = 1,      // Core game logic (combat, movement)
    Normal = 2,    // General features (default)
    Low = 3,       // UI updates, sounds
    Lowest = 4,    // Logging, statistics
}
```

## Phase-Aware Handlers

Handlers can now declare which turn phases they should run in:

```rust
impl EventHandler for MyHandler {
    fn run_in_phases(&self) -> Vec<TurnPhase> {
        vec![TurnPhase::Resolution, TurnPhase::Aftermath]
    }
    
    // ... other methods
}
```

Register phase-specific handlers:

```rust
event_bus.subscribe_for_phase(
    TurnPhase::Resolution,
    Box::new(CombatResolutionHandler::new())
);
```

## Recursive Publishing Protection

The event bus now protects against infinite recursion:

- **Maximum Depth**: Default 10 levels of nested event publishing
- **Batch Buffer**: Events published beyond max depth are buffered
- **Automatic Flush**: Buffer is flushed when recursion depth returns to 0

This prevents handlers from creating infinite event loops while still allowing reasonable event chains.

## API Reference

### Publishing Events

```rust
// Traditional publishing (backward compatible)
event_bus.publish(event);

// Delayed publishing (next frame)
event_bus.publish_delayed(event);

// Phase-aware publishing
event_bus.publish_to_phase(event, priority, phase);
```

### Processing Events

```rust
// Traditional drain (all events)
for event in event_bus.drain() {
    // process event
}

// Phase-specific drain
let events = event_bus.drain_phase(TurnPhase::Resolution);

// Process all events for a phase (with handlers)
event_bus.process_phase_events(TurnPhase::Resolution);
```

### Handler Registration

```rust
// Global handler (all events, all phases)
event_bus.subscribe_all(Box::new(handler));

// Type-specific handler
event_bus.subscribe("DamageDealt", Box::new(handler));

// Phase-specific handler
event_bus.subscribe_for_phase(TurnPhase::Resolution, Box::new(handler));
```

## Integration with Turn System

The turn system now leverages phase-aware event processing:

```rust
// Set current phase
event_bus.set_current_phase(TurnPhase::Input);

// Publish phase-specific events
event_bus.publish_to_phase(
    GameEvent::ActionIntended { ... },
    Priority::Normal,
    TurnPhase::Input,
);

// Process events for current phase
event_bus.process_phase_events(TurnPhase::Input);

// Advance to next phase
event_bus.set_current_phase(TurnPhase::Resolution);
```

## Backward Compatibility

All existing functionality is preserved:

- ✅ Traditional `publish()` and `drain()` still work
- ✅ Existing handlers continue to function
- ✅ Legacy saves load without errors
- ✅ New events are handled gracefully in `handle_core_event()`

## Testing

Comprehensive test suite in `tests/event_bus_taxonomy_tests.rs`:

- ✅ Event categorization
- ✅ Priority ordering within phases
- ✅ Phase-aware event queues
- ✅ Phase-aware handler execution
- ✅ Middleware short-circuit behavior
- ✅ Recursive publishing protection
- ✅ FIFO ordering within same priority
- ✅ Full turn cycle integration
- ✅ Backward compatibility

## Example: Combat Resolution

```rust
// Phase 1: Intent Queue
event_bus.set_current_phase(TurnPhase::IntentQueue);
event_bus.publish_to_phase(
    GameEvent::ActionIntended {
        entity: attacker_id,
        action_type: "attack".to_string(),
        priority: 100,
    },
    Priority::High,
    TurnPhase::IntentQueue,
);

// Phase 2: Resolution
event_bus.set_current_phase(TurnPhase::Resolution);

// High priority: Core combat
event_bus.publish_to_phase(
    GameEvent::DamageDealt { ... },
    Priority::Critical,
    TurnPhase::Resolution,
);

// Normal priority: Status effects
event_bus.publish_to_phase(
    GameEvent::StatusApplied { ... },
    Priority::Normal,
    TurnPhase::Resolution,
);

// Low priority: UI notifications
event_bus.publish_to_phase(
    GameEvent::UINotification { ... },
    Priority::Low,
    TurnPhase::Resolution,
);

// Process all resolution events in priority order
event_bus.process_phase_events(TurnPhase::Resolution);
// Order: Critical -> High -> Normal -> Low

// Phase 3: Aftermath
event_bus.set_current_phase(TurnPhase::Aftermath);
event_bus.publish_to_phase(
    GameEvent::StatusEffectTicked { ... },
    Priority::Normal,
    TurnPhase::Aftermath,
);
event_bus.process_phase_events(TurnPhase::Aftermath);
```

## Migration Guide

### For Existing Code

No changes required! Existing code will continue to work:

```rust
// This still works
event_bus.publish(GameEvent::DamageDealt { ... });
event_bus.subscribe_all(Box::new(handler));
```

### To Use New Features

1. **Categorize your events**: Use `event.category()` to filter
2. **Use phase-aware publishing**: Call `publish_to_phase()` for better control
3. **Implement phase-aware handlers**: Override `run_in_phases()`
4. **Set priorities explicitly**: Use appropriate `Priority` levels

## Performance Considerations

- **Priority Queue**: O(log n) insertion, O(log n) extraction
- **Phase Separation**: Only processes relevant events per phase
- **Batch Buffer**: Prevents infinite recursion with minimal overhead
- **Backward Compatible**: Traditional queue mode has no overhead

## Future Enhancements

Potential areas for expansion:

1. **Event Filtering**: Filter events by category in handlers
2. **Event Recording**: Record and replay event sequences
3. **Async Events**: Support for async event processing
4. **Event Compression**: Merge similar events for performance
5. **Event Analytics**: Track event patterns and statistics
