//! 消息日志渲染器
//! 
//! 显示战斗、移动、物品使用等游戏消息
//! 支持颜色编码、滚动缓冲和消息类型分类

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::collections::VecDeque;

/// 消息类型，用于颜色编码
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Info,        // 一般信息 - 白色
    Success,     // 成功操作 - 绿色 
    Warning,     // 警告信息 - 黄色
    Error,       // 错误信息 - 红色
    Combat,      // 战斗信息 - 红色
    Movement,    // 移动信息 - 蓝色
    Item,        // 物品相关 - 青色
    Dungeon,     // 地牢事件 - 紫色
    Status,      // 状态效果 - 橙色
}

impl MessageType {
    /// 获取对应的颜色
    pub fn color(&self) -> Color {
        match self {
            MessageType::Info => Color::White,
            MessageType::Success => Color::Green,
            MessageType::Warning => Color::Yellow,
            MessageType::Error => Color::Red,
            MessageType::Combat => Color::LightRed,
            MessageType::Movement => Color::LightBlue,
            MessageType::Item => Color::LightCyan,
            MessageType::Dungeon => Color::LightMagenta,
            MessageType::Status => Color::Rgb(255, 165, 0), // Orange
        }
    }

    /// 获取对应的前缀符号
    pub fn prefix(&self) -> &'static str {
        match self {
            MessageType::Info => "",
            MessageType::Success => "✓ ",
            MessageType::Warning => "⚠ ",
            MessageType::Error => "✗ ",
            MessageType::Combat => "⚔ ",
            MessageType::Movement => "→ ",
            MessageType::Item => "📦 ",
            MessageType::Dungeon => "🏰 ",
            MessageType::Status => "◉ ",
        }
    }
}

/// 单条游戏消息
#[derive(Debug, Clone)]
pub struct GameMessage {
    pub content: String,
    pub msg_type: MessageType,
    pub timestamp: std::time::Instant,
    pub count: u32, // 重复次数（用于合并相同消息）
}

impl GameMessage {
    pub fn new(content: String, msg_type: MessageType) -> Self {
        Self {
            content,
            msg_type,
            timestamp: std::time::Instant::now(),
            count: 1,
        }
    }

    /// 创建信息消息
    pub fn info(content: String) -> Self {
        Self::new(content, MessageType::Info)
    }

    /// 创建成功消息
    pub fn success(content: String) -> Self {
        Self::new(content, MessageType::Success)
    }

    /// 创建警告消息
    pub fn warning(content: String) -> Self {
        Self::new(content, MessageType::Warning)
    }

    /// 创建错误消息
    pub fn error(content: String) -> Self {
        Self::new(content, MessageType::Error)
    }

    /// 创建战斗消息
    pub fn combat(content: String) -> Self {
        Self::new(content, MessageType::Combat)
    }

    /// 创建移动消息
    pub fn movement(content: String) -> Self {
        Self::new(content, MessageType::Movement)
    }

    /// 创建物品消息
    pub fn item(content: String) -> Self {
        Self::new(content, MessageType::Item)
    }

    /// 创建地牢消息
    pub fn dungeon(content: String) -> Self {
        Self::new(content, MessageType::Dungeon)
    }

    /// 创建状态消息
    pub fn status(content: String) -> Self {
        Self::new(content, MessageType::Status)
    }

    /// 格式化消息内容（包含前缀和重复计数）
    pub fn formatted_content(&self) -> String {
        let prefix = self.msg_type.prefix();
        if self.count > 1 {
            format!("{}{} (×{})", prefix, self.content, self.count)
        } else {
            format!("{}{}", prefix, self.content)
        }
    }
}

/// 消息系统 - 管理游戏消息的显示和历史
pub struct MessageSystem {
    messages: VecDeque<GameMessage>,
    max_messages: usize,
    auto_clear_duration: std::time::Duration,
}

