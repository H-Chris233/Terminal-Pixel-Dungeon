//! 对话框系统
//!
//! 提供各种类型的对话框和确认窗口：
//! - 确认对话框 (Yes/No)
//! - 信息对话框
//! - 错误对话框
//! - 输入对话框
//! - 选择对话框
//! - 物品选择对话框

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};

/// 对话框类型
#[derive(Debug, Clone)]
pub enum DialogType {
    /// 信息对话框 - 只显示信息，按任意键继续
    Info { message: String },
    /// 错误对话框 - 显示错误信息
    Error { message: String },
    /// 确认对话框 - Yes/No选择
    Confirm { 
        message: String, 
        default_yes: bool 
    },
    /// 输入对话框 - 文本输入
    Input { 
        prompt: String, 
        current_input: String, 
        max_length: usize 
    },
    /// 单选对话框 - 从列表中选择一项
    Select { 
        title: String, 
        options: Vec<String>, 
        selected_index: usize 
    },
    /// 多选对话框 - 选择多项
    MultiSelect { 
        title: String, 
        options: Vec<(String, bool)>, 
        selected_index: usize 
    },
    /// 物品选择对话框 - 特殊的物品选择界面
    ItemSelect { 
        title: String, 
        items: Vec<DialogItem>, 
        selected_index: usize 
    },
}

/// 对话框物品
#[derive(Debug, Clone)]
pub struct DialogItem {
    pub name: String,
    pub description: String,
    pub icon: char,
    pub color: Color,
    pub quantity: Option<u32>,
    pub enabled: bool,
}

