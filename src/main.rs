#![allow(dead_code)]
#![allow(unused)]

pub mod combat;
pub mod dungeon;
pub mod error;
pub mod hero;
pub mod items;
pub mod save;
pub mod ui;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use scopeguard::defer;
use std::{
    io, process,
    time::{Duration, Instant, SystemTime},
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{
    dungeon::dungeon::Dungeon,
    hero::{class::class::Class, hero::Hero},
    save::save::{AutoSave, SaveData, SaveMetadata, SaveSystem},
    ui::ui::TerminalUI,
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

    let seed = {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();
        let pid = process::id();
        (time ^ (pid as u128)) as u64
    };

    let save_system = SaveSystem::new("saves", 5)?;
    let mut auto_save = AutoSave::new(save_system, Duration::from_secs(300));

    let (mut dungeon, mut hero) = match auto_save.save_system.load_game(0) {
        Ok(data) => {
            println!("Loaded saved game (Depth: {})", data.metadata.dungeon_depth);
            let mut hero = data.hero;
            hero.start_time = Instant::now() - Duration::from_secs_f64(data.metadata.play_time);
            (data.dungeon, hero)
        }
        Err(_) => {
            println!("New game started with seed: {}", seed);
            let dungeon = Dungeon::generate(1, seed)?; // 添加初始深度参数
            let hero = Hero::new(Class::Warrior);
            (dungeon, hero)
        }
    };

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;
    let mut ui = TerminalUI::new()?;

    let game_result = ui.run_game_loop(&mut dungeon, &mut hero);

    match (game_result, hero.alive) {
        (Err(e), _) => eprintln!("Game crashed: {}", e),
        (_, false) => println!(
            "☠️ Game Over! {} died at depth {}",
            hero.name, dungeon.depth
        ),
        _ => {
            let save_data = SaveData {
                metadata: SaveMetadata {
                    timestamp: SystemTime::now(),
                    dungeon_depth: dungeon.depth,
                    hero_name: hero.name.clone(),
                    hero_class: format!("{:?}", hero.class), // 使用Debug格式
                    play_time: hero.play_time as f64
                        + Instant::now().duration_since(hero.start_time).as_secs_f64(),
                },
                hero: hero.clone(),
                dungeon: dungeon.clone(),
                game_seed: dungeon.seed,
            };
            auto_save.force_save(&save_data)?;
            println!("💾 Game saved at depth {}", dungeon.depth);
        }
    }

    Ok(())
}
