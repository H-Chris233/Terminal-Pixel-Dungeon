# Turn System Architecture

This document describes the revamped, energy-driven turn architecture that powers Terminal Pixel Dungeon. It explains how the state machine, phase pipeline, and event bus cooperate to schedule actions, when energy is consumed or regenerated, and how to extend the system with new actions or effects without breaking save compatibility.

> **Cross references**: The high-level ECS + module split is covered in the [project README](../README.md). Event bus usage patterns are detailed in [EVENT_BUS_GUIDE.md](EVENT_BUS_GUIDE.md). UI affordances referenced here were introduced in [UI_IMPROVEMENTS.md](../UI_IMPROVEMENTS.md).

## State Machine Overview

The `TurnSystem` exposes a minimal state machine that alternates control between the player and AI actors while allowing intermediate processing states for future extensions:

```
┌────────────┐      player acts       ┌────────────────────┐
│ PlayerTurn │ ─────────────────────▶ │ ProcessingPlayer…  │
└────────────┘                        └────────────────────┘
      ▲                                         │
      │                                         │complete + energy spent
      │      AI gains control                   ▼
┌────────────┐      and resolves      ┌────────────────────┐
│  AITurn    │ ◀───────────────────── │ ProcessingAI…      │
└────────────┘                        └────────────────────┘
```

* **`PlayerTurn`** – the game waits for input and runs player-facing systems (movement, combat resolution, etc.).
* **`ProcessingPlayerAction`** – reserved for multi-step player actions (currently unused but kept for deterministic replay support).
* **`AITurn`** – AI controllers spend energy and act repeatedly until the player regains full energy.
* **`ProcessingAIActions`** – reserved for scripted multi-action AI behaviours.

Although only `PlayerTurn` and `AITurn` are active today, documenting the full state graph clarifies where to attach future hooks.

## Phase Pipeline

During each frame the `GameLoop` executes systems in a deterministic order. The order doubles as the action priority list—earlier systems may enqueue or resolve actions before later systems inspect the same input buffer.

| Order | Phase/System            | Responsibilities |
|------:|-------------------------|------------------|
| 1     | `InputSystem`           | Polls devices and pushes `PlayerAction`s into the `pending_actions` queue. |
| 2     | `MenuSystem`            | Consumes menu/navigation actions immediately (they never cost energy). |
| 3     | `TimeSystem`            | Advances the global `turn_count` and regenerates baseline energy for every `Energy` component. |
| 4     | `MovementSystem`        | Resolves step-wise movement; successful moves are copied to `completed_actions` so energy can be deducted once. |
| 5     | `AISystem`              | Plans AI actor intents based on the world snapshot. |
| 6     | `CombatSystem::run_with_events` | Converts attack actions into combat events, emitting damage/status events to the bus. |
| 7     | `FOVSystem`             | Rebuilds viewsheds for entities whose positions changed this frame. |
| 8     | `EffectSystem`          | Ticks active status effects and may enqueue secondary actions. |
| 9     | `EnergySystem`          | Kept for backwards compatibility but skipped while the turn scheduler manages energy explicitly. |
| 10    | `InventorySystem`       | Applies queued inventory interactions (use/drop). |
| 11    | `HungerSystem::run_with_events` | Drains satiety and emits hunger/starvation warnings. |
| 12    | `DungeonSystem`         | Handles level transitions, trap triggers, etc. |
| 13    | `RenderingSystem`       | Produces UI frames, including the hunger and status indicators documented in `UI_IMPROVEMENTS.md`.

After all systems run, the game loop flushes the event bus (`process_events`), advances the turn state via `TurnSystem::process_turn_cycle`, bridges any `GameStatus` changes to events, processes remaining events again, and finally swaps event buffers with `next_frame()`.

## Energy Model and Reference Tables

Energy is tracked per entity through the `Energy` component (`current`, `max`, `regeneration_rate`). The player can only take a full-cost action when `current >= 100` by convention. AI actors behave similarly; however, AI turns continue until the player is back at full energy, preventing the player from being locked out by slow regeneration.

### Energy Costs

The canonical mapping lives in `turn_system::energy_costs` and `TurnSystem::consume_player_energy`. Use the following table as the authoritative reference:

| Action Category                        | `PlayerAction` variant(s)                            | Energy Cost |
|----------------------------------------|------------------------------------------------------|-------------|
| Movement (walk/diagonal)               | `Move(Direction)`                                   | 100         |
| Melee or ranged attack                 | `Attack(Position)`                                  | 100         |
| Use consumable / activate item         | `UseItem(slot)`                                     | 100         |
| Drop item                              | `DropItem(slot)`                                    | 100         |
| Stair traversal                        | `Ascend`, `Descend`                                 | 100         |
| Wait / guard stance                    | `Wait`                                              | 50          |
| Menu / UI transitions & confirmation   | `OpenInventory`, `OpenOptions`, `OpenHelp`, `OpenCharacterInfo`, `CloseMenu`, `MenuNavigate(_)`, `MenuSelect`, `MenuBack`, `Quit` | 0 |

**Regeneration rules**:

* `TimeSystem` increments `GameClock.turn_count` every frame and adds `max(regeneration_rate, 1)` energy to *every* entity to avoid zero-regeneration soft locks.
* `TurnSystem::regenerate_energy` is called after AI processing to top off energy and ensure the player is eligible for the next turn.
* Loading legacy saves resets the player's energy to full (`current = max = 100`) to keep older save files forward-compatible.

### Action Priority Rules

