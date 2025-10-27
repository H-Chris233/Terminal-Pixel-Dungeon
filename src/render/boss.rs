// src/render/boss.rs
//! Boss UI 渲染模块
//! 负责渲染 Boss 血条、阶段指示器和技能提示

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};

use combat::boss::{BossPhase, BossType};

/// Boss UI 渲染器
pub struct BossUI;

impl BossUI {
    /// 渲染 Boss 血条和信息
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        boss_name: &str,
        boss_type: &BossType,
        hp: u32,
        max_hp: u32,
        phase: &BossPhase,
        shield: u32,
    ) {
        // 计算血量百分比
        let hp_percent = if max_hp > 0 {
            (hp as f64 / max_hp as f64 * 100.0) as u16
        } else {
            0
        };

        // 根据血量百分比选择颜色
        let hp_color = if hp_percent > 66 {
            Color::Green
        } else if hp_percent > 33 {
            Color::Yellow
        } else {
            Color::Red
        };

        // 创建布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Boss 名称和阶段
                Constraint::Length(3), // 血条
                Constraint::Length(2), // 护盾（如果有）
            ])
            .split(area);

        // 渲染 Boss 名称和阶段
        let phase_text = match phase {
            BossPhase::Phase1 => "第一阶段",
            BossPhase::Phase2 => "第二阶段",
            BossPhase::Enraged => "狂暴状态",
        };

        let boss_color = boss_type.color();
        let title_line = Line::from(vec![
            Span::styled(
                boss_name,
                Style::default()
                    .fg(Color::Rgb(boss_color.0, boss_color.1, boss_color.2))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - "),
            Span::styled(
                phase_text,
                Style::default()
                    .fg(Self::get_phase_color(phase))
                    .add_modifier(Modifier::ITALIC),
            ),
        ]);

        let title_block = Paragraph::new(title_line)
            .block(Block::default().borders(Borders::ALL).border_style(
                Style::default().fg(Color::Rgb(boss_color.0, boss_color.1, boss_color.2)),
            ))
            .alignment(Alignment::Center);

        frame.render_widget(title_block, chunks[0]);

        // 渲染血条
        let hp_label = format!("{} / {}", hp, max_hp);
        let hp_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("生命值"))
            .gauge_style(Style::default().fg(hp_color))
            .percent(hp_percent)
            .label(hp_label);

        frame.render_widget(hp_gauge, chunks[1]);

        // 渲染护盾（如果有）
        if shield > 0 {
            let shield_text = format!("护盾: {}", shield);
            let shield_paragraph = Paragraph::new(shield_text)
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center);
            frame.render_widget(shield_paragraph, chunks[2]);
        }
    }

    /// 渲染 Boss 房间入口提示
    pub fn render_entrance_warning(frame: &mut Frame, area: Rect, boss_name: &str) {
        let warning_text = vec![
            Line::from("╔══════════════════════════════════════╗"),
            Line::from("║                                      ║"),
            Line::from(vec![
                Span::raw("║      "),
                Span::styled(
                    "⚠️  WARNING: BOSS AHEAD  ⚠️",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("      ║"),
            ]),
            Line::from("║                                      ║"),
            Line::from(vec![
                Span::raw("║         "),
                Span::styled(
                    boss_name,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("          ║"),
            ]),
            Line::from("║                                      ║"),
            Line::from("╚══════════════════════════════════════╝"),
        ];

        let warning_block = Paragraph::new(warning_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .title("Boss 房间"),
            )
            .alignment(Alignment::Center);

        frame.render_widget(warning_block, area);
    }

    /// 渲染 Boss 技能提示
    pub fn render_skill_notification(
        frame: &mut Frame,
        area: Rect,
        skill_name: &str,
        description: &str,
    ) {
        let skill_text = vec![
            Line::from(vec![
                Span::styled(
                    "技能使用: ",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(skill_name, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(description),
        ];

        let skill_block = Paragraph::new(skill_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(skill_block, area);
    }

    /// 获取阶段对应的颜色
    fn get_phase_color(phase: &BossPhase) -> Color {
        match phase {
            BossPhase::Phase1 => Color::Green,
            BossPhase::Phase2 => Color::Yellow,
            BossPhase::Enraged => Color::Red,
        }
    }

    /// 渲染 Boss 击败奖励提示
    pub fn render_defeat_reward(
        frame: &mut Frame,
        area: Rect,
        boss_name: &str,
        is_first_kill: bool,
        gold: u32,
        items: u32,
    ) {
        let mut reward_text = vec![
            Line::from(vec![
                Span::styled(
                    "Victory! ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("你击败了 "),
                Span::styled(boss_name, Style::default().fg(Color::Yellow)),
                Span::raw("!"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("获得金币: "),
                Span::styled(format!("{}", gold), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("获得物品: "),
                Span::styled(format!("{} 件", items), Style::default().fg(Color::Cyan)),
            ]),
        ];

        if is_first_kill {
            reward_text.push(Line::from(""));
            reward_text.push(Line::from(vec![Span::styled(
                "★ 首杀奖励 ★",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )]));
            reward_text.push(Line::from("获得特殊奖励！"));
        }

        let reward_block = Paragraph::new(reward_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .title("战斗胜利"),
            )
            .alignment(Alignment::Center);

        frame.render_widget(reward_block, area);
    }
}
