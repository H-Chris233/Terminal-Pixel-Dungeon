//! 主菜单渲染器
//!
//! 处理游戏主菜单、暂停菜单等界面渲染。
//! 支持中文界面和键盘导航。

use crate::ecs::{GameStatus, Resources};
use ratatui::text::Text;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// 主菜单渲染器
pub struct MenuRenderer;

impl MenuRenderer {
    pub fn new() -> Self {
        Self
    }

    /// 渲染主菜单
    pub fn render_main_menu(&self, frame: &mut Frame, area: Rect, resources: &Resources) {
        // 清空背景并显示主菜单
        let menu_items = vec![
            "🗡️  开始新游戏",
            "📦 继续游戏",
            "⚙️  游戏设置",
            "❓ 帮助说明",
            "🚪 退出游戏",
        ];

        // 计算选中项 - 使用 game_state 中的选项索引
        let selected_index = match resources.game_state.game_state {
            GameStatus::MainMenu { selected_option } => selected_option,
            _ => 0,
        };

        // 创建居中的菜单布局
        let menu_area = self.centered_rect(area, 40, 60);

        // 背景遮罩（可选）
        let background = Paragraph::new("").style(Style::default().bg(Color::Black));
        frame.render_widget(background, area);

        // 菜单标题
        let title_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题
                Constraint::Min(5),    // 菜单项
                Constraint::Length(3), // 底部提示
            ])
            .split(menu_area);

        // 渲染标题（添加 ASCII 艺术装饰）
        let title_text = vec![
            Line::from("╔═══════════════════════════════════════╗"),
            Line::from(vec![
                Span::raw("║  "),
                Span::styled(
                    "🏰 终端像素地牢 🏰",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  ║"),
            ]),
            Line::from("╚═══════════════════════════════════════╝"),
        ];

        let title = Paragraph::new(title_text)
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .title(" 版本 v0.1.0 ")
                    .title_alignment(ratatui::layout::Alignment::Right)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Double)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(title, title_layout[0]);

        // 渲染菜单项
        let menu_list: Vec<ListItem> = menu_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let (fg_color, bg_color, modifier) = if i == selected_index {
                    (
                        Color::Black,
                        Color::Yellow,
                        Modifier::BOLD | Modifier::REVERSED,
                    )
                } else {
                    (Color::Gray, Color::Reset, Modifier::empty())
                };

                let line = Line::from(Span::styled(
                    format!("  {}  ", item),
                    Style::default()
                        .fg(fg_color)
                        .bg(bg_color)
                        .add_modifier(modifier),
                ));

                ListItem::new(line)
            })
            .collect();

        let list = List::new(menu_list).block(
            Block::default()
                .title("═══ 主菜单 ═══")
                .title_alignment(ratatui::layout::Alignment::Center)
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        frame.render_widget(list, title_layout[1]);

        // 渲染底部提示
        let hint_text = "使用 ↑↓ 键导航，Enter 选择，Esc 退出";
        let hints = Paragraph::new(hint_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);

        frame.render_widget(hints, title_layout[2]);
    }

    /// 渲染暂停菜单
    pub fn render_pause_menu(&self, frame: &mut Frame, area: Rect, resources: &Resources) {
        let menu_items = vec![
            "🔄 继续游戏",
            "🎒 物品栏",
            "👤 角色信息",
            "⚙️  游戏设置",
            "❓ 帮助说明",
            "💾 保存并退出",
        ];

        // 从 game_state 获取选中的索引（如果是暂停状态）
        let selected_index = match resources.game_state.game_state {
            GameStatus::Paused { selected_option } => selected_option,
            _ => 0,
        };

        let menu_area = self.centered_rect(area, 40, 60);

        // 半透明背景
        let background = Paragraph::new("").style(Style::default().bg(Color::Black));
        frame.render_widget(background, area);

        // 暂停菜单布局
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // 标题
                Constraint::Min(6),    // 菜单项
                Constraint::Length(3), // 底部提示
            ])
            .split(menu_area);

        // 渲染标题
        let title = Paragraph::new("⏸️  游戏暂停  ⏸️")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Double)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(title, layout[0]);

        // 渲染菜单项
        let menu_list: Vec<ListItem> = menu_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == selected_index;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(Color::White)
                };

                let text = if is_selected {
                    format!("▶ {} ◀", item)
                } else {
                    format!("  {}  ", item)
                };

                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        let list = List::new(menu_list).block(
            Block::default()
                .title("═══ 选择操作 ═══")
                .title_alignment(ratatui::layout::Alignment::Center)
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        frame.render_widget(list, layout[1]);

        // 渲染底部提示
        let hint_text = "↑↓: 导航  Enter: 确认  Esc: 继续游戏";
        let hints = Paragraph::new(hint_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        frame.render_widget(hints, layout[2]);
    }

    /// 渲染选项菜单
    pub fn render_options_menu(&self, frame: &mut Frame, area: Rect, resources: &Resources) {
        let options = vec![
            ("音效", "开启"),
            ("音乐", "关闭"),
            ("按键绑定", "默认"),
            ("显示模式", "全彩"),
            ("语言", "中文"),
        ];

        let selected_index = match resources.game_state.game_state {
            GameStatus::Options { selected_option } => selected_option,
            _ => 0,
        };

        let menu_area = self.centered_rect(area, 30, 50);

        // 背景
        let background = Paragraph::new("").style(Style::default().bg(Color::Black));
        frame.render_widget(background, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // 标题
                Constraint::Min(8),    // 选项列表
                Constraint::Length(3), // 底部提示
            ])
            .split(menu_area);

        // 标题
        let title = Paragraph::new("⚙️ 游戏设置")
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);

        frame.render_widget(title, layout[0]);

        // 选项列表
        let option_list: Vec<ListItem> = options
            .iter()
            .enumerate()
            .map(|(i, (option, value))| {
                let is_selected = i == selected_index;
                let (fg, bg) = if is_selected {
                    (Color::Black, Color::Yellow)
                } else {
                    (Color::White, Color::Reset)
                };

                let option_text = if is_selected {
                    format!("▶ {}: {} ◀", option, value)
                } else {
                    format!("  {}: {}", option, value)
                };

                let line = Line::from(vec![Span::styled(
                    option_text,
                    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
                )]);

                ListItem::new(line)
            })
            .collect();

        let list =
            List::new(option_list).block(Block::default().title("设置选项").borders(Borders::ALL));

        frame.render_widget(list, layout[1]);

        // 底部提示
        let hint_text = "↑↓: 选择  Enter: 修改  Esc: 返回";
        let hints = Paragraph::new(hint_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        frame.render_widget(hints, layout[2]);
    }

    /// 渲染帮助界面
    pub fn render_help_menu(&self, frame: &mut Frame, area: Rect, _resources: &Resources) {
        let help_text = vec![
            "🎮 游戏操作指南",
            "",
            "移动控制:",
            "  h/j/k/l ←→↑↓ - 8方向移动",
            "  y/u/b/n     - 斜向移动",
            "  .           - 等待一回合",
            "",
            "战斗与交互:",
            "  Shift + 方向键 - 攻击指定方向",
            "  1-9         - 使用对应物品",
            "  d           - 丢弃物品",
            "",
            "地牢探索:",
            "  >           - 下楼梯",
            "  <           - 上楼梯",
            "",
            "游戏控制:",
            "  Esc         - 暂停/返回",
            "  q           - 退出游戏（现已增加确认对话框）",
            "",
            "",
            "🏃 快速提示:",
            "  • 关注饥饿度，定期进食",
            "  • 观察敌人行为模式",
            "  • 合理使用物品和技能",
            "  • 探索地图时注意地形",
        ];

        let help_area = self.centered_rect(area, 80, 90);

        // 背景
        let background = Paragraph::new("").style(Style::default().bg(Color::Black));
        frame.render_widget(background, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题
                Constraint::Min(10),   // 帮助内容
                Constraint::Length(2), // 底部提示
            ])
            .split(help_area);

        // 标题
        let title = Paragraph::new("❓ 帮助说明 ❓")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Double)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(title, layout[0]);

        // 帮助内容
        let help_paragraph = Paragraph::new(help_text.join("\n"))
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("═══ 操作指南 ═══")
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(help_paragraph, layout[1]);

        // 底部提示
        let hint_text = "按任意键返回";
        let hints = Paragraph::new(hint_text)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);

        frame.render_widget(hints, layout[2]);
    }

    /// 渲染确认退出对话框
    pub fn render_confirm_quit(&self, frame: &mut Frame, area: Rect, resources: &Resources) {
        let selected = match resources.game_state.game_state {
            GameStatus::ConfirmQuit {
                selected_option, ..
            } => selected_option,
            _ => 1,
        };

        let popup_area = self.centered_rect(area, 40, 30);

        // 背景遮罩
        frame.render_widget(Clear, popup_area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(popup_area);

        // 标题
        let title = Paragraph::new("确认退出？")
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        // 提示文本
        let info = Paragraph::new("是否退出到主菜单/桌面？")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(info, layout[1]);

        // 选项按钮
        let yes_style = if selected == 0 {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let no_style = if selected == 1 {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let yes = Paragraph::new(Text::styled("  是  ", yes_style))
            .block(Block::default().borders(Borders::ALL).title("确认"))
            .alignment(Alignment::Center);
        let no = Paragraph::new(Text::styled("  否  ", no_style))
            .block(Block::default().borders(Borders::ALL).title("取消"))
            .alignment(Alignment::Center);

        let buttons = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[2]);

        frame.render_widget(yes, buttons[0]);
        frame.render_widget(no, buttons[1]);
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

impl Default for MenuRenderer {
    fn default() -> Self {
        Self::new()
    }
}