* Actions are always dequeued in the order they were inserted. Systems must either handle an action completely or push it back into `pending_actions` if prerequisites are not met.
* Only actions that succeed should be appended to `completed_actions`. The turn system deducts energy once per completed action, preventing double charges when multiple systems collaborate.
* Menu/UI actions bypass the energy scheduler entirely—`MenuSystem` marks them as completed immediately.

## Event Flow

Turn orchestration surfaces events through two channels:

1. **Explicit turn events** (`GameEvent::PlayerTurnStarted`, `GameEvent::AITurnStarted`, `GameEvent::TurnEnded`) are published whenever `TurnSystem.state` changes. `GameLoop::emit_turn_state_events` (invoked from `update_turn`) compares the previous and current states after each `process_turn_cycle` call.
2. **Game status bridge events** (`GameEvent::GamePaused`, `GameEvent::GameResumed`, `GameEvent::GameOver`, `GameEvent::Victory`) are emitted by `GameLoop::bridge_status_events` whenever `GameStatus` transitions between menu, running, and terminal states.

Downstream systems (UI, logging, autosave) can subscribe to these events to show turn banners, kick off AI thinking timers, or inject narrative text into the message log. The hunger and status glyphs added in the HUD rely on the same event bus to stay synchronised with gameplay.

## Developer Guidelines

### Adding a New Player Action

1. **Extend `PlayerAction`** in `ecs.rs` with a new variant and update the input mapping (see `input/key_event_to_player_action_from_internal`).
2. **Choose an energy policy**:
   * Add a constant to `turn_system::energy_costs` if the existing categories do not fit.
   * Update `TurnSystem::consume_player_energy` to map the new variant to its cost.
3. **Implement system-side handling**:
   * Identify the phase that should process the action (movement, combat, inventory, effect, etc.).
   * Pop the action from `pending_actions`, execute the gameplay logic, and push it into `completed_actions` on success.
4. **Emit events** for UI/logging by publishing to the event bus. Prefer the `GameEvent` variants shared with other systems to reduce coupling.
5. **Update HUD/UI** if the action introduces a new indicator. Follow the iconography patterns in `render/hud.rs` (e.g., 🍖/🍗/🥩/💀 for hunger).
6. **Verify save/load**: ensure `ECSWorld::to_save_data` and `from_save_data` capture any new component state that the action relies on. When adding new fields, provide safe defaults so existing `.sav` files remain valid.

### Adding or Modifying Effects

* Use the `EffectSystem` phase to tick and resolve status effects. Effects that schedule secondary actions should append new `PlayerAction`s or AI intents with zero energy cost and let the turn loop charge the initiating action instead.
* For visual feedback, emit `GameEvent::StatusApplied` / `StatusRemoved` so UI layers can react without tight coupling.
* If an effect influences energy (e.g., haste/slowness), adjust `Energy.regeneration_rate` via components rather than hardcoding values in the scheduler.

### Save Compatibility Checklist

* `save::SaveData` currently serialises hero stats, bag contents, dungeon, and RNG seed. Turn state and energy are reconstructed on load (`TurnSystem.state` defaults to `PlayerTurn`; energy defaults to full).
* When introducing new per-entity energy or action cooldown fields, store them either in existing components that already round-trip through the save pipeline or register additional metadata in `SaveMetadata`.
* Always provide backwards-compatible defaults so that existing save files (which lack the new data) continue to load without errors.

## Sample Turn Timeline

The following timeline illustrates a single player move followed by two AI actions:

1. **Input** – The player presses `h` (west). `InputSystem` pushes `PlayerAction::Move(Direction::West)` into `pending_actions`.
2. **Menu System** – No menu state is active; nothing happens.
3. **Time System** – Global `turn_count` increases to 1234, every entity regains at least 1 energy.
4. **Movement System** – Validates the target tile, moves the player, and appends the action to `completed_actions`.
5. **Combat/FOV/Effects** – No combat triggers; FOV recalculates because the position changed.
6. **Inventory/Hunger/Dungeon** – No-op this frame; hunger tick may trigger `GameEvent::PlayerHungry` which drives the HUD icon (🍗 → 🥩).
7. **Rendering** – HUD reflects updated position, energy remains ≥ 0.
8. **Turn System** – Detects a completed action, subtracts 100 energy, marks the player turn as spent, and switches to `AITurn`.
9. **AI Loop** – The goblin has 120 energy, so it moves twice (each move costs 100). After the loop, `TurnSystem::regenerate_energy` tops every entity back up to their cap, then the state returns to `PlayerTurn`.
10. **Event Dispatch** – `GameEvent::AITurnStarted` fires when the state flip occurs; `GameEvent::TurnEnded { turn: 1234 }` fires when AI resolves and the state returns to `PlayerTurn`. Autosave checks run afterwards.

## UI and Telemetry Notes

* The HUD hunger glyphs and HP/EXP gauges (see `render/hud.rs`) act as the canonical turn/condition indicators. Whenever turn events are emitted, the HUD refreshes via the rendering phase, so new mechanics should follow the same pattern for consistency.
* Confirmation dialogs (e.g., quit confirmation) are energy-free actions processed in the menu phase, preventing accidental desynchronisation with the turn scheduler.

## Further Reading

* [`turn_system.rs`](../src/turn_system.rs) – Implementation of the scheduler and energy policies.
* [`game_loop.rs`](../src/game_loop.rs) – Phase orchestration and event bridging.
* [`systems.rs`](../src/systems.rs) – Concrete system implementations that populate/consume the action buffers.
* [`event_bus.rs`](../src/event_bus.rs) – Event definitions and middleware hooks for observing turn transitions.
