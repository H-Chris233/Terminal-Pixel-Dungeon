//src/ui/render/hud.rs
use hero::class::Class;
use hero::Hero;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
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
    pub fn render(&mut self, f: &mut Frame, area: Rect, hero: &Hero) {
        // 更新动画状态（需在游戏循环中每帧调用update）
        self.current_exp = hero.experience;
        self.next_level_exp = hero.level * 100;

        // 主布局：顶部状态栏 + 底部经验/饥饿条
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // 主状态栏
                Constraint::Length(1), // 经验条和饥饿度
            ])
            .split(area);

        // 经典四栏布局
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // 等级+职业
                Constraint::Min(10),    // 血条
                Constraint::Length(12), // 金币
                Constraint::Length(10), // 深度
            ])
            .split(main_chunks[0]);

        // 1. 渲染等级和职业
        self.render_level(f, chunks[0], hero);

        // 2. 渲染动态血条
        self.render_health(f, chunks[1], hero);

        // 3. 渲染金币（带闪光效果）
        self.render_gold(f, chunks[2], hero);

        // 4. 渲染深度指示
        self.render_depth(f, chunks[3], hero);

        // 5. 渲染经验条和饥饿度
        self.render_exp_and_hunger(f, main_chunks[1], hero);

        // 6. 渲染浮动伤害数字
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
    fn render_level(&self, f: &mut Frame, area: Rect, hero: &Hero) {
        let class_icon = match hero.class {
            Class::Warrior => "⚔",
            Class::Mage => "🔮",
            Class::Rogue => "🏹",
            Class::Huntress => "🌿",
        };

        let text = Line::from(vec![
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

    fn render_health(&self, f: &mut Frame, area: Rect, hero: &Hero) {
        let ratio = hero.hp as f64 / hero.max_hp as f64;
        let is_danger = ratio <= 0.25;
        let label = format!("{}/{}", hero.hp, hero.max_hp);

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

    fn render_gold(&self, f: &mut Frame, area: Rect, hero: &Hero) {
        let gold_style = if self.gold_flash_alpha > 0.0 {
            Style::default().fg(Color::LightYellow).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let text = Line::from(vec![
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

    fn render_depth(&self, f: &mut Frame, area: Rect, _hero: &Hero) {
        let depth_value = 1; // 简化处理，默认为第1层
        let text = Line::from(vec![
            Span::styled("🏰", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!(" {}", depth_value),
                Style::default().fg(Color::White),
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

    fn render_exp_and_hunger(&self, f: &mut Frame, area: Rect, hero: &Hero) {
        // 分为两半：左边经验条，右边饥饿度
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70), // 经验条
                Constraint::Percentage(30), // 饥饿度
            ])
            .split(area);

        // 渲染经验条
        let exp_ratio = if self.next_level_exp > 0 {
            (self.current_exp as f64 / self.next_level_exp as f64).min(1.0)
        } else {
            0.0
        };

        let exp_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Magenta))
            .percent((exp_ratio * 100.0) as u16)
            .label(format!("EXP {}/{}", self.current_exp, self.next_level_exp))
            .use_unicode(true);

        f.render_widget(exp_gauge, chunks[0]);

        // 渲染饥饿度
        let hunger_ratio = (hero.satiety as f64 / 10.0).min(1.0);
        let hunger_color = match hunger_ratio {
            r if r > 0.5 => Color::Green,
            r if r > 0.25 => Color::Yellow,
            _ => Color::Red,
        };

        let hunger_icon = match hunger_ratio {
            r if r > 0.75 => "🍖",
            r if r > 0.5 => "🥩",
            r if r > 0.25 => "🍗",
            _ => "💀",
        };

        let hunger_text = Line::from(vec![
            Span::styled(hunger_icon, Style::default().fg(hunger_color)),
            Span::styled(
                format!(" {}%", (hunger_ratio * 100.0) as u16),
                Style::default().fg(hunger_color),
            ),
        ]);

        f.render_widget(
            Paragraph::new(hunger_text).alignment(Alignment::Center),
            chunks[1],
        );
    }

    fn render_damage_numbers(&mut self, f: &mut Frame) {
        for num in &self.damage_numbers {
            let color = if num.is_critical { Color::Yellow } else { Color::Red };

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
