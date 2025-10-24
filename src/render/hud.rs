//! HUD 渲染器
//!
//! 显示玩家状态信息：生命值、等级、金币、饱食度等。
//! 直接从 ECS World 读取 Player 实体的组件数据。

use crate::ecs::Color as GameColor;
use crate::ecs::{
    Actor, Faction, GameState, Hunger, Player, PlayerProgress, Stats, TurnHudState, TurnQueueEntry,
    Wealth,
};
use hecs::World;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};

/// HUD 渲染器
///
/// 布局：
/// ```text
/// | 职业+等级 | ======= 生命值 ======= | 💰金币 | 🍖饱食度 |
/// ```
pub struct HudRenderer;

impl HudRenderer {
    pub fn new() -> Self {
        Self
    }

    /// 渲染 HUD
    pub fn render(&self, frame: &mut Frame, area: Rect, world: &World, game_state: &GameState) {
        // 获取玩家数据
        let player_data = self.get_player_data(world);

        if player_data.is_none() {
            // 没有玩家数据，渲染空 HUD
            let text = Paragraph::new("⚠️ 未找到玩家数据")
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            frame.render_widget(text, area);
            return;
        }

        let (stats, wealth, hunger, progress, actor_name) = player_data.unwrap();

        // 主布局：顶部状态栏 + 底部经验条 + 回合信息
        let mut constraints = vec![
            Constraint::Length(2), // 主状态栏
            Constraint::Length(1), // 经验条
        ];
        if area.height > 3 {
            constraints.push(Constraint::Min(1)); // 回合信息区域
        }

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let status_bar_area = main_chunks[0];

        // 顶部四栏布局
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(15), // 等级+职业
                Constraint::Min(20),    // 血条
                Constraint::Length(12), // 金币
                Constraint::Length(12), // 饱食度
            ])
            .split(status_bar_area);

        // 1. 渲染等级和职业
        if let Some(area) = top_chunks.get(0) {
            self.render_level(frame, *area, &stats, &progress, &actor_name);
        }

        // 2. 渲染生命值条
        if let Some(area) = top_chunks.get(1) {
            self.render_health(frame, *area, &stats);
        }

        // 3. 渲染金币
        if let Some(area) = top_chunks.get(2) {
            self.render_gold(frame, *area, &wealth);
        }

        // 4. 渲染饱食度
        if let Some(area) = top_chunks.get(3) {
            self.render_hunger(frame, *area, &hunger);
        }

        // 渲染经验条（使用 Stats 中的经验值）
        if let Some(exp_area) = main_chunks.get(1) {
            self.render_experience(frame, *exp_area, &stats);
        }

        // 渲染回合与队列信息
        if let Some(turn_area) = main_chunks.get(2) {
            self.render_turn_section(frame, *turn_area, game_state);
        }
    }

    /// 从 ECS World 获取玩家数据
    fn get_player_data(
        &self,
        world: &World,
    ) -> Option<(Stats, Wealth, Hunger, PlayerProgress, String)> {
        for (_, (stats, wealth, hunger, progress, actor, _player)) in world
            .query::<(&Stats, &Wealth, &Hunger, &PlayerProgress, &Actor, &Player)>()
            .iter()
        {
            return Some((
                stats.clone(),
                wealth.clone(),
                hunger.clone(),
                progress.clone(),
                actor.name.clone(),
            ));
        }
        None
    }

    fn render_level(
        &self,
        frame: &mut Frame,
        area: Rect,
        stats: &Stats,
        progress: &PlayerProgress,
        _name: &str,
    ) {
        let class_icon = match progress.class.as_str() {
            "Warrior" => "⚔",
            "Mage" => "🔮",
            "Rogue" => "🗡",
            "Huntress" => "🏹",
            _ => "👤",
        };

        let text = Line::from(vec![
            Span::styled(class_icon, Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(
                format!("Lv.{}", stats.level),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), area);
    }

    fn render_health(&self, frame: &mut Frame, area: Rect, stats: &Stats) {
        let ratio = stats.hp as f64 / stats.max_hp.max(1) as f64;
        let label = format!("{}/{}", stats.hp, stats.max_hp);

        // 根据生命值比例选择颜色
        let color = match ratio {
            r if r > 0.6 => Color::Green,
            r if r > 0.3 => Color::Yellow,
            _ => Color::Red,
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .percent((ratio * 100.0).min(100.0) as u16)
            .label(label)
            .use_unicode(true);

        frame.render_widget(gauge, area);
    }

    fn render_gold(&self, frame: &mut Frame, area: Rect, wealth: &Wealth) {
        let text = Line::from(vec![
            Span::styled("💰 ", Style::default().fg(Color::Yellow)),
            Span::styled(
                wealth.gold.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), area);
    }

    fn render_hunger(&self, frame: &mut Frame, area: Rect, hunger: &Hunger) {
        // 饱食度 0-10，显示图标
        let (icon, color) = match hunger.satiety {
            9..=10 => ("🍖", Color::Green),   // 饱食
            6..=8 => ("🍗", Color::Yellow),   // 正常
            3..=5 => ("🥩", Color::LightRed), // 饥饿
            _ => ("💀", Color::Red),          // 饥饿至极
        };

        let text = Line::from(vec![
            Span::styled(icon, Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(hunger.satiety.to_string(), Style::default().fg(color)),
        ]);

        frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), area);
    }

    fn render_experience(&self, frame: &mut Frame, area: Rect, stats: &Stats) {
        // 计算经验值比例（简单估算：下一级需要 level * 100 经验）
        let current_exp = stats.experience;
        let next_level_exp = stats.level * 100;

        let exp_ratio = if next_level_exp > 0 {
            (current_exp as f64 / next_level_exp as f64).min(1.0)
        } else {
            0.0
        };

        let exp_gauge = Gauge::default()
            .gauge_style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )
            .percent((exp_ratio * 100.0) as u16)
            .label(format!("EXP {}/{}", current_exp, next_level_exp))
            .use_unicode(true);

        frame.render_widget(exp_gauge, area);
    }

    fn render_turn_section(&self, frame: &mut Frame, area: Rect, game_state: &GameState) {
        let turn_state = &game_state.turn_overlay;
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        let summary_area = columns.get(0).copied().unwrap_or(area);
        let queue_area = columns.get(1).copied().unwrap_or(area);

        let summary_lines = self.build_summary_lines(turn_state);
        let summary = Paragraph::new(summary_lines)
            .block(Block::default().title("回合状态").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        frame.render_widget(summary, summary_area);

        let queue_lines = self.build_queue_lines(turn_state);
        let queue = Paragraph::new(queue_lines)
            .block(Block::default().title("行动队列").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        frame.render_widget(queue, queue_area);
    }

    fn build_summary_lines(&self, turn_state: &TurnHudState) -> Vec<Line> {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![Span::styled(
            format!("回合 {}", turn_state.turn_count),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));

        if let Some(active) = &turn_state.current_actor {
            let color = self.faction_color(&active.faction);
            lines.push(Line::from(vec![
                Span::styled("当前: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    active.name.clone(),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
            ]));
            let annotation = self.queue_annotation(active);
            lines.push(Line::from(vec![Span::styled(
                format!(
                    "{} {}/{} {}",
                    self.energy_bar(active.energy, active.max_energy, 12),
                    active.energy,
                    active.max_energy,
                    annotation
                ),
                Style::default().fg(Color::White),
            )]));
        } else {
            lines.push(Line::from(vec![Span::styled(
                "当前: ---",
                Style::default().fg(Color::DarkGray),
            )]));
        }

        if !turn_state.status_feed.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "状态:",
                Style::default().fg(Color::Gray),
            )]));
            for entry in turn_state.status_feed.iter().rev().take(3) {
                let color = self.map_status_color(&entry.color);
                lines.push(Line::from(vec![
                    Span::styled("• ", Style::default().fg(color)),
                    Span::styled(entry.message.clone(), Style::default().fg(color)),
                ]));
            }
        }

        lines
    }

    fn build_queue_lines(&self, turn_state: &TurnHudState) -> Vec<Line> {
        if turn_state.queue.is_empty() {
            return vec![Line::from("暂无行动实体")];
        }

        let mut lines = Vec::new();
        for (index, entry) in turn_state.queue.iter().take(6).enumerate() {
            let color = self.faction_color(&entry.faction);
            let annotation = self.queue_annotation(entry);
            let bar = self.energy_bar(entry.energy, entry.max_energy, 10);

            let mut marker_style = Style::default().fg(color);
            let mut name_style = Style::default().fg(color);
            if index == 0 {
                marker_style = marker_style.add_modifier(Modifier::BOLD);
                name_style = name_style.add_modifier(Modifier::BOLD);
            }

            let marker = if index == 0 {
                "▶"
            } else if entry.eta == 0 {
                "●"
            } else {
                "…"
            };

            lines.push(Line::from(vec![
                Span::styled(marker, marker_style),
                Span::raw(" "),
                Span::styled(format!("{:<12}", entry.name), name_style),
                Span::raw(" "),
                Span::styled(bar, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(
                    format!("{}/{} {}", entry.energy, entry.max_energy, annotation),
                    Style::default().fg(Color::Gray),
                ),
            ]));
        }

        lines
    }

    fn energy_bar(&self, current: u32, max: u32, width: usize) -> String {
        if width == 0 || max == 0 {
            return " ".repeat(width);
        }
        let filled = ((current as f64 / max as f64) * width as f64).round() as usize;
        let filled = filled.min(width);
        let empty = width.saturating_sub(filled);
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }

    fn queue_annotation(&self, entry: &TurnQueueEntry) -> String {
        let mut label = if entry.eta == 0 {
            if entry.max_energy > 0 && entry.energy >= entry.max_energy {
                "ready".to_string()
            } else {
                "charging".to_string()
            }
        } else {
            format!("eta {}", entry.eta)
        };

        if let Some(action) = &entry.queued_action {
            if !action.is_empty() {
                if !label.is_empty() {
                    label.push_str(" – ");
                }
                label.push_str(action);
            }
        }

        label
    }

    fn faction_color(&self, faction: &Faction) -> Color {
        match faction {
            Faction::Player => Color::Yellow,
            Faction::Enemy => Color::Red,
            Faction::Neutral => Color::Cyan,
        }
    }

    fn map_status_color(&self, color: &GameColor) -> Color {
        match color {
            GameColor::Red => Color::Red,
            GameColor::Green => Color::Green,
            GameColor::Yellow => Color::Yellow,
            GameColor::Blue => Color::Blue,
            GameColor::Magenta => Color::Magenta,
            GameColor::Cyan => Color::Cyan,
            GameColor::Gray => Color::Gray,
            GameColor::DarkGray => Color::DarkGray,
            GameColor::White => Color::White,
            GameColor::Black => Color::Black,
            GameColor::Reset => Color::Reset,
            GameColor::Rgb(r, g, b) => Color::Rgb(*r, *g, *b),
        }
    }
}
