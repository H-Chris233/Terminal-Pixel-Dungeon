//! Ratatui renderer implementation for the ECS architecture.

use crate::ecs::*;
use crate::render::{
    DungeonRenderer, GameOverRenderer, HudRenderer, InventoryRenderer, MenuRenderer,
};
use anyhow;

use ratatui::{
    Frame, Terminal,
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color as TuiColor, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use std::collections::HashMap;
use std::io::{self, Stdout};
use std::time::Duration;

/// Trait for rendering the game state
pub trait Renderer {
    type Backend: Backend;

    /// Initialize the renderer
    fn init(&mut self) -> anyhow::Result<()>;

    /// Draw the current game state
    fn draw(&mut self, ecs_world: &mut ECSWorld) -> anyhow::Result<()>;

    /// Draw UI elements
    fn draw_ui(&mut self, frame: &mut Frame<'_>, area: Rect);

    /// Handle terminal resize
    fn resize(&mut self, resources: &mut Resources, width: u16, height: u16) -> anyhow::Result<()>;

    /// Cleanup resources
    fn cleanup(&mut self) -> anyhow::Result<()>;
}

/// Trait for time management
pub trait Clock {
    /// Get the current time
    fn now(&self) -> std::time::SystemTime;

    /// Get elapsed time since a given point
    fn elapsed(&self, since: std::time::SystemTime) -> Duration;

    /// Sleep for duration
    fn sleep(&self, duration: Duration);

    /// Get fixed time step for game logic updates
    fn tick_rate(&self) -> Duration;
}

/// Ratatui terminal renderer implementation
pub struct RatatuiRenderer {
    terminal: Terminal<ratatui::backend::CrosstermBackend<Stdout>>,
    last_render_time: std::time::Instant,
    render_cache: HashMap<(i32, i32, i32), RenderCacheEntry>, // x, y, z coordinates
    // 模块化渲染器
    dungeon_renderer: DungeonRenderer,
    hud_renderer: HudRenderer,
    inventory_renderer: InventoryRenderer,
    menu_renderer: MenuRenderer,
    game_over_renderer: GameOverRenderer,
}

/// Cached rendering data for optimization
struct RenderCacheEntry {
    symbol: char,
    fg: TuiColor,
    bg: TuiColor,
    timestamp: std::time::Instant,
}

impl RatatuiRenderer {
    pub fn new() -> anyhow::Result<Self> {
        let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            last_render_time: std::time::Instant::now(),
            render_cache: HashMap::new(),
            dungeon_renderer: DungeonRenderer::new(),
            hud_renderer: HudRenderer::new(),
            inventory_renderer: InventoryRenderer::new(),
            menu_renderer: MenuRenderer::new(),
            game_over_renderer: GameOverRenderer::new(),
        })
    }

    /// Render the ECS world to the terminal
    fn render_ecs_world(&mut self, ecs_world: &mut ECSWorld) -> anyhow::Result<()> {
        self.terminal.draw(|f| {
            // 根据游戏状态决定渲染内容
            match ecs_world.resources.game_state.game_state {
                // === 菜单状态 ===
                GameStatus::MainMenu => {
                    self.menu_renderer
                        .render_main_menu(f, f.area(), &ecs_world.resources);
                }

                GameStatus::Paused => {
                    self.menu_renderer
                        .render_pause_menu(f, f.area(), &ecs_world.resources);
                }

                GameStatus::Options { .. } => {
                    self.menu_renderer
                        .render_options_menu(f, f.area(), &ecs_world.resources);
                }

                GameStatus::Help => {
                    self.menu_renderer
                        .render_help_menu(f, f.area(), &ecs_world.resources);
                }

                GameStatus::CharacterInfo => {
                    // TODO: 实现角色信息界面
                    Self::render_character_info_static(f, f.area(), &ecs_world.resources);
                }

                GameStatus::Inventory { .. } => {
                    self.inventory_renderer
                        .render(f, f.area(), &ecs_world.world);
                }

                // === 游戏结束状态 ===
                GameStatus::GameOver { .. } => {
                    self.game_over_renderer
                        .render_game_over(f, f.area(), &ecs_world.resources);
                }

                GameStatus::Victory => {
                    self.game_over_renderer
                        .render_victory(f, f.area(), &ecs_world.resources);
                }

                GameStatus::ConfirmQuit { .. } => {
                    self.menu_renderer
                        .render_confirm_quit(f, f.area(), &ecs_world.resources);
                }

                // === 正常游戏状态 ===
                GameStatus::Running => {
                    // Create main layout
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(3), // HUD (状态栏)
                            Constraint::Min(10),   // Main game area (地牢)
                            Constraint::Length(3), // Message log (消息栏)
                        ])
                        .split(f.area());

                    // 渲染 HUD
                    self.hud_renderer.render(f, chunks[0], &ecs_world.world);

                    // 渲染地牢
                    self.dungeon_renderer.render(f, chunks[1], &ecs_world.world);

                    // 渲染消息日志
                    let messages = Paragraph::new(format_messages(
                        &ecs_world.resources.game_state.message_log,
                    ))
                    .style(Style::default().fg(TuiColor::Gray))
                    .block(Block::default().borders(Borders::TOP));
                    f.render_widget(messages, chunks[2]);
                }
            }
        })?;
        Ok(())
    }

    /// 渲染角色信息界面（临时实现）
    fn render_character_info_static(frame: &mut Frame<'_>, area: Rect, resources: &Resources) {
        let text = vec![
            Line::from("👤 角色信息"),
            Line::from(""),
            Line::from("这里将显示详细的角色属性和成长数据"),
            Line::from("按 Esc 返回游戏"),
        ];

        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(TuiColor::White))
            .block(Block::default().title("角色信息").borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}

