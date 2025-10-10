//src/ui/render/dungeon.rs
use dungeon::Dungeon;
use dungeon::level::tiles::{TerrainType, Tile, StairDirection};
use hero::Hero;
use ratatui::widgets::ListState;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// 地牢渲染器（含FOV和记忆系统）
pub struct DungeonRenderer {
    pub visible_range: u8,           // 可见范围（经典值为8）
    pub fov_algorithm: FovAlgorithm, // FOV计算算法
    pub show_all: bool,              // 调试模式显示全部
}

/// FOV算法类型（参考Roguelike视野算法设计）
pub enum FovAlgorithm {
    ShadowCasting, // 阴影投射（默认）
    DiamondWalls,  // 菱形墙算法
    RayCasting,    // 光线投射
}

impl DungeonRenderer {
    pub fn new() -> Self {
        Self {
            visible_range: 8,
            fov_algorithm: FovAlgorithm::ShadowCasting,
            show_all: false,
        }
    }

    /// 主渲染入口
    pub fn render(&self, f: &mut Frame, area: Rect, dungeon: &Dungeon, hero: &Hero) {
        let block = Block::default()
            .title(format!("Depth: {}", dungeon.depth))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(block.clone(), area);

        self.render_visible_area(f, block.inner(area), dungeon, hero);
    }

    /// 渲染可见区域（核心逻辑）
    fn render_visible_area(
        &self,
        f: &mut Frame,
        area: Rect,
        dungeon: &Dungeon,
        hero: &Hero,
    ) {
        // 计算可见区域边界
        let (left, top, right, bottom) = self.calculate_view_bounds(hero, dungeon);

        // 计算单元格尺寸
        let cell_width = area.width / (right - left + 1) as u16;
        let cell_height = area.height / (bottom - top + 1) as u16;

        // 渲染每个单元格
        for y in top..=bottom {
            for x in left..=right {
                if let Some(tile) = dungeon.current_level().get_tile(x, y) {
                    let cell_x = area.x + (x - left) as u16 * cell_width;
                    let cell_y = area.y + (y - top) as u16 * cell_height;
                    let cell_rect = Rect::new(cell_x, cell_y, cell_width, cell_height);

                    // 可见性检查
                    let is_visible = self.check_visibility(x, y, hero, dungeon);
                    let is_remembered = dungeon.current_level().is_explored(x, y);

                    // 渲染逻辑
                    if is_visible || is_remembered || self.show_all {
                        self.render_tile(
                            f,
                            cell_rect,
                            tile,
                            is_visible,
                            x == hero.x && y == hero.y,
                        );
                    }
                }
            }
        }
    }

    /// 计算可见区域边界
    fn calculate_view_bounds(&self, hero: &Hero, dungeon: &Dungeon) -> (i32, i32, i32, i32) {
        let level = dungeon.current_level();
        (
            (hero.x - self.visible_range as i32).max(0),
            (hero.y - self.visible_range as i32).max(0),
            (hero.x + self.visible_range as i32).min(level.width - 1),
            (hero.y + self.visible_range as i32).min(level.height - 1),
        )
    }

    /// 可见性检查（根据FOV算法）
    fn check_visibility(&self, x: i32, y: i32, hero: &Hero, dungeon: &Dungeon) -> bool {
        if self.show_all {
            return true;
        }

        match self.fov_algorithm {
            FovAlgorithm::ShadowCasting => self.shadow_casting_fov(x, y, hero, dungeon),
            FovAlgorithm::DiamondWalls => self.diamond_walls_fov(x, y, hero, dungeon),
            FovAlgorithm::RayCasting => self.ray_casting_fov(x, y, hero, dungeon),
        }
    }

    /// 阴影投射FOV算法（参考Roguelike视野算法）
    fn shadow_casting_fov(&self, x: i32, y: i32, hero: &Hero, dungeon: &Dungeon) -> bool {
        // 简化的八方向阴影投射实现
        let dx = (x - hero.x).abs();
        let dy = (y - hero.y).abs();
        let distance = dx.max(dy) as i32;

        if distance as u8 > self.visible_range {
            return false;
        }

        // 基础视线检查
        let mut x_step = hero.x as f32;
        let mut y_step = hero.y as f32;
        let x_diff = x as f32 - hero.x as f32;
        let y_diff = y as f32 - hero.y as f32;
        let steps = distance as f32;
        let x_inc = x_diff / steps;
        let y_inc = y_diff / steps;

        for _ in 0..distance as i32 {
            x_step += x_inc;
            y_step += y_inc;
            let check_x = x_step.round() as i32;
            let check_y = y_step.round() as i32;

            if check_x == x && check_y == y {
                break;
            }

            if let Some(tile) = dungeon.current_level().get_tile(check_x, check_y) {
                if tile.blocks_sight() {
                    return false;
                }
            }
        }

        true
    }

