
#![allow(dead_code)]
#![allow(unused)]

mod dungeon;
mod error;
mod hero;
mod save;
mod ui;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{Duration, Instant, SystemTime},
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{
    dungeon::Dungeon,
    hero::{class::Class, Hero},
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

    // 2. 初始化存档系统（每5分钟自动保存）
    let save_system = SaveSystem::new("saves", 5)?;
    let mut auto_save = AutoSave::new(save_system, Duration::from_secs(300));

    // 3. 加载或创建新游戏
    let (mut dungeon, mut hero) = match auto_save.system.load_game(0) {
        Ok(data) => {
            println!("Loaded saved game (Depth: {})", data.metadata.dungeon_depth);
            (data.dungeon, data.hero)
        }
        Err(_) => {
            let seed = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();
            let dungeon = Dungeon::generate(seed, 1)?;
            let hero = Hero::new(Class::Warrior);
            println!("New game started with seed: {}", seed);
            (dungeon, hero)
        }
    };

    // 4. 主游戏循环
    let mut ui = TerminalUI::new()?;
    let game_result = ui.run_game_loop(&mut dungeon, &mut hero);

    // 5. 游戏结束处理
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(ui.backend_mut(), LeaveAlternateScreen).context("Failed to leave alternate screen")?;

    // 6. 保存游戏状态（如果角色存活）
    if hero.alive {
        let save_data = SaveData {
            metadata: SaveMetadata {
                timestamp: SystemTime::now(),
                dungeon_depth: dungeon.depth,
                hero_name: hero.name.clone(),
                hero_class: hero.class.to_string(),
                play_time: hero.play_time + Instant::now().duration_since(hero.start_time),
            },
            hero: hero.clone(),
            dungeon: dungeon.clone(),
            game_seed: dungeon.seed,
        };
        auto_save.try_save(&save_data)?;
    }

    // 7. 显示退出消息
    match (game_result, hero.alive) {
        (Err(e), _) => eprintln!("Game crashed: {}", e),
        (_, false) => println!("☠️ Game Over! {} died at depth {}", hero.name, dungeon.depth),
        _ => println!("💾 Game saved at depth {}", dungeon.depth),
    }

    Ok(())
}