impl Renderer for RatatuiRenderer {
    type Backend = ratatui::backend::CrosstermBackend<Stdout>;

    fn init(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn draw(&mut self, ecs_world: &mut ECSWorld) -> anyhow::Result<()> {
        self.render_ecs_world(ecs_world)
    }

    fn draw_ui(&mut self, frame: &mut Frame<'_>, area: Rect) {
        // Draw UI elements in the provided area
        let block = Block::default().title("UI Panel").borders(Borders::ALL);
        frame.render_widget(block, area);
    }

    fn resize(&mut self, resources: &mut Resources, width: u16, height: u16) -> anyhow::Result<()> {
        // Update game state with new dimensions
        resources.game_state.terminal_width = width;
        resources.game_state.terminal_height = height;
        Ok(())
    }

    fn cleanup(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Helper function to format messages for display
fn format_messages(messages: &[String]) -> String {
    if messages.is_empty() {
        "Welcome to Pixel Dungeon!".to_string()
    } else {
        // 显示最近的 3 条消息
        messages
            .iter()
            .rev()
            .take(3)
            .rev()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

/// Clock implementation for time management
pub struct GameClock {
    tick_rate: Duration,
    start_time: std::time::SystemTime,
}

impl GameClock {
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
            start_time: std::time::SystemTime::now(),
        }
    }
}

impl Clock for GameClock {
    fn now(&self) -> std::time::SystemTime {
        std::time::SystemTime::now()
    }

    fn elapsed(&self, since: std::time::SystemTime) -> Duration {
        self.now()
            .duration_since(since)
            .unwrap_or(Duration::from_millis(0))
    }

    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }

    fn tick_rate(&self) -> Duration {
        self.tick_rate
    }
}

/// Convert ECS colors to ratatui colors
impl From<Color> for TuiColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Red => TuiColor::Red,
            Color::Green => TuiColor::Green,
            Color::Yellow => TuiColor::Yellow,
            Color::Blue => TuiColor::Blue,
            Color::Magenta => TuiColor::Magenta,
            Color::Cyan => TuiColor::Cyan,
            Color::Gray => TuiColor::Gray,
            Color::DarkGray => TuiColor::DarkGray,
            Color::White => TuiColor::White,
            Color::Black => TuiColor::Black,
            Color::Reset => TuiColor::Reset,
            Color::Rgb(r, g, b) => TuiColor::Rgb(r, g, b),
        }
    }
}