    /// 菱形墙FOV算法
    fn diamond_walls_fov(&self, x: i32, y: i32, hero: &Hero, dungeon: &Dungeon) -> bool {
        // 简化的菱形墙算法实现
        let dx = (x - hero.x).abs();
        let dy = (y - hero.y).abs();
        let distance = dx + dy;

        if distance > self.visible_range as i32 * 2 {
            return false;
        }

        // 基础视线检查（考虑对角线）
        let mut x_step = hero.x as f32;
        let mut y_step = hero.y as f32;
        let x_diff = x as f32 - hero.x as f32;
        let y_diff = y as f32 - hero.y as f32;
        let steps = (dx + dy) as f32;
        let x_inc = x_diff / steps;
        let y_inc = y_diff / steps;

        for _ in 0..=distance {
            x_step += x_inc;
            y_step += y_inc;
            let check_x = x_step.round() as i32;
            let check_y = y_step.round() as i32;

            if check_x == x && check_y == y {
                break;
            }

            if let Some(tile) = dungeon.current_level().get_tile(check_x, check_y) {
                if tile.blocks_sight() {
                    // 菱形墙特殊处理
                    if (check_x - hero.x).abs() <= 1
                        && (check_y - hero.y).abs() <= 1
                    {
                        continue; // 允许看到相邻墙
                    }
                    return false;
                }
            }
        }

        true
    }

    /// 光线投射FOV算法
    fn ray_casting_fov(&self, x: i32, y: i32, hero: &Hero, dungeon: &Dungeon) -> bool {
        // 简化的Bresenham算法实现
        let dx = (x - hero.x).abs();
        let dy = (y - hero.y).abs();
        let distance = dx.max(dy);

        if distance as u8 > self.visible_range {
            return false;
        }

        let mut x_step = hero.x as i32;
        let mut y_step = hero.y as i32;
        let x_inc = if hero.x < x { 1 } else { -1 };
        let y_inc = if hero.y < y { 1 } else { -1 };
        let mut error = dx - dy;

        loop {
            if x_step == x && y_step == y {
                break;
            }

            let e2 = 2 * error;
            if e2 > -dy {
                error -= dy;
                x_step += x_inc;
            }
            if e2 < dx {
                error += dx;
                y_step += y_inc;
            }

            if let Some(tile) = dungeon.current_level().get_tile(x_step, y_step) {
                if tile.blocks_sight() {
                    return false;
                }
            }
        }

        true
    }

    /// 渲染单个地图格子
    fn render_tile(
        &self,
        f: &mut Frame,
        rect: Rect,
        tile: &Tile,
        is_visible: bool,
        is_hero: bool,
    ) {
        // 经典像素地牢符号系统
        let (symbol, color) = match tile.info.terrain_type {
            TerrainType::Wall => ('#', Color::Gray),
            TerrainType::Floor => ('.', Color::Gray),
            TerrainType::Door(_) => ('+', Color::Yellow),
            TerrainType::Stair(StairDirection::Down) => ('>', Color::White),
            TerrainType::Stair(StairDirection::Up) => ('<', Color::White),
            TerrainType::Water => ('~', Color::Blue),
            TerrainType::Trap(_) => ('^', Color::Red),
            _ => (' ', Color::Reset),
        };

        // 可见性处理（记忆系统）
        let style = if is_hero {
            Style::default().fg(Color::Red) // 英雄始终高亮
        } else if is_visible {
            Style::default().fg(color) // 可见区域正常颜色
        } else {
            // 记忆区域颜色变暗处理
            let dark_color = match color {
                Color::Red => Color::Rgb(100, 0, 0),       // DarkRed
                Color::Green => Color::Rgb(0, 100, 0),     // DarkGreen
                Color::Yellow => Color::Rgb(100, 100, 0),  // DarkYellow
                Color::Blue => Color::Rgb(0, 0, 100),      // DarkBlue
                Color::Magenta => Color::Rgb(100, 0, 100), // DarkMagenta
                Color::Cyan => Color::Rgb(0, 100, 100),    // DarkCyan
                Color::Gray => Color::Rgb(50, 50, 50),     // DarkGray
                other => other,                            // 其他颜色保持不变
            };
            Style::default().fg(dark_color)
        };

        // 使用Paragraph包装Span进行渲染
        let paragraph = Paragraph::new(Span::styled(symbol.to_string(), style));
        f.render_widget(paragraph, rect);
    }
}

/// Tile扩展方法

/// 颜色扩展方法

#[test]
fn test_shadow_casting_edge() {
    let renderer = DungeonRenderer::new();
    let hero = Hero::default();
    let mut dungeon = Dungeon::generate(3, 0).unwrap();
    assert!(!renderer.shadow_casting_fov(5, 7, &hero, &dungeon)); // 验证视线阻挡
}
