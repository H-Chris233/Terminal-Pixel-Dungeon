//! 职业选择渲染器
//!
//! 处理职业选择界面的渲染，展示职业描述、属性预览、初始装备和技能提示

use crate::ecs::{GameStatus, Resources};
use hero::class::Class;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// 职业选择渲染器
pub struct ClassSelectionRenderer;

impl ClassSelectionRenderer {
    pub fn new() -> Self {
        Self
    }

    /// 渲染职业选择界面
    pub fn render(&self, frame: &mut Frame, area: Rect, resources: &Resources) {
        let cursor = match resources.game_state.game_state {
            GameStatus::ClassSelection { cursor } => cursor,
            _ => 0,
        };

        // 清空背景
        let background = Paragraph::new("").style(Style::default().bg(Color::Black));
        frame.render_widget(background, area);

        // 创建居中的选择区域
        let selection_area = self.centered_rect(area, 90, 85);

        // 主布局：标题 + 职业列表 + 详情面板 + 底部提示
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题
                Constraint::Min(15),   // 内容区域
                Constraint::Length(3), // 底部提示
            ])
            .split(selection_area);

        // 渲染标题
        self.render_title(frame, main_layout[0]);

        // 内容区域：职业列表（左） + 详情面板（右）
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // 职业列表
                Constraint::Percentage(70), // 详情面板
            ])
            .split(main_layout[1]);

        // 渲染职业列表
        self.render_class_list(frame, content_layout[0], cursor);

        // 渲染选中职业的详细信息
        self.render_class_details(frame, content_layout[1], cursor);

        // 渲染底部提示
        self.render_hints(frame, main_layout[2]);
    }

    /// 渲染标题
    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title_text = vec![Line::from(vec![
            Span::styled("⚔️  ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "选择你的英雄职业",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ⚔️", Style::default().fg(Color::Yellow)),
        ])];

        let title = Paragraph::new(title_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Double)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(title, area);
    }

    /// 渲染职业列表
    fn render_class_list(&self, frame: &mut Frame, area: Rect, cursor: usize) {
        let classes = vec![
            ("战士", "⚔️", Color::Red),
            ("法师", "🔮", Color::Blue),
            ("盗贼", "🗡️", Color::Green),
            ("女猎手", "🏹", Color::Yellow),
        ];

        let class_items: Vec<ListItem> = classes
            .iter()
            .enumerate()
            .map(|(i, (name, icon, color))| {
                let is_selected = i == cursor;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(*color)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(*color)
                };

                let text = if is_selected {
                    format!("▶ {} {} ◀", icon, name)
                } else {
                    format!("  {} {}  ", icon, name)
                };

                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        let list = List::new(class_items).block(
            Block::default()
                .title("═══ 职业列表 ═══")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(Color::White)),
        );

        frame.render_widget(list, area);
    }

    /// 渲染职业详细信息
    fn render_class_details(&self, frame: &mut Frame, area: Rect, cursor: usize) {
        let class = match cursor {
            0 => Class::Warrior,
            1 => Class::Mage,
            2 => Class::Rogue,
            3 => Class::Huntress,
            _ => Class::Warrior,
        };

        // 详情面板布局：描述 + 属性 + 装备
        let details_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),  // 职业描述
                Constraint::Length(10), // 基础属性
                Constraint::Min(6),     // 初始装备
            ])
            .split(area);

        // 渲染描述
        self.render_description(frame, details_layout[0], &class);

        // 渲染属性
        self.render_stats(frame, details_layout[1], &class);

        // 渲染初始装备
        self.render_starting_kit(frame, details_layout[2], &class);
    }

    /// 渲染职业描述
    fn render_description(&self, frame: &mut Frame, area: Rect, class: &Class) {
        let (title, desc, special) = match class {
            Class::Warrior => (
                "战士 - 坚韧的近战专家",
                "战士拥有最高的生命值和防御力，是团队的前排坦克。\n他们在近战战斗中表现出色，能够承受大量伤害。",
                "特性：高生命值 | 平衡攻防 | 近战优势",
            ),
            Class::Mage => (
                "法师 - 强大的魔法大师",
                "法师精通各种魔法，虽然生命值较低，但能造成毁灭性的\n魔法伤害。需要谨慎保持距离作战。",
                "特性：低生命值 | 高魔法伤害 | 魔法特化",
            ),
            Class::Rogue => (
                "盗贼 - 致命的暗影刺客",
                "盗贼擅长偷袭和暴击，拥有最高的暴击率。在暗处发动\n攻击时能造成惊人的伤害。",
                "特性：高暴击率 | 速攻专家 | 潜行优势",
            ),
            Class::Huntress => (
                "女猎手 - 精准的远程射手",
                "女猎手是远程战斗的专家，能够在安全距离攻击敌人。\n精通弓箭和自然魔法。",
                "特性：远程攻击 | 自然亲和 | 精准射击",
            ),
        };

        let text = vec![
            Line::from(Span::styled(
                title,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(desc),
            Line::from(""),
            Line::from(Span::styled(special, Style::default().fg(Color::Cyan))),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title("═══ 职业介绍 ═══")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    /// 渲染基础属性
    fn render_stats(&self, frame: &mut Frame, area: Rect, class: &Class) {
        let base_hp = class.base_hp();
        let hp_per_level = class.hp_per_level();
        let attack_mod = class.attack_mod();
        let defense_mod = class.defense_mod();
        let crit_mod = class.crit_mod();

        let stats_text = vec![
            Line::from(vec![
                Span::styled("❤️  生命值：", Style::default().fg(Color::Red)),
                Span::styled(
                    format!("{} ", base_hp),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("(+{}/级)", hp_per_level),
                    Style::default().fg(Color::Gray),
                ),
            ]),
            Line::from(vec![
                Span::styled("⚔️  攻击力：", Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}x ", attack_mod),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("基础伤害", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("🛡️  防御力：", Style::default().fg(Color::Blue)),
                Span::styled(
                    format!("{}x ", defense_mod),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("基础防御", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("💥 暴击率：", Style::default().fg(Color::Magenta)),
                Span::styled(
                    format!("{}% ", (crit_mod * 100.0) as u32),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("额外加成", Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                class.description(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::ITALIC),
            )),
        ];

        let stats = Paragraph::new(stats_text)
            .block(
                Block::default()
                    .title("═══ 基础属性 ═══")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(stats, area);
    }

    /// 渲染初始装备
    fn render_starting_kit(&self, frame: &mut Frame, area: Rect, class: &Class) {
        let kit = class.starting_kit();

        let mut kit_lines = vec![
            Line::from(Span::styled(
                "初始装备清单：",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        for item in kit.iter() {
            let (icon, color) = match &item.kind {
                items::ItemKind::Weapon(_) => ("⚔️", Color::Red),
                items::ItemKind::Armor(_) => ("🛡️", Color::Blue),
                items::ItemKind::Potion(_) => ("🧪", Color::Magenta),
                items::ItemKind::Scroll(_) => ("📜", Color::Cyan),
                items::ItemKind::Seed(_) => ("🌱", Color::Green),
                _ => ("📦", Color::White),
            };

            kit_lines.push(Line::from(vec![
                Span::styled(format!("{}  ", icon), Style::default().fg(color)),
                Span::styled(item.name(), Style::default().fg(Color::White)),
            ]));
        }

        let kit_paragraph = Paragraph::new(kit_lines)
            .block(
                Block::default()
                    .title("═══ 初始装备 ═══")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(kit_paragraph, area);
    }

    /// 渲染底部提示
    fn render_hints(&self, frame: &mut Frame, area: Rect) {
        let hint_text = "↑↓: 选择职业  Enter: 确认  Esc: 返回主菜单";
        let hints = Paragraph::new(hint_text)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);

        frame.render_widget(hints, area);
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

impl Default for ClassSelectionRenderer {
    fn default() -> Self {
        Self::new()
    }
}
