//src/ui/render/render.rs
use crate::terminal::TerminalController;
use dungeon::Dungeon;
use hero::Hero;
use anyhow::{Context, Result};
use ratatui::style::Color;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Line},
};

/// 主渲染系统（协调所有子渲染模块）
pub struct RenderSystem {
    pub dungeon: DungeonRenderer,
    pub hud: HudRenderer,
    pub inventory: InventoryRenderer,
    pub animation_timer: f32, // 统一动画计时器
}

impl RenderSystem {
    /// 初始化渲染系统
    pub fn new() -> Self {
        Self {
            dungeon: DungeonRenderer::new(),
            hud: HudRenderer::new(),
            inventory: InventoryRenderer::new(),
            animation_timer: 0.0,
        }
    }

    /// 主渲染流程（每帧调用）
    pub fn render_game(
        &mut self,
        terminal: &mut TerminalController,
        dungeon: &Dungeon,
        hero: &Hero,
    ) -> Result<()> {
        // 更新动画状态
        // animations placeholder

        terminal
            .draw(|f| {
                // 经典像素地牢三明治布局
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // HUD
                        Constraint::Min(10),   // 地牢
                        Constraint::Length(4), // 日志
                    ])
                    .split(f.area());

                // 按Z顺序渲染各层
                self.dungeon.render(f, chunks[1], dungeon, hero);
                self.hud.render(f, chunks[0], hero);
                // message log placeholder
            })
            .context("Failed to render game frame")
    }

    /// 物品栏专用渲染
    pub fn render_inventory(
        &mut self,
        terminal: &mut TerminalController,
        hero: &Hero,
    ) -> Result<()> {
        terminal
            .draw(|f| {
                let area = centered_rect(60, 70, f.area());
                self.inventory.render(f, area, hero);
            })
            .context("Failed to render inventory")
    }

    /// 更新所有动画状态
    fn update_animations(&mut self, _delta_time: f32) {}

    /// 消息日志渲染（带滚动缓冲）
    fn render_message_log(&self, _f: &mut ratatui::Frame, _area: Rect, messages: &[String]) {
        let _ = (_f, _area);
        let _visible_messages: Vec<Line> = messages
            .iter()
            .rev()
            .take(3)
            .rev()
            .map(|msg| {
                let color = if msg.starts_with("!") {
                    Color::Red
                } else if msg.starts_with("+") {
                    Color::Green
                } else if msg.starts_with("*") {
                    Color::Yellow
                } else {
                    Color::White
                };
                Line::from(Span::styled(msg, Style::default().fg(color)))
            })
            .collect();
    }
}

/// 辅助函数：创建居中矩形（用于弹窗）
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

pub use super::dungeon::DungeonRenderer;
pub use super::hud::HudRenderer;
pub use super::inventory::InventoryRenderer;
