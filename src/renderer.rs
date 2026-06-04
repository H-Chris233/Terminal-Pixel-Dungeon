//! Ratatui renderer implementation for the ECS architecture.

use crate::ecs::*;
use crate::render::{
    ClassSelectionRenderer, DungeonRenderer, GameOverRenderer, HudRenderer, InventoryRenderer,
    MenuRenderer,
};
use anyhow;

use ratatui::{
    Frame, Terminal,
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color as TuiColor, Modifier, Style},
    text::{Line, Span},
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
#[allow(dead_code)]
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
    class_selection_renderer: ClassSelectionRenderer,
}

#[allow(dead_code)]
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
            class_selection_renderer: ClassSelectionRenderer::new(),
        })
    }

    /// Render the ECS world to the terminal
    fn render_ecs_world(&mut self, ecs_world: &mut ECSWorld) -> anyhow::Result<()> {
        self.terminal.draw(|f| {
            // 根据游戏状态决定渲染内容
            match ecs_world.resources.game_state.game_state {
                // === 菜单状态 ===
                GameStatus::MainMenu { .. } => {
                    self.menu_renderer
                        .render_main_menu(f, f.area(), &ecs_world.resources);
                }

                GameStatus::Paused { .. } => {
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
                    // 渲染角色信息界面
                    Self::render_character_info(f, f.area(), &ecs_world.world);
                }

                GameStatus::ClassSelection { .. } => {
                    self.class_selection_renderer
                        .render(f, f.area(), &ecs_world.resources);
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
                            Constraint::Length(5), // Message log (消息栏)
                        ])
                        .split(f.area());

                    // 渲染 HUD
                    self.hud_renderer.render(f, chunks[0], &ecs_world.world);

                    // 渲染地牢
                    self.dungeon_renderer.render(f, chunks[1], &ecs_world.world);

                    // 渲染消息日志（改进版）
                    Self::render_message_log(
                        f,
                        chunks[2],
                        &ecs_world.resources.game_state.message_log,
                    );
                }
            }
        })?;
        Ok(())
    }

    /// 渲染角色信息界面
    fn render_character_info(frame: &mut Frame<'_>, area: Rect, world: &hecs::World) {
        use crate::ecs::{Actor, Hunger, Player, PlayerProgress, Stats, Wealth};

        // 获取玩家数据
        let player_data = world
            .query::<(&Stats, &Wealth, &Hunger, &PlayerProgress, &Actor, &Player)>()
            .iter()
            .next()
            .map(|(_, (stats, wealth, hunger, progress, actor, _))| {
                (
                    stats.clone(),
                    wealth.clone(),
                    hunger.clone(),
                    progress.clone(),
                    actor.name.clone(),
                )
            });

        if player_data.is_none() {
            let text = Paragraph::new("⚠️ 未找到角色数据")
                .style(Style::default().fg(TuiColor::Red))
                .block(Block::default().title("角色信息").borders(Borders::ALL))
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(text, area);
            return;
        }

        let (stats, wealth, hunger, progress, actor_name) = player_data.unwrap();

        // 主布局
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题
                Constraint::Min(10),   // 内容
                Constraint::Length(2), // 底部提示
            ])
            .split(area);

        // 标题
        let title = Paragraph::new(format!("👤 {} - {}", actor_name, progress.class))
            .style(
                Style::default()
                    .fg(TuiColor::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Double)
                    .border_style(Style::default().fg(TuiColor::Cyan)),
            )
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(title, main_chunks[0]);

        // 内容区域
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // 左侧：基础属性
                Constraint::Percentage(50), // 右侧：战斗属性
            ])
            .split(main_chunks[1]);

        // 左侧：基础属性
        let basic_info = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("等级: ", Style::default().fg(TuiColor::Gray)),
                Span::styled(
                    format!("{}", stats.level),
                    Style::default()
                        .fg(TuiColor::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("经验: ", Style::default().fg(TuiColor::Gray)),
                Span::styled(
                    format!("{}/{}", stats.experience, stats.level * 100),
                    Style::default().fg(TuiColor::Magenta),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("生命值: ", Style::default().fg(TuiColor::Gray)),
                Span::styled(
                    format!("{}/{}", stats.hp, stats.max_hp),
                    Style::default().fg(TuiColor::Red),
                ),
            ]),
            Line::from(vec![
                Span::styled("力量: ", Style::default().fg(TuiColor::Gray)),
                Span::styled(
                    format!("{}", progress.strength),
                    Style::default().fg(TuiColor::Green),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("金币: ", Style::default().fg(TuiColor::Gray)),
                Span::styled(
                    format!("💰 {}", wealth.gold),
                    Style::default().fg(TuiColor::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("饱食度: ", Style::default().fg(TuiColor::Gray)),
                Span::styled(
                    format!("🍖 {}/10", hunger.satiety),
                    Style::default().fg(if hunger.is_hungry() {
                        TuiColor::Red
                    } else {
                        TuiColor::Green
                    }),
                ),
            ]),
        ];

        let basic_paragraph = Paragraph::new(basic_info)
            .style(Style::default().fg(TuiColor::White))
            .block(
                Block::default()
                    .title("═══ 基础属性 ═══")
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(TuiColor::Green)),
            );
        frame.render_widget(basic_paragraph, content_chunks[0]);

        // 右侧：战斗属性
        let combat_info = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("攻击力: ", Style::default().fg(TuiColor::Gray)),
                Span::styled(
                    format!("{}", stats.attack),
                    Style::default()
                        .fg(TuiColor::Red)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("防御力: ", Style::default().fg(TuiColor::Gray)),
                Span::styled(
                    format!("{}", stats.defense),
                    Style::default()
                        .fg(TuiColor::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("命中率: ", Style::default().fg(TuiColor::Gray)),
                Span::styled(
                    format!("{}", stats.accuracy),
                    Style::default().fg(TuiColor::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("闪避率: ", Style::default().fg(TuiColor::Gray)),
                Span::styled(
                    format!("{}", stats.evasion),
                    Style::default().fg(TuiColor::Cyan),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("职业: ", Style::default().fg(TuiColor::Gray)),
                Span::styled(
                    format!("{}", progress.class),
                    Style::default()
                        .fg(TuiColor::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let combat_paragraph = Paragraph::new(combat_info)
            .style(Style::default().fg(TuiColor::White))
            .block(
                Block::default()
                    .title("═══ 战斗属性 ═══")
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(TuiColor::Red)),
            );
        frame.render_widget(combat_paragraph, content_chunks[1]);

        // 底部提示
        let hint = Paragraph::new("按 Esc 返回游戏")
            .style(Style::default().fg(TuiColor::Gray))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(hint, main_chunks[2]);
    }

    /// 渲染消息日志（改进版）
    fn render_message_log(frame: &mut Frame<'_>, area: Rect, messages: &[String]) {
        let message_lines: Vec<Line> = if messages.is_empty() {
            vec![
                Line::from(vec![
                    ratatui::text::Span::styled("欢迎来到", Style::default().fg(TuiColor::White)),
                    ratatui::text::Span::styled(
                        " 终端像素地牢",
                        Style::default().fg(TuiColor::Yellow),
                    ),
                    ratatui::text::Span::styled("！", Style::default().fg(TuiColor::White)),
                ]),
                Line::from(ratatui::text::Span::styled(
                    "小心探索，祝你好运！",
                    Style::default().fg(TuiColor::Green),
                )),
            ]
        } else {
            // 显示最近的 3 条消息
            messages
                .iter()
                .rev()
                .take(3)
                .rev()
                .map(|msg| {
                    let (prefix, color) = if msg.starts_with("!")
                        || msg.contains("死亡")
                        || msg.contains("受伤")
                    {
                        ("⚠ ", TuiColor::Red)
                    } else if msg.starts_with("+") || msg.contains("获得") || msg.contains("拾取")
                    {
                        ("✓ ", TuiColor::Green)
                    } else if msg.starts_with("*") || msg.contains("发现") {
                        ("★ ", TuiColor::Yellow)
                    } else {
                        ("• ", TuiColor::White)
                    };

                    Line::from(vec![
                        ratatui::text::Span::styled(prefix, Style::default().fg(color)),
                        ratatui::text::Span::styled(msg, Style::default().fg(color)),
                    ])
                })
                .collect()
        };

        let messages_widget = Paragraph::new(message_lines)
            .style(Style::default().fg(TuiColor::Gray))
            .block(
                Block::default()
                    .title("📜 消息日志")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(TuiColor::Rgb(100, 100, 100)))
                    .border_type(ratatui::widgets::BorderType::Rounded),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(messages_widget, area);
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    /// Mock backend for testing
    pub struct MockBackend;

    impl Backend for MockBackend {
        fn draw<'a, I>(&mut self, _content: I) -> io::Result<()>
        where
            I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
        {
            Ok(())
        }

        fn hide_cursor(&mut self) -> io::Result<()> {
            Ok(())
        }

        fn show_cursor(&mut self) -> io::Result<()> {
            Ok(())
        }

        fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
            Ok((0, 0))
        }

        fn get_cursor_position(&mut self) -> io::Result<ratatui::layout::Position> {
            Ok(ratatui::layout::Position { x: 0, y: 0 })
        }

        fn set_cursor(&mut self, _x: u16, _y: u16) -> io::Result<()> {
            Ok(())
        }

        fn set_cursor_position<P: Into<ratatui::layout::Position>>(&mut self, _position: P) -> io::Result<()> {
            Ok(())
        }

        fn clear(&mut self) -> io::Result<()> {
            Ok(())
        }

        fn size(&self) -> io::Result<ratatui::layout::Size> {
            Ok(ratatui::layout::Size { width: 80, height: 24 })
        }

        fn window_size(&mut self) -> io::Result<ratatui::backend::WindowSize> {
            Ok(ratatui::backend::WindowSize {
                columns_rows: ratatui::layout::Size { width: 80, height: 24 },
                pixels: ratatui::layout::Size { width: 800, height: 600 },
            })
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// Mock renderer for testing
    pub struct MockRenderer;

    impl Renderer for MockRenderer {
        type Backend = MockBackend;

        fn init(&mut self) -> anyhow::Result<()> {
            Ok(())
        }

        fn draw(&mut self, _ecs_world: &mut ECSWorld) -> anyhow::Result<()> {
            Ok(())
        }

        fn draw_ui(&mut self, _frame: &mut Frame<'_>, _area: Rect) {
            // No-op for tests
        }

        fn resize(&mut self, _resources: &mut Resources, _width: u16, _height: u16) -> anyhow::Result<()> {
            Ok(())
        }

        fn cleanup(&mut self) -> anyhow::Result<()> {
            Ok(())
        }
    }
}
