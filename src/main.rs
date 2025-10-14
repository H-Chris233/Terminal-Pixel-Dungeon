pub mod ecs;
pub mod systems;
pub mod game_loop;
pub mod input;
pub mod renderer;
pub mod turn_system;
pub mod core;
pub mod event_bus;


use anyhow::Context;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;

use crate::{
    renderer::{RatatuiRenderer, GameClock},
    input::ConsoleInput,
    game_loop::GameLoop,
};

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn main() -> anyhow::Result<()> {
    let _guard = TerminalGuard;
    enable_raw_mode().context("Failed to enable raw mode")?;
    execute!(io::stdout(), EnterAlternateScreen).context("Failed to enter alternate screen")?;

    // Initialize the new ECS-based renderer and input source
    let renderer = RatatuiRenderer::new()?;
    let input_source = ConsoleInput::new();
    let clock = GameClock::new(16); // ~60 FPS
    
    // Initialize and run the game loop
    let mut game_loop = GameLoop::new(renderer, input_source, clock);
    game_loop.initialize()?;
    game_loop.run()?;

    Ok(())
}
