//! 游戏结束界面渲染器
//!
//! 处理游戏结束（死亡、胜利）等各种结局界面的渲染。
//! 支持显示死亡原因、统计信息等。

use crate::ecs::{GameOverReason, GameStatus, Resources};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// 游戏结束界面渲染器
pub struct GameOverRenderer;

impl GameOverRenderer {
    pub fn new() -> Self {
        Self
    }

    /// 渲染游戏结束界面
    pub fn render_game_over(&self, frame: &mut Frame, area: Rect, resources: &Resources) {
        let (_reason, title_color, title_text, emoji) = match resources.game_state.game_state {
            GameStatus::GameOver { reason } => {
                let (emoji, title_color, title_text) = match reason {
                    GameOverReason::Died(cause) => ("☠️", Color::Red, format!("你死了：{}", cause)),
                    GameOverReason::Defeated(enemy) => {
                        ("⚔️", Color::Red, format!("被{}击败", enemy))
                    }
                    GameOverReason::Starved => ("🍖", Color::Yellow, "饿死在地牢中".to_string()),
                    GameOverReason::Trapped(trap) => {
                        ("🕳️", Color::Red, format!("死于陷阱：{}", trap))
                    }
                    GameOverReason::Quit => ("🚪", Color::Gray, "游戏结束".to_string()),
                };
                (reason, title_color, title_text, emoji)
            }
            _ => (
                GameOverReason::Died("未知原因"),
                Color::Red,
                "游戏结束".to_string(),
                "❓",
            ),
        };

        let game_over_area = self.centered_rect(area, 70, 80);

        // 半透明背景遮罩
        let background = Paragraph::new("").style(Style::default().bg(Color::Black));
        frame.render_widget(background, area);

        // 主布局
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // 标题区域
                Constraint::Min(8),    // 统计信息区域
                Constraint::Length(3), // 底部菜单
            ])
            .split(game_over_area);

        // 渲染标题
        self.render_title(frame, layout[0], &emoji, &title_text, title_color);

        // 渲染统计信息
        self.render_statistics(frame, layout[1], resources);

        // 渲染底部菜单
        self.render_menu(frame, layout[2]);
    }

    /// 渲染游戏胜利界面
    pub fn render_victory(&self, frame: &mut Frame, area: Rect, resources: &Resources) {
        let victory_area = self.centered_rect(area, 70, 80);

        // 背景
        let background = Paragraph::new("").style(Style::default().bg(Color::Black));
        frame.render_widget(background, area);

        // 主布局
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // 标题区域
                Constraint::Min(8),    // 统计信息区域
                Constraint::Length(3), // 底部菜单
            ])
            .split(victory_area);

        // 渲染胜利标题
        let title_lines = vec![Line::from(vec![
            Span::styled(
                "🏆",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "恭喜通关！",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "🏆",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ])];

        let title = Paragraph::new(title_lines)
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .title("═══ 🎊 胜利 🎊 ═══")
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Double)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(title, layout[0]);

        // 渲染统计信息
        self.render_statistics(frame, layout[1], resources);

        // 渲染底部菜单
        self.render_menu(frame, layout[2]);
    }

    /// 渲染标题
    fn render_title(
        &self,
        frame: &mut Frame,
        area: Rect,
        emoji: &str,
        title_text: &str,
        title_color: Color,
    ) {
        let title_lines = vec![Line::from(vec![
            Span::styled(
                emoji,
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                title_text,
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ])];

        let title = Paragraph::new(title_lines)
            .style(
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .title("═══ 💀 游戏结束 💀 ═══")
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Double)
                    .border_style(Style::default().fg(title_color)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(title, area);
    }

    /// 渲染游戏统计信息
    fn render_statistics(&self, frame: &mut Frame, area: Rect, resources: &Resources) {
        // 获取玩家统计信息（需要查询 ECS）
        let stats_lines = self.get_game_statistics(resources);

        let stats_paragraph = Paragraph::new(stats_lines)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title("═══ 📊 本局统计 ═══")
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(stats_paragraph, area);
    }

    /// 获取游戏统计信息
    fn get_game_statistics(&self, resources: &Resources) -> Vec<Line<'_>> {
        vec![
            Line::from(vec![
                Span::styled("游戏时长: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} 分钟", self.calculate_playtime(resources)),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("到达深度: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("第 {} 层", resources.game_state.depth.min(99)),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("渲染帧数: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", resources.game_state.frame_count),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::default(), // 空行
            Line::from(vec![Span::styled(
                "消息记录:",
                Style::default().fg(Color::Yellow),
            )]),
            // 显示最后几条游戏消息
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(Color::Gray)),
                Span::styled(
                    self.get_last_message(resources),
                    Style::default().fg(Color::White),
                ),
            ]),
        ]
    }

    /// 计算游戏时长（简化版本）
    fn calculate_playtime(&self, _resources: &Resources) -> u32 {
        // 这里应该使用实际的时钟数据，暂时返回模拟值
        15
    }

    /// 获取最后一条消息
    fn get_last_message(&self, resources: &Resources) -> String {
        resources
            .game_state
            .message_log
            .last()
            .cloned()
            .unwrap_or_else(|| "没有什么特别的事情发生".to_string())
    }

    /// 渲染底部菜单
    fn render_menu(&self, frame: &mut Frame, area: Rect) {
        let menu_text = "Enter: 重新开始  |  Esc: 返回主菜单  |  Q: 退出游戏";

        let menu_paragraph = Paragraph::new(menu_text)
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("═══ 选择操作 ═══")
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(Color::White)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(menu_paragraph, area);
    }

    /// 创建居中的矩形区域
    fn centered_rect(&self, r: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let popup_layout = Layout::default()
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
            .split(popup_layout[1])[1]
    }
}

impl Default for GameOverRenderer {
    fn default() -> Self {
        Self::new()
    }
}
