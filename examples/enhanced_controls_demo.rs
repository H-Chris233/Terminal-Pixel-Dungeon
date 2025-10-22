//! 增强控制演示
//! 
//! 展示完整的WASD支持和Del键丢弃功能

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
    collections::VecDeque,
};

/// 演示应用程序
struct ControlsDemoApp {
    /// 玩家位置
    player_pos: (i32, i32),
    /// 移动历史
    move_history: VecDeque<String>,
    /// 输入历史
    input_log: VecDeque<String>,
    /// 运行状态
    running: bool,
    /// 上次更新时间
    last_update: Instant,
}

impl ControlsDemoApp {
    fn new() -> Self {
        let mut app = Self {
            player_pos: (10, 10),
            move_history: VecDeque::with_capacity(20),
            input_log: VecDeque::with_capacity(10),
            running: true,
            last_update: Instant::now(),
        };
        
        app.add_log("Welcome to Enhanced Controls Demo!".to_string());
        app.add_log("Use WASD or HJKL to move".to_string());
        app.add_log("Press Del to 'drop item'".to_string());
        app.add_log("Press Q to quit".to_string());
        
        app
    }

    /// 添加日志消息
    fn add_log(&mut self, message: String) {
        self.input_log.push_back(message);
        while self.input_log.len() > 10 {
            self.input_log.pop_front();
        }
    }

    /// 添加移动记录
    fn add_move(&mut self, direction: &str) {
        let timestamp = format!("{:.1}s", self.last_update.elapsed().as_secs_f32());
        let move_msg = format!("[{}] Moved {}", timestamp, direction);
        self.move_history.push_back(move_msg);
        while self.move_history.len() > 20 {
            self.move_history.pop_front();
        }
    }

    /// 处理输入
    fn handle_input(&mut self, key: KeyCode) {
        let old_pos = self.player_pos;
        
        match key {
            // 完整的WASD支持
            KeyCode::Char('w') | KeyCode::Char('k') | KeyCode::Up => {
                self.player_pos.1 = (self.player_pos.1 - 1).max(1);
                if self.player_pos != old_pos {
                    self.add_move("North (W/K/↑)");
                    self.add_log("Moved North using W/K/↑".to_string());
                }
            }
            KeyCode::Char('s') | KeyCode::Char('j') | KeyCode::Down => {
                self.player_pos.1 = (self.player_pos.1 + 1).min(18);
                if self.player_pos != old_pos {
                    self.add_move("South (S/J/↓)");
                    self.add_log("Moved South using S/J/↓".to_string());
                }
            }
            KeyCode::Char('a') | KeyCode::Char('h') | KeyCode::Left => {
                self.player_pos.0 = (self.player_pos.0 - 1).max(1);
                if self.player_pos != old_pos {
                    self.add_move("West (A/H/←)");
                    self.add_log("Moved West using A/H/←".to_string());
                }
            }
            KeyCode::Char('d') | KeyCode::Char('l') | KeyCode::Right => {
                self.player_pos.0 = (self.player_pos.0 + 1).min(38);
                if self.player_pos != old_pos {
                    self.add_move("East (D/L/→)");
                    self.add_log("Moved East using D/L/→".to_string());
                }
            }

            // 对角线移动 (vi-keys)
            KeyCode::Char('y') => {
                self.player_pos.0 = (self.player_pos.0 - 1).max(1);
                self.player_pos.1 = (self.player_pos.1 - 1).max(1);
                if self.player_pos != old_pos {
                    self.add_move("Northwest (Y)");
                    self.add_log("Moved Northwest using Y".to_string());
                }
            }
            KeyCode::Char('u') => {
                self.player_pos.0 = (self.player_pos.0 + 1).min(38);
                self.player_pos.1 = (self.player_pos.1 - 1).max(1);
                if self.player_pos != old_pos {
                    self.add_move("Northeast (U)");
                    self.add_log("Moved Northeast using U".to_string());
                }
            }
            KeyCode::Char('b') => {
                self.player_pos.0 = (self.player_pos.0 - 1).max(1);
                self.player_pos.1 = (self.player_pos.1 + 1).min(18);
                if self.player_pos != old_pos {
                    self.add_move("Southwest (B)");
                    self.add_log("Moved Southwest using B".to_string());
                }
            }
            KeyCode::Char('n') => {
                self.player_pos.0 = (self.player_pos.0 + 1).min(38);
                self.player_pos.1 = (self.player_pos.1 + 1).min(18);
                if self.player_pos != old_pos {
                    self.add_move("Southeast (N)");
                    self.add_log("Moved Southeast using N".to_string());
                }
            }

            // Del键丢弃物品
            KeyCode::Delete => {
                self.add_log("🗑️ Dropped item using Delete key!".to_string());
            }

            // 其他功能
            KeyCode::Char('.') => {
                self.add_log("⏳ Waiting... (used . key)".to_string());
            }
            KeyCode::Char('i') => {
                self.add_log("🎒 Opening inventory...".to_string());
            }
            KeyCode::Char('?') => {
                self.add_log("❓ Help requested".to_string());
            }

            // 退出
            KeyCode::Char('q') | KeyCode::Esc => {
                self.running = false;
                self.add_log("Goodbye!".to_string());
            }

            _ => {}
        }
    }

