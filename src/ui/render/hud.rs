//src/ui/render/hud.rs
use crate::{hero::class::Class, hero::hero::Hero, ui::terminal::TerminalController};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

/// 像素地牢风格HUD渲染器（含完整动画系统）
pub struct HudRenderer {
    // 危险状态动画
    danger_flash: bool,
    danger_flash_timer: f32,

    // 金币动画
    gold_flash_timer: f32,
    gold_flash_alpha: f32,

    // 伤害数字动画
    damage_numbers: Vec<DamageNumber>,

    // 经验条动画
    exp_animated_ratio: f32,
    current_exp: u32,
    next_level_exp: u32,
}

/// 浮动伤害数字数据结构
#[derive(Clone)]
struct DamageNumber {
    value: i32,
    position: (u16, u16),
    lifetime: f32,
    alpha: f32,
    is_critical: bool,
    y_offset: f32,
}

impl HudRenderer {
    /// 创建新的HUD渲染器
    pub fn new() -> Self {
        Self {
            danger_flash: false,
            danger_flash_timer: 0.0,
            gold_flash_timer: 0.0,
            gold_flash_alpha: 0.0,
            damage_numbers: Vec::new(),
            exp_animated_ratio: 0.0,
            current_exp: 0,
            next_level_exp: 100,
        }
    }

    /// 主渲染方法（整合所有动画效果）
    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect, hero: &Hero) {
        // 更新动画状态（需在游戏循环中每帧调用update）
        self.current_exp = hero.exp;
        self.next_level_exp = hero.exp_to_next_level();

        // 经典四栏布局
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // 等级+职业
                Constraint::Min(10),    // 血条
                Constraint::Length(12), // 金币
                Constraint::Length(10), // 深度
            ])
            .split(area);

        // 1. 渲染等级和职业
        self.render_level(f, chunks[0], hero);

        // 2. 渲染动态血条
        self.render_health(f, chunks[1], hero);

        // 3. 渲染金币（带闪光效果）
        self.render_gold(f, chunks[2], hero);

        // 4. 渲染深度指示
        self.render_depth(f, chunks[3], hero);

        // 5. 渲染浮动伤害数字
        self.render_damage_numbers(f);
    }

    /// 更新所有动画状态（需在游戏循环中每帧调用）
    pub fn update(&mut self, delta_time: f32) {
        self.update_danger_flash(delta_time);
        self.update_gold_flash(delta_time);
        self.update_damage_numbers(delta_time);
        self.update_exp_growth(delta_time);
    }

    /// 触发金币收集动画
    pub fn trigger_gold_flash(&mut self) {
        self.gold_flash_timer = 0.5;
        self.gold_flash_alpha = 1.0;
    }

    /// 添加浮动伤害数字
    pub fn add_damage_number(&mut self, value: i32, is_critical: bool, position: (u16, u16)) {
        self.damage_numbers.push(DamageNumber {
            value,
            position,
            lifetime: 1.2,
            alpha: 1.0,
            is_critical,
            y_offset: 0.0,
        });
    }
}

// ===== 私有实现 =====
impl HudRenderer {
    fn render_level<B: Backend>(&self, f: &mut Frame<B>, area: Rect, hero: &Hero) {
        let class_icon = match hero.class {
            Class::Warrior => "⚔",
            Class::Mage => "🔮",
            Class::Rogue => "🏹",
            Class::Huntress => "🌿",
        };

        let text = Spans::from(vec![
            Span::styled(class_icon, Style::default().fg(Color::Red)),
            Span::styled(
                format!(" Lv.{}", hero.level),
                Style::default().fg(Color::Yellow),
            ),
        ]);

        let block = Block::default().borders(Borders::NONE);

        f.render_widget(
            Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center),
            area,
        );
    }

    fn render_health<B: Backend>(&self, f: &mut Frame<B>, area: Rect, hero: &Hero) {
        let ratio = hero.health as f64 / hero.max_health as f64;
        let is_danger = ratio <= 0.25;
        let label = format!("{}/{}", hero.health, hero.max_health);

        // 动态颜色（危险状态带闪烁）
        let color = if is_danger && self.danger_flash {
            Color::LightRed
        } else {
            match ratio {
                r if r > 0.6 => Color::Green,
                r if r > 0.3 => Color::Yellow,
                _ => Color::Red,
            }
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::NONE))
            .gauge_style(Style::default().fg(color))
            .percent((ratio * 100.0) as u16)
            .label(label)
            .use_unicode(true);

        f.render_widget(gauge, area);
    }

    fn render_gold<B: Backend>(&self, f: &mut Frame<B>, area: Rect, hero: &Hero) {
        let gold_style = if self.gold_flash_alpha > 0.0 {
            Style::default().fg(Color::LightYellow).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let text = Spans::from(vec![
            Span::styled("💰 ", gold_style),
            Span::styled(hero.gold.to_string(), gold_style),
        ]);

        let block = Block::default().borders(Borders::NONE);
        f.render_widget(
            Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center),
            area,
        );
    }

    fn render_depth<B: Backend>(&self, f: &mut Frame<B>, area: Rect, hero: &Hero) {
        let stairs_icon = if hero.on_stairs {
            Span::styled(
                ">",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw(" ")
        };

        let text = Spans::from(vec![
            stairs_icon,
            Span::styled(
                format!(" D.{}", hero.depth),
                Style::default().fg(Color::Blue),
            ),
        ]);

        let block = Block::default().borders(Borders::NONE);
        f.render_widget(
            Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center),
            area,
        );
    }

    fn render_damage_numbers<B: Backend>(&mut self, f: &mut Frame<B>) {
        for num in &self.damage_numbers {
            let color = if num.is_critical {
                Color::Yellow
            } else {
                Color::Red
            }
            .clone()
            .set_alpha((num.alpha * 255.0) as u8);

            let text = Span::styled(
                format!("{}", num.value),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            );

            let pos = (
                num.position.0,
                num.position.1.saturating_sub(num.y_offset as u16),
            );

            f.render_widget(Paragraph::new(text), Rect::new(pos.0, pos.1, 10, 1));
        }
    }

    fn update_danger_flash(&mut self, delta_time: f32) {
        const FLASH_INTERVAL: f32 = 0.3;
        self.danger_flash_timer += delta_time;

        if self.danger_flash_timer >= FLASH_INTERVAL {
            self.danger_flash = !self.danger_flash;
            self.danger_flash_timer = 0.0;
        }
    }

    fn update_gold_flash(&mut self, delta_time: f32) {
        if self.gold_flash_timer > 0.0 {
            self.gold_flash_timer = (self.gold_flash_timer - delta_time).max(0.0);
            self.gold_flash_alpha = (self.gold_flash_timer / 0.5).powf(2.0);
        }
    }

    fn update_damage_numbers(&mut self, delta_time: f32) {
        for num in &mut self.damage_numbers {
            num.lifetime -= delta_time;
            num.alpha = (num.lifetime / 1.2).clamp(0.0, 1.0);
            num.y_offset += delta_time * 10.0;
        }
        self.damage_numbers.retain(|n| n.lifetime > 0.0);
    }

    fn update_exp_growth(&mut self, delta_time: f32) {
        let target = self.current_exp as f32 / self.next_level_exp as f32;
        let speed = 2.0 * delta_time;

        if (self.exp_animated_ratio - target).abs() > 0.01 {
            self.exp_animated_ratio += (target - self.exp_animated_ratio).signum() * speed;
        } else {
            self.exp_animated_ratio = target;
        }
    }
}
