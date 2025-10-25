//! HUD 渲染器
//!
//! 显示玩家状态信息：生命值、等级、金币、饱食度等。
//! 直接从 ECS World 读取 Player 实体的组件数据。

use crate::ecs::{Actor, Hunger, Player, PlayerProgress, Stats, Wealth};
use hecs::World;
use hero::class::Class;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};

/// HUD 渲染器
///
/// 布局：
/// ```
/// | 职业+等级 | ======= 生命值 ======= | 💰金币 | 🍖饱食度 |
/// ```
pub struct HudRenderer;

impl HudRenderer {
    pub fn new() -> Self {
        Self
    }

    /// 渲染 HUD
    pub fn render(&self, frame: &mut Frame, area: Rect, world: &World) {
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

        // 主布局：顶部状态栏 + 底部经验条
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // 主状态栏
                Constraint::Length(1), // 经验条
            ])
            .split(area);

        // 四栏布局
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(15), // 等级+职业
                Constraint::Min(20),    // 血条
                Constraint::Length(12), // 金币
                Constraint::Length(12), // 饱食度
            ])
            .split(main_chunks[0]);

        // 1. 渲染等级和职业
        self.render_level(frame, chunks[0], &stats, &progress, &actor_name);

        // 2. 渲染生命值条
        self.render_health(frame, chunks[1], &stats);

        // 3. 渲染金币
        self.render_gold(frame, chunks[2], &wealth);

        // 4. 渲染饱食度
        self.render_hunger(frame, chunks[3], &hunger);

        // 5. 渲染经验条（使用 Stats 中的经验值）
        self.render_experience(frame, main_chunks[1], &stats);
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
        let class_icon = match progress.class.clone() {
            Class::Warrior => "⚔",
            Class::Mage => "🔮",
            Class::Rogue => "🗡",
            Class::Huntress => "🏹",
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
            .gauge_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            .percent((exp_ratio * 100.0) as u16)
            .label(format!("EXP {}/{}", current_exp, next_level_exp))
            .use_unicode(true);

        frame.render_widget(exp_gauge, area);
    }
}
