use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    widgets::{Block, Borders},
    layout
    Frame, Terminal,
};

/// 终端控制器，负责初始化/清理终端状态和基础绘制
pub struct TerminalController {
    backend: CrosstermBackend<io::Stdout>,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalController {
    /// 初始化终端进入原始模式和交替屏幕
    pub fn new() -> Result<Self> {
        enable_raw_mode().context("Failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
        
        
        Ok(Self { backend: CrosstermBackend::new(stdout),
                  terminal: Terminal::new(backend).context("Failed to create terminal")?,
                   })
    }

    /// 恢复终端原始设置（应在程序退出前调用）
    pub fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode().context("Failed to disable raw mode")?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)
            .context("Failed to leave alternate screen")?;
        Ok(())
    }

    /// 基础布局划分（适用于像素地牢的经典三栏布局）
    pub fn create_layout(frame: &mut Frame) -> Vec<tui::layout::Rect> {
        Layout::default()
            .direction(layout::Direction::Vertical)
            .constraints([
                Constraint::Length(1), // 顶部状态栏
                Constraint::Min(10),   // 主游戏区域
                Constraint::Length(3), // 底部消息栏
            ])
            .split(frame.size())
    }

    /// 绘制基础游戏边框（带当前地牢层数显示）
    pub fn draw_game_border(frame: &mut Frame, depth: u32) {
        let block = Block::default()
            .title(format!("Pixel Dungeon - Depth {}", depth))
            .borders(Borders::ALL);
        frame.render_widget(block, frame.size());
    }

    /// 处理输入事件（适配像素地牢的经典控制方案）
    pub fn handle_input(&self) -> Result<Option<GameAction>> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => {
                        return Ok(Some(GameAction::Move(Direction::West)))
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        return Ok(Some(GameAction::Move(Direction::South)))
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        return Ok(Some(GameAction::Move(Direction::North)))
                    }
                    KeyCode::Char('l') | KeyCode::Right => {
                        return Ok(Some(GameAction::Move(Direction::East)))
                    }
                    KeyCode::Char('i') => return Ok(Some(GameAction::ShowInventory)),
                    KeyCode::Char('u') => return Ok(Some(GameAction::UseItem)),
                    KeyCode::Char('d') => return Ok(Some(GameAction::DropItem)),
                    KeyCode::Char('>') => return Ok(Some(GameAction::Descend)),
                    KeyCode::Char('<') => return Ok(Some(GameAction::Ascend)),
                    KeyCode::Char('q') => return Ok(Some(GameAction::Quit)),
                    _ => {}
                }
            }
        }
        Ok(None)
    }

    /// 主绘制入口（委托给渲染系统）
    pub fn draw<F>(&mut self, draw_fn: F) -> Result<()>
    where
        F: FnOnce(&mut Frame),
    {
        self.terminal.draw(draw_fn)?;
        Ok(())
    }

    /// 获取后端可变引用（用于特殊终端操作）
    pub fn backend_mut(&mut self) -> &mut CrosstermBackend<io::Stdout> {
        self.terminal.backend_mut()
    }
}

/// 游戏动作枚举（对应像素地牢的核心操作）
#[derive(Debug, Clone, Copy)]
pub enum GameAction {
    Move(Direction),
    ShowInventory,
    UseItem,
    DropItem,
    Ascend,
    Descend,
    Quit,
}

/// 移动方向（与游戏逻辑对齐）
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West,
}