    /// 更新逻辑
    fn update(&mut self) {
        self.last_update = Instant::now();
    }

    /// 渲染界面
    fn render(&self, f: &mut Frame) {
        let size = f.size();

        // 主布局
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // 标题
                Constraint::Min(12),     // 游戏区域
                Constraint::Length(8),   // 控制说明
            ])
            .split(size);

        // 标题
        let title = Paragraph::new("Enhanced Controls Demo - WASD + Del Key Support")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, main_chunks[0]);

        // 游戏区域布局
        let game_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),  // 游戏地图
                Constraint::Percentage(40),  // 信息面板
            ])
            .split(main_chunks[1]);

        // 渲染游戏地图
        self.render_game_map(f, game_chunks[0]);

        // 信息面板布局
        let info_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),  // 移动历史
                Constraint::Percentage(60),  // 输入日志
            ])
            .split(game_chunks[1]);

        // 渲染移动历史
        self.render_move_history(f, info_chunks[0]);

        // 渲染输入日志
        self.render_input_log(f, info_chunks[1]);

        // 渲染控制说明
        self.render_controls(f, main_chunks[2]);
    }

    /// 渲染游戏地图
    fn render_game_map(&self, f: &mut Frame, area: Rect) {
        let mut map_lines = Vec::new();
        
        for y in 0..20 {
            let mut line_spans = Vec::new();
            for x in 0..40 {
                let symbol = if (x as i32, y as i32) == self.player_pos {
                    "@"  // 玩家
                } else if x == 0 || x == 39 || y == 0 || y == 19 {
                    "#"  // 墙壁
                } else {
                    "."  // 地板
                };

                let color = if (x as i32, y as i32) == self.player_pos {
                    Color::Yellow
                } else if x == 0 || x == 39 || y == 0 || y == 19 {
                    Color::Gray
                } else {
                    Color::DarkGray
                };

                line_spans.push(Span::styled(symbol, Style::default().fg(color)));
            }
            map_lines.push(Line::from(line_spans));
        }

        let map_widget = Paragraph::new(map_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Game Map ")
                    .style(Style::default().fg(Color::White))
            );

        f.render_widget(map_widget, area);
    }

    /// 渲染移动历史
    fn render_move_history(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.move_history
            .iter()
            .rev()
            .take(area.height as usize - 2)
            .map(|move_str| {
                ListItem::new(Line::from(Span::styled(
                    move_str,
                    Style::default().fg(Color::Green)
                )))
            })
            .collect();

        let moves_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Move History ")
                    .style(Style::default().fg(Color::Green))
            );

        f.render_widget(moves_list, area);
    }

    /// 渲染输入日志
    fn render_input_log(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.input_log
            .iter()
            .rev()
            .take(area.height as usize - 2)
            .map(|log_str| {
                let color = if log_str.contains("🗑️") {
                    Color::Red
                } else if log_str.contains("🎒") || log_str.contains("❓") {
                    Color::Cyan
                } else if log_str.contains("⏳") {
                    Color::Yellow
                } else {
                    Color::White
                };

                ListItem::new(Line::from(Span::styled(
                    log_str,
                    Style::default().fg(color)
                )))
            })
            .collect();

        let log_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input Log ")
                    .style(Style::default().fg(Color::Cyan))
            );

        f.render_widget(log_list, area);
    }

    /// 渲染控制说明
    fn render_controls(&self, f: &mut Frame, area: Rect) {
        let controls_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(area);

        // 移动控制
        let movement_text = vec![
            Line::from(Span::styled("Movement:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from("WASD - Cardinal movement"),
            Line::from("HJKL - Vi-style movement"), 
            Line::from("YUBN - Diagonal movement"),
            Line::from("Arrows - Alternative movement"),
        ];

        let movement_widget = Paragraph::new(movement_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Movement ")
                    .style(Style::default().fg(Color::Yellow))
            );

        f.render_widget(movement_widget, controls_chunks[0]);

        // 动作控制
        let actions_text = vec![
            Line::from(Span::styled("Actions:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            Line::from("Del - Drop item"),
            Line::from(". - Wait/Rest"),
            Line::from("i - Inventory"),
            Line::from("? - Help"),
        ];

        let actions_widget = Paragraph::new(actions_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Actions ")
                    .style(Style::default().fg(Color::Red))
            );

        f.render_widget(actions_widget, controls_chunks[1]);

        // 系统控制
        let system_text = vec![
            Line::from(Span::styled("System:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
            Line::from("Q - Quit"),
            Line::from("Esc - Exit"),
            Line::from(""),
            Line::from(format!("Position: ({}, {})", self.player_pos.0, self.player_pos.1)),
        ];

        let system_widget = Paragraph::new(system_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" System ")
                    .style(Style::default().fg(Color::Magenta))
            );

        f.render_widget(system_widget, controls_chunks[2]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 运行应用
    let mut app = ControlsDemoApp::new();
    let result = run_app(&mut terminal, &mut app);

    // 清理终端
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut ControlsDemoApp,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // 渲染界面
        terminal.draw(|f| app.render(f))?;

        // 处理输入
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                app.handle_input(key.code);
            }
        }

        // 更新应用状态
        app.update();

        // 检查是否应该退出
        if !app.running {
            break;
        }
    }

    Ok(())
}