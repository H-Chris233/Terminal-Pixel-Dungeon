#![allow(dead_code)]
#![allow(unused)]

// src/main.rs
mod dungeon;
mod error;
mod hero;
mod save;
mod ui;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{Instant, SystemTime},
};
use tui::{Terminal, backend::CrosstermBackend};

use crate::{
    dungeon::Dungeon,
    hero::Hero,
    save::{AutoSave, SaveData, SaveMetadata, SaveSystem},
    ui::TerminalUI,
};

fn main() -> anyhow::Result<()> {
    // 1. 初始化终端UI
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // 2. 初始化存档系统
    let save_system = SaveSystem::new("saves", 5).context("Failed to initialize save system")?;

    // 3. 尝试加载存档或创建新游戏
    let (mut dungeon, mut hero) = match save_system.load_game(0) {
        Ok(data) => {
            println!("Loaded saved game");
            (data.dungeon, data.hero)
        }
        Err(e) => {
            println!("No save found, creating new game: {}", e);
            let dungeon = Dungeon::generate(1).context("Failed to generate dungeon")?;
            let hero = Hero::new(hero::Class::Warrior);
            (dungeon, hero)
        }
    };

    // 4. 初始化自动保存
    let mut auto_save = AutoSave::new(
        save_system,
        std::time::Duration::from_secs(300), // 每5分钟自动保存
    );

    // 5. 初始化UI并运行主游戏循环
    let mut ui = TerminalUI::new(terminal).context("Failed to initialize UI")?;
    let game_result = ui.run_game_loop(&mut dungeon, &mut hero, &mut auto_save);

    // 6. 游戏结束处理
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(ui.backend_mut(), LeaveAlternateScreen).context("Failed to leave alternate screen")?;

    // 7. 退出前保存游戏状态
    if let (Ok(()), true) = (game_result, hero.alive) {
        let save_data = SaveData {
            metadata: SaveMetadata {
                timestamp: SystemTime::now(),
                dungeon_depth: dungeon.depth,
                hero_name: hero.name.clone(),
                hero_class: format!("{:?}", hero.class),
                play_time: hero.play_time,
            },
            hero,
            dungeon,
            game_seed: dungeon.seed.unwrap_or(0),
        };
        auto_save.save_system.save_game(0, &save_data)?;
    }

    // 8. 显示退出消息
    if let Err(e) = game_result {
        eprintln!("Game ended with error: {}", e);
    } else if !hero.alive {
        println!("Game over! Your hero has died.");
    } else {
        println!("Game saved successfully. See you next time!");
    }

    Ok(())
}
