//! UI系统演示程序
//! 
//! 展示所有新增的UI组件功能：
//! - 消息系统
//! - 对话框系统
//! - 动画效果
//! - 帮助系统
//! - 增强输入处理

use ui::{
    GameMessage, MessageType, DialogType, DialogManager, DialogResult,
    AnimationType, Animation, AnimationManager, EaseType,
    HelpState, InputMode, InputContextManager,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
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
};

/// 演示应用程序状态
struct DemoApp {
    /// 消息系统
    message_system: ui::MessageRenderer,
    /// 对话框管理器
    dialog_manager: DialogManager,
    /// 动画管理器
    animation_manager: AnimationManager,
    /// 输入上下文管理器
    input_manager: InputContextManager,
    /// 帮助状态
    help_state: Option<HelpState>,
    /// 当前演示索引
    demo_index: usize,
    /// 可用演示列表
    demos: Vec<Demo>,
    /// 应用运行状态
    running: bool,
    /// 最后更新时间
    last_update: Instant,
}

/// 演示项目
#[derive(Clone)]
struct Demo {
    name: String,
    description: String,
    action: DemoAction,
}

#[derive(Clone)]
enum DemoAction {
    ShowMessages,
    ShowDialogs,
    ShowAnimations,
    ShowHelp,
    TestInput,
}

impl DemoApp {
    fn new() -> Self {
        let demos = vec![
            Demo {
                name: "Message System".to_string(),
                description: "Test different types of game messages".to_string(),
                action: DemoAction::ShowMessages,
            },
            Demo {
                name: "Dialog System".to_string(),
                description: "Show various dialog types".to_string(),
                action: DemoAction::ShowDialogs,
            },
            Demo {
                name: "Animation System".to_string(),
                description: "Display UI animations".to_string(),
                action: DemoAction::ShowAnimations,
            },
            Demo {
                name: "Help System".to_string(),
                description: "Open the help interface".to_string(),
                action: DemoAction::ShowHelp,
            },
            Demo {
                name: "Input System".to_string(),
                description: "Test enhanced input processing".to_string(),
                action: DemoAction::TestInput,
            },
        ];

        let mut animation_manager = AnimationManager::new();
        animation_manager.add_animation(
            "demo_pulse".to_string(),
            Animation::infinite(AnimationType::Pulse, Duration::from_millis(1000), EaseType::EaseInOut),
        );

        Self {
            message_system: ui::MessageRenderer::new(),
            dialog_manager: DialogManager::new(),
            animation_manager,
            input_manager: InputContextManager::new(),
            help_state: None,
            demo_index: 0,
            demos,
            running: true,
            last_update: Instant::now(),
        }
    }

    fn handle_input(&mut self, key: KeyEvent) {
        // 处理帮助界面
        if let Some(ref mut help) = self.help_state {
            if !help.handle_input(key) {
                self.help_state = None;
                self.input_manager.pop_context();
            }
            return;
        }

        // 处理对话框
        if self.dialog_manager.has_active_dialog() {
            if let Some(result) = self.dialog_manager.handle_input(key) {
                self.handle_dialog_result(result);
            }
            return;
        }

        // 处理主菜单输入
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.demo_index > 0 {
                    self.demo_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.demo_index < self.demos.len() - 1 {
                    self.demo_index += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.execute_demo();
            }
            KeyCode::Char('?') => {
                self.show_help();
            }
            _ => {}
        }
    }