impl MessageSystem {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::with_capacity(100),
            max_messages: 100,
            auto_clear_duration: std::time::Duration::from_secs(30),
        }
    }

    /// 添加消息
    pub fn add_message(&mut self, message: GameMessage) {
        // 尝试合并相同类型的连续消息
        if let Some(last_msg) = self.messages.back_mut() {
            if last_msg.content == message.content && 
               last_msg.msg_type == message.msg_type &&
               last_msg.timestamp.elapsed() < std::time::Duration::from_secs(5) {
                last_msg.count += 1;
                last_msg.timestamp = message.timestamp;
                return;
            }
        }

        // 添加新消息
        self.messages.push_back(message);

        // 保持消息数量限制
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }

        // 清理过期消息
        self.clear_old_messages();
    }

    /// 清理过期消息
    fn clear_old_messages(&mut self) {
        let now = std::time::Instant::now();
        while let Some(front) = self.messages.front() {
            if now.duration_since(front.timestamp) > self.auto_clear_duration {
                self.messages.pop_front();
            } else {
                break;
            }
        }
    }

    /// 获取最近的消息（用于简要显示）
    pub fn get_recent_messages(&self, count: usize) -> Vec<&GameMessage> {
        self.messages.iter().rev().take(count).collect()
    }

    /// 获取所有消息（用于完整历史）
    pub fn get_all_messages(&self) -> &VecDeque<GameMessage> {
        &self.messages
    }

    /// 清空所有消息
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

/// 消息渲染器
pub struct MessageRenderer {
    message_system: MessageSystem,
}

impl MessageRenderer {
    pub fn new() -> Self {
        Self {
            message_system: MessageSystem::new(),
        }
    }

    /// 添加消息到系统
    pub fn add_message(&mut self, message: GameMessage) {
        self.message_system.add_message(message);
    }

    /// 渲染简要消息日志（游戏界面底部）
    pub fn render_brief(&mut self, f: &mut Frame, area: Rect) {
        let recent_messages = self.message_system.get_recent_messages(3);
        
        let items: Vec<ListItem> = recent_messages
            .into_iter()
            .rev() // 最新消息在底部
            .map(|msg| {
                let style = Style::default()
                    .fg(msg.msg_type.color())
                    .add_modifier(if msg.count > 1 { Modifier::BOLD } else { Modifier::empty() });
                
                ListItem::new(Line::from(Span::styled(
                    msg.formatted_content(),
                    style
                )))
            })
            .collect();

        let messages_widget = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .title(" Messages ")
                    .style(Style::default().fg(Color::Gray))
            );

        f.render_widget(messages_widget, area);
    }

    /// 渲染完整消息历史（全屏覆盖）
    pub fn render_full(&mut self, f: &mut Frame, area: Rect) {
        let all_messages = self.message_system.get_all_messages();
        
        let items: Vec<ListItem> = all_messages
            .iter()
            .rev() // 最新消息在顶部
            .map(|msg| {
                let style = Style::default()
                    .fg(msg.msg_type.color())
                    .add_modifier(if msg.count > 1 { Modifier::BOLD } else { Modifier::empty() });
                
                // 添加时间戳
                let elapsed = msg.timestamp.elapsed().as_secs();
                let time_str = if elapsed < 60 {
                    format!("{}s", elapsed)
                } else if elapsed < 3600 {
                    format!("{}m", elapsed / 60)
                } else {
                    format!("{}h", elapsed / 3600)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("[{}] ", time_str),
                        Style::default().fg(Color::DarkGray)
                    ),
                    Span::styled(msg.formatted_content(), style),
                ]))
            })
            .collect();

        let messages_widget = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Message History (Press ESC to close) ")
                    .style(Style::default().fg(Color::White))
            );

        f.render_widget(messages_widget, area);
    }

    /// 渲染战斗消息窗口（右侧面板）
    pub fn render_combat_panel(&mut self, f: &mut Frame, area: Rect) {
        let combat_messages: Vec<&GameMessage> = self.message_system
            .get_all_messages()
            .iter()
            .filter(|msg| matches!(msg.msg_type, MessageType::Combat | MessageType::Status))
            .rev()
            .take(10)
            .collect();

        let items: Vec<ListItem> = combat_messages
            .into_iter()
            .rev()
            .map(|msg| {
                let style = Style::default()
                    .fg(msg.msg_type.color())
                    .add_modifier(if msg.count > 1 { Modifier::BOLD } else { Modifier::empty() });
                
                ListItem::new(Line::from(Span::styled(
                    msg.formatted_content(),
                    style
                )))
            })
            .collect();

        let combat_widget = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Combat Log ")
                    .style(Style::default().fg(Color::Red))
            );

        f.render_widget(combat_widget, area);
    }

    /// 获取消息系统的可变引用（用于外部添加消息）
    pub fn message_system_mut(&mut self) -> &mut MessageSystem {
        &mut self.message_system
    }

    /// 获取消息系统的不可变引用
    pub fn message_system(&self) -> &MessageSystem {
        &self.message_system
    }
}

impl Default for MessageRenderer {
    fn default() -> Self {
        Self::new()
    }
}