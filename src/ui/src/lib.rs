//src/ui/src/lib.rs

pub mod input;
pub mod render;
pub mod states;
pub mod terminal;

use dungeon::Dungeon;
use hero::Hero;
use crossterm::{
    terminal::{enable_raw_mode, EnterAlternateScreen},
    event::{self, Event, KeyCode},
};
use std::io;
use std::thread;
use std::time::{Duration, Instant};
use ratatui::{prelude::CrosstermBackend, Terminal};

// 重新导出所有UI组件供外部使用
pub use render::{
    animation::{Animation, AnimationManager, AnimationType, EaseType},
    dialogs::{DialogManager, DialogResult, DialogState, DialogType},
    messages::{GameMessage, MessageRenderer, MessageSystem, MessageType},
};

pub use input::{
    EnhancedInputEvent, EnhancedInputProcessor, InputContextManager, 
    InputMode, KeyMapping,
};

pub use states::{
    help::HelpState,
};

pub struct TerminalUI {
    pub terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    pub message_renderer: MessageRenderer,
    pub dialog_manager: DialogManager,
    pub animation_manager: AnimationManager,
    pub input_manager: InputContextManager,
    pub help_state: Option<HelpState>,
}



impl TerminalUI {
    pub fn new() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { 
            terminal,
            message_renderer: MessageRenderer::new(),
            dialog_manager: DialogManager::new(),
            animation_manager: AnimationManager::new(),
            input_manager: InputContextManager::new(),
            help_state: None,
        })
    }

    /// 添加游戏消息
    pub fn add_message(&mut self, message: GameMessage) {
        self.message_renderer.add_message(message);
    }

    /// 显示对话框
    pub fn show_dialog(&mut self, dialog_type: DialogType) {
        self.dialog_manager.show_dialog(dialog_type);
    }

    /// 显示帮助界面
    pub fn show_help(&mut self) {
        self.help_state = Some(HelpState::new());
        self.input_manager.push_context(InputMode::Menu);
    }

    /// 隐藏帮助界面
    pub fn hide_help(&mut self) {
        self.help_state = None;
        self.input_manager.pop_context();
    }

    /// 检查是否显示帮助
    pub fn is_help_visible(&self) -> bool {
        self.help_state.is_some()
    }

    /// 检查是否有活动对话框
    pub fn has_active_dialog(&self) -> bool {
        self.dialog_manager.has_active_dialog()
    }
    pub fn run_game_loop(&mut self, dungeon: &mut Dungeon, hero: &mut Hero) -> anyhow::Result<()> {
        let mut last_frame_time = Instant::now();

        loop {
            // 处理输入和游戏逻辑
            if let Event::Key(key) = event::read()? {
                // 首先检查是否有活动对话框
                if self.has_active_dialog() {
                    if let Some(result) = self.dialog_manager.handle_input(key) {
                        self.handle_dialog_result(result);
                    }
                    continue;
                }

                // 检查是否显示帮助
                if self.is_help_visible() {
                    if let Some(ref mut help) = self.help_state {
                        if !help.handle_input(key) {
                            self.hide_help();
                        }
                    }
                    continue;
                }

                // 处理游戏按键
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => { 
                        let old_pos = (hero.x, hero.y);
                        hero.x = (hero.x - 1).max(0);
                        if (hero.x, hero.y) != old_pos {
                            self.add_message(GameMessage::movement("Moved west".to_string()));
                        }
                    }
                    KeyCode::Char('j') | KeyCode::Down => { 
                        let old_pos = (hero.x, hero.y);
                        hero.y = (hero.y + 1).max(0);
                        if (hero.x, hero.y) != old_pos {
                            self.add_message(GameMessage::movement("Moved south".to_string()));
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => { 
                        let old_pos = (hero.x, hero.y);
                        hero.y = (hero.y - 1).max(0);
                        if (hero.x, hero.y) != old_pos {
                            self.add_message(GameMessage::movement("Moved north".to_string()));
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right => { 
                        let old_pos = (hero.x, hero.y);
                        hero.x = (hero.x + 1).max(0);
                        if (hero.x, hero.y) != old_pos {
                            self.add_message(GameMessage::movement("Moved east".to_string()));
                        }
                    }
                    KeyCode::Char('i') => self.show_inventory(hero),
                    KeyCode::Char('u') => self.use_item(hero),
                    KeyCode::Char('d') => self.drop_item(hero),
                    KeyCode::Char('>') => self.descend(dungeon, hero),
                    KeyCode::Char('<') => self.ascend(dungeon, hero),
                    KeyCode::Char('?') => self.show_help(),
                    KeyCode::Char('q') => {
                        // 显示退出确认对话框
                        use crate::render::dialogs::DialogPresets;
                        self.show_dialog(DialogPresets::quit_confirmation());
                    },
                    _ => {} // 其他按键处理...
                }
            }

            // 更新动画
            self.animation_manager.update();

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

    /// 处理对话框结果
    fn handle_dialog_result(&mut self, result: DialogResult) {
        match result {
            DialogResult::Confirmed(true) => {
                // 用户确认退出
                std::process::exit(0);
            },
            DialogResult::Confirmed(false) => {
                // 用户取消，继续游戏
                self.add_message(GameMessage::info("Game continued".to_string()));
            },
            DialogResult::Cancelled => {
                // 对话框被取消
                self.add_message(GameMessage::info("Action cancelled".to_string()));
            },
            _ => {
                // 其他结果的处理
            }
        }
    }

    fn draw(&mut self, dungeon: &Dungeon, hero: &Hero) -> anyhow::Result<()> {
        // 检查是否显示帮助
        let show_help = self.help_state.is_some();
        let has_dialog = self.dialog_manager.has_active_dialog();
        
        // 收集渲染需要的数据
        let hero_name = hero.name.clone();
        let hero_hp = hero.hp;
        let hero_max_hp = hero.max_hp;
        let hero_x = hero.x;
        let hero_y = hero.y;
        let dungeon_depth = dungeon.depth;

        self.terminal.draw(|f| {
            let size = f.area();

            // 如果显示帮助，渲染帮助界面
            if show_help {
                // 这里我们无法直接访问help_state，需要重新设计
                // 暂时显示一个简单的帮助界面
                use ratatui::{
                    widgets::{Block, Borders, Paragraph},
                    style::{Color, Style},
                };
                let help_text = Paragraph::new("Help System - Press ESC to close\n\nControls:\nhjkl - Move\ni - Inventory\n? - Help")
                    .block(Block::default().borders(Borders::ALL).title(" Help "))
                    .style(Style::default().fg(Color::Cyan));
                f.render_widget(help_text, size);
                return;
            }

            // 主游戏界面布局
            use ratatui::{
                layout::{Constraint, Direction, Layout},
                widgets::{Block, Borders, Paragraph},
                style::{Color, Style},
                text::Line,
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),  // HUD区域
                    Constraint::Min(10),    // 游戏区域
                    Constraint::Length(4),  // 消息区域
                ])
                .split(size);

            // 渲染HUD (简化版本)
            let hud_text = vec![
                Line::from(format!("Hero: {} (Lv.1)", hero_name)),
                Line::from(format!("Health: {}/{}", hero_hp, hero_max_hp)),
                Line::from(format!("Position: ({}, {})", hero_x, hero_y)),
                Line::from(format!("Depth: {} Floor", dungeon_depth)),
            ];

            let hud = Paragraph::new(hud_text)
                .block(Block::default().borders(Borders::ALL).title(" Status "))
                .style(Style::default().fg(Color::White));

            f.render_widget(hud, chunks[0]);

            // 渲染游戏区域 (简化版本)
            let game_content = vec![
                Line::from("┌─────────────────────────┐"),
                Line::from("│                         │"),
                Line::from("│           @             │"), // 玩家位置
                Line::from("│                         │"),
                Line::from("│                         │"),
                Line::from("│                         │"),
                Line::from("│                         │"),
                Line::from("│                         │"),
                Line::from("└─────────────────────────┘"),
            ];

            let game_area = Paragraph::new(game_content)
                .block(Block::default().borders(Borders::ALL).title(" Dungeon "))
                .style(Style::default().fg(Color::White));

            f.render_widget(game_area, chunks[1]);
        })?;

        // 在闭包外渲染消息和对话框
        self.terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),  // HUD区域
                    Constraint::Min(10),    // 游戏区域
                    Constraint::Length(4),  // 消息区域
                ])
                .split(size);

            // 渲染消息日志
            self.message_renderer.render_brief(f, chunks[2]);

            // 渲染对话框 (如果有的话)
            if has_dialog {
                self.dialog_manager.render(f, size);
            }
        })?;

        Ok(())
    }

    pub fn show_inventory(&mut self, _hero: &Hero) {
        use crate::render::dialogs::{DialogType, DialogItem};
        
        // 创建物品列表 (简化版本)
        let items = vec![
            DialogItem::new("Health Potion".to_string(), "Restores 25 HP".to_string())
                .with_icon('🧪')
                .with_color(ratatui::style::Color::Red)
                .with_quantity(3),
            DialogItem::new("Bread".to_string(), "Restores hunger".to_string())
                .with_icon('🍞')
                .with_color(ratatui::style::Color::Yellow)
                .with_quantity(2),
            DialogItem::new("Sword".to_string(), "A basic iron sword".to_string())
                .with_icon('⚔')
                .with_color(ratatui::style::Color::Cyan),
        ];

        self.show_dialog(DialogType::ItemSelect {
            title: "Inventory".to_string(),
            items,
            selected_index: 0,
        });

        self.add_message(GameMessage::info("Opened inventory".to_string()));
    }

    pub fn use_item(&mut self, _hero: &mut Hero) {
        // 实现使用物品逻辑
        self.add_message(GameMessage::item("Used item".to_string()));
    }

    pub fn backend_mut(&mut self) -> &mut CrosstermBackend<io::Stdout> {
        self.terminal.backend_mut()
    }

    pub fn drop_item(&mut self, _hero: &mut Hero) {
        use crate::render::dialogs::DialogPresets;
        
        self.show_dialog(DialogPresets::info_message("Select an item to drop"));
        self.add_message(GameMessage::item("Dropped item".to_string()));
    }

    pub fn descend(&mut self, dungeon: &mut Dungeon, hero: &mut Hero) {
        // 实现下楼逻辑
        if dungeon.can_descend(hero.x, hero.y) {
            if dungeon.descend().is_ok() {
                self.add_message(GameMessage::dungeon(
                    format!("Descended to floor {}", dungeon.depth)
                ));
                // 重置英雄位置到新层的楼梯位置
                hero.x = 10; // 简化的楼梯位置
                hero.y = 10;
            } else {
                self.add_message(GameMessage::error("Cannot descend here".to_string()));
            }
        } else {
            self.add_message(GameMessage::warning("No stairs here".to_string()));
        }
    }

    pub fn ascend(&mut self, dungeon: &mut Dungeon, hero: &mut Hero) {
        // 实现上楼逻辑
        if dungeon.depth > 1 && dungeon.can_ascend(hero.x, hero.y) {
            if dungeon.ascend().is_ok() {
                self.add_message(GameMessage::dungeon(
                    format!("Ascended to floor {}", dungeon.depth)
                ));
                // 重置英雄位置到上层的楼梯位置
                hero.x = 10; // 简化的楼梯位置
                hero.y = 10;
            } else {
                self.add_message(GameMessage::error("Cannot ascend here".to_string()));
            }
        } else if dungeon.depth <= 1 {
            self.add_message(GameMessage::warning("Already at the top floor".to_string()));
        } else {
            self.add_message(GameMessage::warning("No stairs here".to_string()));
        }
    }
}
