# Terminal Pixel Dungeon

## Project Overview

Terminal Pixel Dungeon is a terminal-based reimplementation of the popular Shattered Pixel Dungeon game. It's written in Rust and uses a text-based user interface to recreate the classic Roguelike gaming experience in the terminal.

The project is structured as a monorepo with multiple Rust crates organized in the `src` directory:
- `combat`: Handles combat mechanics
- `dungeon`: Manages dungeon generation and state
- `error`: Contains error types and handling
- `hero`: Manages the player character and hero-related mechanics
- `items`: Handles game items and inventory
- `save`: Implements save/load functionality
- `ui`: Provides the terminal user interface using `crossterm` and `tui`

The project is currently in early development and not yet playable, as noted in the README.

## Key Technologies

- Rust 2024 edition
- `tui` crate for terminal user interface
- `crossterm` for cross-platform terminal manipulation
- `serde` for serialization/deserialization
- `anyhow` and `thiserror` for error handling
- `rand` for random number generation

## Building and Running

### Prerequisites
- Rust (latest stable version)
- Cargo

### Commands
To build and run the project:
```bash
cargo run
```

To build without running:
```bash
cargo build
```

To build in release mode:
```bash
cargo build --release
```

The game currently supports auto-saving functionality with a 5-minute interval and can load saved games.

## Project Structure

```
Terminal-Pixel-Dungeon/
├── Cargo.toml          # Main project Cargo manifest
├── Cargo.lock          # Dependency lock file
├── README.md           # Project overview
├── commit.sh          # Commit script
├── src/               # Source code directory
│   ├── lib.rs         # Library module declarations
│   ├── main.rs        # Entry point for the application
│   ├── combat/        # Combat system module
│   ├── dungeon/       # Dungeon generation module (with its own Cargo.toml)
│   ├── error/         # Error handling module
│   ├── hero/          # Hero/player module (with its own Cargo.toml)
│   ├── items/         # Items system module
│   ├── save/          # Save/load functionality module
│   └── ui/            # Terminal UI module (with its own Cargo.toml)
```

## Development Notes

The project uses several external crates for core functionality:
- Terminal UI rendering via `tui` and `crossterm`
- Serialization via `serde` and `bincode`
- Random number generation for dungeon generation and game mechanics
- Error handling with `anyhow` and `thiserror`

Each sub-module (combat, dungeon, hero, items, save, ui) is structured as a separate Rust crate with its own Cargo.toml file, suggesting a modular architecture approach.

## Current Status

The project is marked as "early stage" and currently not playable according to the README. The main loop has been implemented with basic save/load functionality, but gameplay mechanics are still under development.