    fn execute_demo(&mut self) {
        let demo = self.demos[self.demo_index].clone();
        match demo.action {
            DemoAction::ShowMessages => {
                self.message_system.add_message(GameMessage::info("This is an info message".to_string()));
                self.message_system.add_message(GameMessage::success("Operation completed successfully!".to_string()));
                self.message_system.add_message(GameMessage::warning("This is a warning message".to_string()));
                self.message_system.add_message(GameMessage::error("Something went wrong!".to_string()));
                self.message_system.add_message(GameMessage::combat("You hit the enemy for 15 damage!".to_string()));
                self.message_system.add_message(GameMessage::movement("You moved to the north".to_string()));
                self.message_system.add_message(GameMessage::item("You found a health potion".to_string()));
                self.message_system.add_message(GameMessage::dungeon("You entered a new area".to_string()));
                self.message_system.add_message(GameMessage::status("You are now invisible".to_string()));
            }
            DemoAction::ShowDialogs => {
                self.dialog_manager.show_dialog(DialogType::Confirm {
                    message: "Do you want to continue the demo?".to_string(),
                    default_yes: true,
                });
            }
            DemoAction::ShowAnimations => {
                self.animation_manager.add_animation(
                    "demo_flash".to_string(),
                    Animation::looped(AnimationType::Flash, Duration::from_millis(200), EaseType::Linear, 5),
                );
                self.animation_manager.add_animation(
                    "demo_shake".to_string(),
                    Animation::looped(AnimationType::Shake, Duration::from_millis(100), EaseType::Linear, 10),
                );
                self.message_system.add_message(GameMessage::info("Animations started!".to_string()));
            }
            DemoAction::ShowHelp => {
                self.show_help();
            }
            DemoAction::TestInput => {
                self.message_system.add_message(GameMessage::info("Input test mode activated. Press keys to see enhanced input processing.".to_string()));
            }
        }
    }

    fn show_help(&mut self) {
        self.help_state = Some(HelpState::new());
        self.input_manager.push_context(InputMode::Menu);
    }

    fn handle_dialog_result(&mut self, result: DialogResult) {
        match result {
            DialogResult::Confirmed(true) => {
                self.message_system.add_message(GameMessage::success("Demo confirmed!".to_string()));
            }
            DialogResult::Confirmed(false) => {
                self.message_system.add_message(GameMessage::info("Demo cancelled.".to_string()));
            }
            DialogResult::Cancelled => {
                self.message_system.add_message(GameMessage::warning("Dialog cancelled.".to_string()));
            }
            _ => {
                self.message_system.add_message(GameMessage::info("Dialog completed.".to_string()));
            }
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // 更新动画
        self.animation_manager.update();
    }

    fn render(&mut self, f: &mut Frame) {
        let size = f.size();

        // 如果显示帮助，渲染帮助界面
        if let Some(ref mut help) = self.help_state {
            help.render(f, size);
            return;
        }

        // 主布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // 标题
                Constraint::Min(8),      // 演示列表
                Constraint::Length(5),   // 消息区域
                Constraint::Length(2),   // 帮助信息
            ])
            .split(size);

        // 渲染标题
        let title = Paragraph::new("Terminal Pixel Dungeon - UI Components Demo")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // 渲染演示列表
        let demo_items: Vec<ListItem> = self.demos
            .iter()
            .enumerate()
            .map(|(i, demo)| {
                let mut style = Style::default().fg(Color::White);
                
                // 应用动画效果到选中项
                if i == self.demo_index {
                    if let Some(animation_value) = self.animation_manager.get_value("demo_pulse") {
                        style = style.add_modifier(Modifier::BOLD);
                        if animation_value.color_intensity > 0.8 {
                            style = style.fg(Color::Yellow);
                        }
                    }
                }

                ListItem::new(vec![
                    Line::from(Span::styled(&demo.name, style)),
                    Line::from(Span::styled(
                        format!("  {}", demo.description),
                        Style::default().fg(Color::Gray)
                    )),
                ])
            })
            .collect();

        let demo_list = List::new(demo_items)
            .block(Block::default().borders(Borders::ALL).title(" Available Demos "))
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("► ");

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(self.demo_index));
        f.render_stateful_widget(demo_list, chunks[1], &mut list_state);

        // 渲染消息区域
        self.message_system.render_brief(f, chunks[2]);

        // 渲染帮助信息
        let help_text = "↑/↓: Navigate | Enter: Execute Demo | ?: Help | Q/ESC: Quit";
        let help_widget = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(help_widget, chunks[3]);

        // 渲染对话框（如果有）
        if self.dialog_manager.has_active_dialog() {
            self.dialog_manager.render(f, size);
        }
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
    let mut app = DemoApp::new();
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
    app: &mut DemoApp,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // 渲染界面
        terminal.draw(|f| app.render(f))?;

        // 处理输入
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                app.handle_input(key);
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