impl DialogItem {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            icon: '?',
            color: Color::White,
            quantity: None,
            enabled: true,
        }
    }

    pub fn with_icon(mut self, icon: char) -> Self {
        self.icon = icon;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_quantity(mut self, quantity: u32) -> Self {
        self.quantity = Some(quantity);
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// 对话框结果
#[derive(Debug, Clone)]
pub enum DialogResult {
    /// 对话框被取消
    Cancelled,
    /// 确认对话框：用户选择
    Confirmed(bool),
    /// 输入对话框：用户输入的文本
    Input(String),
    /// 选择对话框：选择的索引
    Selected(usize),
    /// 多选对话框：选择的索引列表
    MultiSelected(Vec<usize>),
    /// 继续（信息对话框）
    Continue,
}

/// 对话框状态
pub struct DialogState {
    pub dialog_type: DialogType,
    pub is_visible: bool,
    pub list_state: ListState,
}

impl DialogState {
    pub fn new(dialog_type: DialogType) -> Self {
        let mut list_state = ListState::default();
        // 根据对话框类型设置初始选择
        match &dialog_type {
            DialogType::Confirm { default_yes, .. } => {
                list_state.select(Some(if *default_yes { 0 } else { 1 }));
            }
            DialogType::Select { selected_index, .. } |
            DialogType::MultiSelect { selected_index, .. } |
            DialogType::ItemSelect { selected_index, .. } => {
                list_state.select(Some(*selected_index));
            }
            _ => {}
        }

        Self {
            dialog_type,
            is_visible: true,
            list_state,
        }
    }

    /// 处理键盘输入
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<DialogResult> {
        match &mut self.dialog_type {
            DialogType::Info { .. } | DialogType::Error { .. } => {
                // 任意键继续
                Some(DialogResult::Continue)
            }
            
            DialogType::Confirm { .. } => {
                match key.code {
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('a') => {
                        self.list_state.select(Some(0)); // Yes
                        None
                    }
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('d') => {
                        self.list_state.select(Some(1)); // No
                        None
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let selected = self.list_state.selected().unwrap_or(1);
                        Some(DialogResult::Confirmed(selected == 0))
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        Some(DialogResult::Confirmed(true))
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        Some(DialogResult::Confirmed(false))
                    }
                    KeyCode::Esc => Some(DialogResult::Cancelled),
                    _ => None,
                }
            }
            
            DialogType::Input { current_input, max_length, .. } => {
                match key.code {
                    KeyCode::Char(c) if current_input.len() < *max_length => {
                        current_input.push(c);
                        None
                    }
                    KeyCode::Backspace => {
                        current_input.pop();
                        None
                    }
                    KeyCode::Enter => {
                        Some(DialogResult::Input(current_input.clone()))
                    }
                    KeyCode::Esc => Some(DialogResult::Cancelled),
                    _ => None,
                }
            }
            
            DialogType::Select { options, .. } => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                        let current = self.list_state.selected().unwrap_or(0);
                        let new_index = if current == 0 { options.len() - 1 } else { current - 1 };
                        self.list_state.select(Some(new_index));
                        None
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                        let current = self.list_state.selected().unwrap_or(0);
                        let new_index = (current + 1) % options.len();
                        self.list_state.select(Some(new_index));
                        None
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let selected = self.list_state.selected().unwrap_or(0);
                        Some(DialogResult::Selected(selected))
                    }
                    KeyCode::Esc => Some(DialogResult::Cancelled),
                    _ => None,
                }
            }
            
            DialogType::MultiSelect { options, .. } => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                        let current = self.list_state.selected().unwrap_or(0);
                        let new_index = if current == 0 { options.len() - 1 } else { current - 1 };
                        self.list_state.select(Some(new_index));
                        None
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                        let current = self.list_state.selected().unwrap_or(0);
                        let new_index = (current + 1) % options.len();
                        self.list_state.select(Some(new_index));
                        None
                    }
                    KeyCode::Char(' ') => {
                        // 切换当前项的选择状态
                        if let Some(selected) = self.list_state.selected() {
                            if selected < options.len() {
                                options[selected].1 = !options[selected].1;
                            }
                        }
                        None
                    }
                    KeyCode::Enter => {
                        // 返回所有被选中的项
                        let selected_indices: Vec<usize> = options
                            .iter()
                            .enumerate()
                            .filter_map(|(i, (_, selected))| if *selected { Some(i) } else { None })
                            .collect();
                        Some(DialogResult::MultiSelected(selected_indices))
                    }
                    KeyCode::Esc => Some(DialogResult::Cancelled),
                    _ => None,
                }
            }
            
            DialogType::ItemSelect { items, .. } => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                        let current = self.list_state.selected().unwrap_or(0);
                        let new_index = if current == 0 { items.len() - 1 } else { current - 1 };
                        self.list_state.select(Some(new_index));
                        None
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                        let current = self.list_state.selected().unwrap_or(0);
                        let new_index = (current + 1) % items.len();
                        self.list_state.select(Some(new_index));
                        None
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let selected = self.list_state.selected().unwrap_or(0);
                        if selected < items.len() && items[selected].enabled {
                            Some(DialogResult::Selected(selected))
                        } else {
                            None
                        }
                    }
                    KeyCode::Esc => Some(DialogResult::Cancelled),
                    _ => None,
                }
            }
        }
    }

    /// 隐藏对话框
    pub fn close(&mut self) {
        self.is_visible = false;
    }
}

/// 对话框渲染器
pub struct DialogRenderer;

impl DialogRenderer {
    /// 渲染对话框
    pub fn render(dialog: &mut DialogState, f: &mut Frame, area: Rect) {
        if !dialog.is_visible {
            return;
        }

        match &dialog.dialog_type {
            DialogType::Info { message } => {
                Self::render_info_dialog(f, area, message);
            }
            DialogType::Error { message } => {
                Self::render_error_dialog(f, area, message);
            }
            DialogType::Confirm { message, .. } => {
                Self::render_confirm_dialog(f, area, message, &mut dialog.list_state);
            }
            DialogType::Input { prompt, current_input, .. } => {
                Self::render_input_dialog(f, area, prompt, current_input);
            }
            DialogType::Select { title, options, .. } => {
                Self::render_select_dialog(f, area, title, options, &mut dialog.list_state);
            }
            DialogType::MultiSelect { title, options, .. } => {
                Self::render_multi_select_dialog(f, area, title, options, &mut dialog.list_state);
            }
            DialogType::ItemSelect { title, items, .. } => {
                Self::render_item_select_dialog(f, area, title, items, &mut dialog.list_state);
            }
        }
    }

