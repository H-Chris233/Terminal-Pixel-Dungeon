# Terminal Pixel Dungeon - Qwen Code Context

## Project Overview

Terminal Pixel Dungeon is a terminal-based roguelike dungeon crawler inspired by Shattered Pixel Dungeon, built with Rust and ECS (Entity Component System) architecture. This game runs entirely in your terminal and features classic roguelike gameplay with turn-based combat, item collection, and dungeon exploration.

## Architecture

The game follows a robust ECS (Entity-Component-System) pattern with modular crate architecture:

### Components
Components are plain data structures that represent properties of entities:
- `Position`: x, y, z coordinates
- `Actor`: Contains name and faction
- `Renderable`: Visual properties like symbol and color
- `Stats`: Health, attack, defense, etc.
- `Inventory`: Item management
- `Viewshed`: Field of view system
- `Energy`: Turn-based movement system
- `AI`: Enemy behavior
- `Effects`: Active status effects
- `Tile`: Terrain properties

### Entities
Entities are collections of components that represent game objects like:
- Player character
- Enemies (goblins, etc.)
- Items (potions, weapons, etc.)
- Dungeon tiles (walls, floors, stairs)

### Systems
Systems contain the game logic and operate on entities with specific component combinations:
- `MovementSystem`: Handles entity movement and collision
- `AISystem`: Processes enemy AI behavior
- `CombatSystem`: Processes combat interactions
- `FOVSystem`: Calculates field of view for entities
- `EffectSystem`: Manages active effects and status conditions
- `EnergySystem`: Manages turn scheduling based on energy
- `InventorySystem`: Handles item management
- `RenderingSystem`: Prepares data for rendering
- `InputSystem`: Processes player input
- `TimeSystem`: Manages game time progression
- `DungeonSystem`: Manages dungeon generation and level management
- `TurnSystem`: Manages turn-based gameplay flow

## Project Structure

```
├── src/
│   ├── combat/           # Combat system crate
│   ├── core.rs          # Core game engine structure
│   ├── dungeon/         # Dungeon generation system crate
│   ├── ecs.rs           # ECS framework and core components
│   ├── error/           # Error handling system crate
│   ├── event_bus.rs     # Event bus for inter-module communication
│   ├── game_loop.rs     # Main game loop implementation
│   ├── gfx.rs           # Graphics utilities
│   ├── hero/            # Hero module crate
│   ├── hero_adapter.rs  # Bridge between ECS and hero module
│   ├── input.rs         # Input handling
│   ├── items/           # Item system crate
│   ├── lib.rs           # Library module exports
│   ├── renderer.rs      # Terminal rendering with ratatui
│   ├── save/            # Save/load system crate
│   ├── systems.rs       # Game systems implementation
│   ├── turn_system.rs   # Turn-based system implementation
│   ├── ui/              # UI system crate
│   └── main.rs          # Entry point
├── tests/               # Test files
├── Cargo.toml           # Project dependencies and configuration
├── README.md           # Project documentation
├── COMBAT_SYSTEM.md    # Combat system documentation
└── LICENSE             # Project license
```

## Building and Running

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- Git
- A terminal that supports UTF-8 characters

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/your-username/terminal-pixel-dungeon.git
cd terminal-pixel-dungeon
```

2. Build the project:
```bash
cargo build --release
```

3. Run the game:
```bash
cargo run
```

### Project Dependencies
- `anyhow`: Error handling
- `crossterm`: Cross-platform terminal manipulation
- `hecs`: ECS architecture
- `ratatui`: Terminal UI rendering
- `rand`: Random number generation
- `serde`: Serialization/deserialization
- `bincode`: Binary serialization
- `tempfile`: Temporary file handling
- `thiserror`: Custom error types

## Development Conventions

- Follow Rust best practices and idioms
- Use `rustfmt` to format your code
- Write meaningful commit messages
- Add documentation for new features
- Write tests for new functionality
- Ensure all tests pass before submitting

The project uses a modular architecture with separate crates for different game systems (combat, dungeon, hero, etc.) that communicate through an event bus system. The ECS system is built on top of `hecs` for flexible game entity management.

## Combat System

The combat system mimics mechanics from Shattered Pixel Dungeon with:
- Turn-based combat with strategic elements
- Hit/miss calculations using SPD-style formulas
- Critical hit mechanics
- Damage calculation with defense mitigation
- Ambush attack bonuses (2x damage)
- Various status effects (Burning, Poison, Paralysis, etc.)

## Controls

The game uses vi-keys and numpad for movement:

| Key | Action |
|-----|--------|
| `k`, `↑` | Move North |
| `j`, `↓` | Move South |
| `h`, `←` | Move West |
| `l`, `→` | Move East |
| `y`, `7` | Move Northwest |
| `u`, `9` | Move Northeast |
| `b`, `1` | Move Southwest |
| `n`, `3` | Move Southeast |
| `.` | Wait/Skip turn |
| `>` | Descend stairs |
| `<` | Ascend stairs |
| `Shift+K/J/H/L/Y/U/B/N` | Attack in direction |
| `1-9` | Use item from inventory |
| `d` | Drop item |
| `q` | Quit game |

## Game States
- **Running**: Normal gameplay
- **Paused**: Temporary pause
- **Game Over**: When the player dies
- **Victory**: When the player wins