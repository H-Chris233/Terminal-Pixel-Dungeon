
pub mod input;
pub mod render;
pub mod states;
pub mod terminal;

use dungeon::dungeon::Dungeon;
use hero::hero::Hero;
use save::save::AutoSave;
use save::save::SaveData;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::thread;
use std::time::{Duration, Instant};
use tui::{backend::CrosstermBackend, Terminal};

pub struct TerminalUI {
    pub terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl TerminalUI {
    pub fn new() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }
    pub fn run_game_loop(&mut self, dungeon: &mut Dungeon, hero: &mut Hero) -> anyhow::Result<()> {
        let mut last_frame_time = Instant::now();

        loop {
            // 处理输入和游戏逻辑
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => hero.move_to(-1, 0, dungeon),
                    KeyCode::Char('j') | KeyCode::Down => hero.move_to(0, 1, dungeon),
                    KeyCode::Char('k') | KeyCode::Up => hero.move_to(0, -1, dungeon),
                    KeyCode::Char('l') | KeyCode::Right => hero.move_to(1, 0, dungeon),
                    KeyCode::Char('i') => self.show_inventory(hero),
                    KeyCode::Char('u') => self.use_item(hero),
                    KeyCode::Char('d') => self.drop_item(hero),
                    KeyCode::Char('>') => self.descend(dungeon, hero),
                    KeyCode::Char('<') => self.ascend(dungeon, hero),
                    KeyCode::Char('q') => break,
                    _ => {} // 其他按键处理...
                }
            }

            // 渲染游戏状态
            self.draw(dungeon, hero)?;

            // 控制帧率
            let frame_time = Instant::now() - last_frame_time;
            if frame_time < Duration::from_millis(16) {
                thread::sleep(Duration::from_millis(16) - frame_time);
            }
            last_frame_time = Instant::now();
        }

        Ok(())
    }

    fn draw(&mut self, dungeon: &Dungeon, hero: &Hero) -> anyhow::Result<()> {
        self.terminal.draw(|f| {
            // 绘制地牢地图
            // 绘制英雄状态
            // 绘制消息日志
        })?;
        Ok(())
    }

    pub fn show_inventory(&mut self, hero: &Hero) {
        // 实现物品栏显示逻辑
    }

    pub fn use_item(&mut self, hero: &mut Hero) {
        // 实现使用物品逻辑
    }

    pub fn backend_mut(&mut self) -> &mut CrosstermBackend<io::Stdout> {
        self.terminal.backend_mut()
    }
    pub fn drop_item(&mut self, hero: &mut Hero) {
        // 实现丢弃物品逻辑
        // 例如：从英雄物品栏移除物品并添加到地牢当前层
    }

    pub fn descend(&mut self, dungeon: &mut Dungeon, hero: &mut Hero) {
        // 实现下楼逻辑
        if dungeon.can_descend(hero.x, hero.y) {
            dungeon.depth += 1;
            // 重置英雄位置到新层的楼梯位置
        }
    }

    pub fn ascend(&mut self, dungeon: &mut Dungeon, hero: &mut Hero) {
        // 实现上楼逻辑
        if dungeon.depth > 1 && dungeon.can_ascend(hero.x, hero.y) {
            dungeon.depth -= 1;
            // 重置英雄位置到上层的楼梯位置
        }
    }
}