    /// 创建居中的对话框区域
    fn create_centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(vertical[1])[1]
    }

    /// 渲染信息对话框
    fn render_info_dialog(f: &mut Frame, area: Rect, message: &str) {
        let dialog_area = Self::create_centered_rect(60, 30, area);
        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(" Information ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let text = Paragraph::new(message)
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);

        f.render_widget(text, dialog_area);

        // 提示信息
        let hint_area = Rect {
            y: dialog_area.bottom() - 2,
            height: 1,
            ..dialog_area
        };
        let hint = Paragraph::new("Press any key to continue")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(hint, hint_area.inner(&Margin { horizontal: 1, vertical: 0 }));
    }

    /// 渲染错误对话框
    fn render_error_dialog(f: &mut Frame, area: Rect, message: &str) {
        let dialog_area = Self::create_centered_rect(60, 30, area);
        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(" Error ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Red));

        let text = Paragraph::new(message)
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));

        f.render_widget(text, dialog_area);
    }

    /// 渲染确认对话框
    fn render_confirm_dialog(f: &mut Frame, area: Rect, message: &str, list_state: &mut ListState) {
        let dialog_area = Self::create_centered_rect(50, 25, area);
        f.render_widget(Clear, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(dialog_area);

        // 消息
        let message_block = Block::default()
            .title(" Confirm ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));

        let message_text = Paragraph::new(message)
            .block(message_block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);

        f.render_widget(message_text, chunks[0]);

        // 按钮
        let buttons = vec![
            ListItem::new("Yes").style(Style::default().fg(Color::Green)),
            ListItem::new("No").style(Style::default().fg(Color::Red)),
        ];

        let buttons_list = List::new(buttons)
            .block(Block::default().borders(Borders::TOP))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("► ");

        f.render_stateful_widget(buttons_list, chunks[1], list_state);
    }

    /// 渲染输入对话框
    fn render_input_dialog(f: &mut Frame, area: Rect, prompt: &str, current_input: &str) {
        let dialog_area = Self::create_centered_rect(60, 25, area);
        f.render_widget(Clear, dialog_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(1)])
            .split(dialog_area);

        // 提示
        let prompt_block = Block::default()
            .title(" Input ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let prompt_text = Paragraph::new(prompt)
            .block(prompt_block)
            .alignment(Alignment::Center);

        f.render_widget(prompt_text, chunks[0]);

        // 输入框
        let input_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let input_text = Paragraph::new(format!("{}_", current_input))
            .block(input_block);

        f.render_widget(input_text, chunks[1]);

        // 提示信息
        let hint = Paragraph::new("Press Enter to confirm, Esc to cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(hint, chunks[2]);
    }

    /// 渲染选择对话框
    fn render_select_dialog(f: &mut Frame, area: Rect, title: &str, options: &[String], list_state: &mut ListState) {
        let dialog_area = Self::create_centered_rect(50, 60, area);
        f.render_widget(Clear, dialog_area);

        let items: Vec<ListItem> = options
            .iter()
            .map(|option| ListItem::new(Line::from(option.as_str())))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" {} ", title))
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("► ");

        f.render_stateful_widget(list, dialog_area, list_state);
    }

    /// 渲染多选对话框
    fn render_multi_select_dialog(
        f: &mut Frame, 
        area: Rect, 
        title: &str, 
        options: &[(String, bool)], 
        list_state: &mut ListState
    ) {
        let dialog_area = Self::create_centered_rect(60, 70, area);
        f.render_widget(Clear, dialog_area);

        let items: Vec<ListItem> = options
            .iter()
            .map(|(option, selected)| {
                let checkbox = if *selected { "☑ " } else { "☐ " };
                let style = if *selected {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(format!("{}{}", checkbox, option))).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" {} (Space to toggle) ", title))
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("► ");

        f.render_stateful_widget(list, dialog_area, list_state);
    }

    /// 渲染物品选择对话框
    fn render_item_select_dialog(
        f: &mut Frame, 
        area: Rect, 
        title: &str, 
        items: &[DialogItem], 
        list_state: &mut ListState
    ) {
        let dialog_area = Self::create_centered_rect(70, 80, area);
        f.render_widget(Clear, dialog_area);

        let list_items: Vec<ListItem> = items
            .iter()
            .map(|item| {
                let quantity_str = item.quantity
                    .map(|q| format!(" ({})", q))
                    .unwrap_or_default();
                
                let name_with_quantity = format!("{} {}{}", 
                    item.icon, 
                    item.name, 
                    quantity_str
                );

                let style = if item.enabled {
                    Style::default().fg(item.color)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                ListItem::new(vec![
                    Line::from(Span::styled(name_with_quantity, style)),
                    Line::from(Span::styled(
                        format!("  {}", item.description), 
                        Style::default().fg(Color::Gray)
                    )),
                ])
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .title(format!(" {} ", title))
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("► ");

        f.render_stateful_widget(list, dialog_area, list_state);
    }
}

/// 对话框管理器
pub struct DialogManager {
    current_dialog: Option<DialogState>,
}

impl DialogManager {
    pub fn new() -> Self {
        Self {
            current_dialog: None,
        }
    }

    /// 显示对话框
    pub fn show_dialog(&mut self, dialog_type: DialogType) {
        self.current_dialog = Some(DialogState::new(dialog_type));
    }

    /// 处理输入并返回结果
    pub fn handle_input(&mut self, key: KeyEvent) -> Option<DialogResult> {
        if let Some(ref mut dialog) = self.current_dialog {
            let result = dialog.handle_input(key);
            if result.is_some() {
                self.current_dialog = None; // 自动关闭对话框
            }
            result
        } else {
            None
        }
    }

    /// 渲染当前对话框
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if let Some(ref mut dialog) = self.current_dialog {
            DialogRenderer::render(dialog, f, area);
        }
    }

    /// 检查是否有活动的对话框
    pub fn has_active_dialog(&self) -> bool {
        self.current_dialog.is_some()
    }

    /// 关闭当前对话框
    pub fn close_current_dialog(&mut self) {
        self.current_dialog = None;
    }
}

impl Default for DialogManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 预设对话框创建器
pub struct DialogPresets;

impl DialogPresets {
    /// 退出确认
    pub fn quit_confirmation() -> DialogType {
        DialogType::Confirm {
            message: "Are you sure you want to quit the game?".to_string(),
            default_yes: false,
        }
    }

    /// 删除存档确认
    pub fn delete_save_confirmation(save_name: &str) -> DialogType {
        DialogType::Confirm {
            message: format!("Delete save '{}'?\nThis action cannot be undone.", save_name),
            default_yes: false,
        }
    }

    /// 输入玩家名称
    pub fn player_name_input() -> DialogType {
        DialogType::Input {
            prompt: "Enter your hero's name:".to_string(),
            current_input: String::new(),
            max_length: 16,
        }
    }

    /// 选择英雄职业
    pub fn hero_class_selection() -> DialogType {
        DialogType::Select {
            title: "Choose your class".to_string(),
            options: vec![
                "Warrior - High health and defense".to_string(),
                "Rogue - High accuracy and stealth".to_string(),
                "Mage - Powerful spells and magic".to_string(),
                "Huntress - Ranged combat specialist".to_string(),
            ],
            selected_index: 0,
        }
    }

    /// 错误信息
    pub fn error_message(message: &str) -> DialogType {
        DialogType::Error {
            message: message.to_string(),
        }
    }

    /// 信息提示
    pub fn info_message(message: &str) -> DialogType {
        DialogType::Info {
            message: message.to_string(),
        }
    }
}