pub mod core;
pub mod ecs;
pub mod event_bus;
pub mod game_loop;
pub mod input;
pub mod render; // 模块化渲染组件
pub mod renderer;
pub mod systems;
pub mod turn_system;

use anyhow::Context;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io;

use crate::{
    game_loop::GameLoop,
    input::ConsoleInput,
    renderer::{GameClock, RatatuiRenderer},
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

    // 初始化基于 ECS 的渲染器和输入源
    let renderer = RatatuiRenderer::new()?;
    let input_source = ConsoleInput::new();
    let clock = GameClock::new(16); // ~60 FPS

    // 初始化并运行游戏循环
    let mut game_loop = GameLoop::new(renderer, input_source, clock);
    game_loop.initialize()?;
    game_loop.run()?;

    Ok(())
}
