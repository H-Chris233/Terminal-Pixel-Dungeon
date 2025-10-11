# Terminal Pixel Dungeon

A terminal-based roguelike dungeon crawler inspired by Shattered Pixel Dungeon, built with Rust and ECS (Entity Component System) architecture. This game runs entirely in your terminal and features classic roguelike gameplay with turn-based combat, item collection, and dungeon exploration.

## Table of Contents
- [Features](#features)
- [Architecture](#architecture)
- [Installation](#installation)
- [Usage](#usage)
- [Gameplay](#gameplay)
- [Controls](#controls)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Terminal-based Interface**: Play the game directly in your terminal using `ratatui` for rendering
- **ECS Architecture**: Entity-Component-System design for flexible and maintainable game logic
- **Roguelike Elements**: Random dungeon generation, permadeath, and challenging enemies
- **Turn-Based Combat**: Strategic combat system with energy management
- **Item System**: Collect and use various items, weapons, and armor
- **Field of View**: Advanced line-of-sight system for exploration
- **AI System**: Enemy AI with different behaviors (aggressive, passive, etc.)
- **Extensible Design**: Easy to add new systems, components, and entities

## Architecture

The game follows the ECS (Entity-Component-System) pattern:

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

## Installation

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

## Usage

The game is currently in development and not fully playable yet. You can explore the ECS architecture and contribute to the development process. To run the current build:

```bash
cargo run --release
```

## Gameplay

### Core Mechanics
- **Turn-Based Movement**: Each action takes time based on your energy level
- **Field of Vision**: Only visible tiles are rendered, creating atmosphere and strategy
- **Item Management**: Collect, use, and manage various items in your inventory
- **Combat System**: Strategic turn-based combat with attack, defense, and accuracy mechanics
- **Enemy AI**: Different enemy types with unique behaviors and strategies
- **Dungeon Progression**: Explore multiple levels with increasing difficulty

### Character Progression
- Gain experience and level up
- Improve stats over time
- Find better equipment as you descend deeper

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

### Game States
- **Running**: Normal gameplay
- **Paused**: Temporary pause
- **Game Over**: When the player dies
- **Victory**: When the player wins

## Development

### Project Structure

```
├── src/
│   ├── ecs.rs          # ECS framework and core components
│   ├── systems.rs      # Game systems implementation
│   ├── game_loop.rs    # Main game loop
│   ├── renderer.rs     # Terminal rendering with ratatui
│   ├── input.rs        # Input handling and event processing
│   └── main.rs         # Entry point
├── Cargo.toml          # Project dependencies and configuration
├── README.md           # This file
└── LICENSE             # Project license
```

### ECS and Modular Architecture

The project combines ECS (Entity-Component-System) architecture with a modular crate structure:

- **ECS Implementation**: Built on top of `hecs` for flexible game entity management
- **Modular Design**: Separate crates for different game systems (combat, dungeon, hero, etc.)
- **Core Integration**: Central `core` module to coordinate ECS and modular systems
- **Benefits**:
  - Decoupled game logic
  - Flexible entity composition
  - Efficient system execution
  - Clear module boundaries
  - Easy testing and maintenance

### Rendering

The game uses `ratatui` for terminal rendering, providing:
- Responsive terminal UI
- Customizable layouts
- Color and style support
- Widget-based rendering

### Input Handling

Input is managed through the `crossterm` crate, providing:
- Cross-platform terminal support
- Real-time input detection
- Event-based input processing

## Contributing

We welcome contributions to Terminal Pixel Dungeon! Here's how you can help:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Ensure your code follows the project's style and conventions
5. Add or update tests as needed
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Development Guidelines

- Follow Rust best practices and idioms
- Use `rustfmt` to format your code
- Write meaningful commit messages
- Add documentation for new features
- Write tests for new functionality
- Ensure all tests pass before submitting

### Areas Needing Contributions

- **Game Mechanics**: Implementing missing gameplay features
- **Dungeon Generation**: Creating interesting and varied dungeon layouts
- **Balance**: Tuning stats, items, and enemy difficulty
- **Art and Text**: Descriptions, flavor text, and game lore
- **Bug Fixes**: Identifying and resolving issues
- **Performance**: Optimizing systems for better performance
- **Features**: Adding new items, abilities, and content

## License

This project is licensed under the terms found in the [LICENSE](LICENSE) file.

## Acknowledgments

- Inspired by [Shattered Pixel Dungeon](https://shatteredpixel.com/)
- Built with [Rust](https://www.rust-lang.org/)
- Uses [ratatui](https://github.com/ratatui-org/ratatui) for terminal UI
- Uses [crossterm](https://github.com/crossterm-rs/crossterm) for cross-platform terminal manipulation
- Uses [hecs](https://github.com/Ralith/hecs) for ECS architecture

## Roadmap

- [ ] Implement full combat system
- [ ] Design and implement dungeon generation
- [ ] Add more enemy types and AI behaviors
- [ ] Implement item discovery and identification systems
- [ ] Create multiple dungeon levels
- [ ] Add character classes and abilities
- [ ] Implement save/load functionality
- [ ] Add more items and equipment types
- [ ] Balance game mechanics
- [ ] Create a fully playable demo version

## Support

If you encounter any issues or have questions about the project, please open an issue in the GitHub repository.

---
*Terminal Pixel Dungeon - A Rust-based terminal roguelike inspired by Shattered Pixel Dungeon*