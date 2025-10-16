//! 地牢渲染器
//!
//! 负责渲染地牢地图、实体（玩家、怪物、物品）和 FOV 效果。
//! 直接从 ECS World 读取数据，使用 FOVSystem 计算的可见性信息。

use crate::ecs::{Actor, Color, Player, Position, Renderable, TerrainType, Tile, Viewshed};
use hecs::World;
use ratatui::{
    Frame,
    backend::Backend,
    buffer::Buffer,
    layout::Rect,
    style::{Color as TuiColor, Style},
    widgets::{Block, Borders, Widget},
};
use std::collections::HashSet;

/// 地牢渲染器
///
/// 渲染策略：
/// 1. 从 ECS World 获取玩家的 Viewshed 组件
/// 2. 根据 visible_tiles 和 memory 渲染地图
/// 3. 可见区域渲染完整颜色，记忆区域渲染暗色
pub struct DungeonRenderer {
    /// 是否显示全部地图（调试模式）
    pub show_all: bool,
}

impl DungeonRenderer {
    pub fn new() -> Self {
        Self { show_all: false }
    }

    /// 主渲染入口
    ///
    /// 从 ECS World 读取数据并渲染到指定区域
    pub fn render(&self, frame: &mut Frame, area: Rect, world: &World) {
        let block = Block::default()
            .title("Dungeon")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(TuiColor::DarkGray));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // 创建自定义 Widget 进行渲染
        let dungeon_widget = DungeonWidget {
            world,
            show_all: self.show_all,
        };

        frame.render_widget(dungeon_widget, inner_area);
    }
}

/// 自定义 Widget 用于渲染地牢
struct DungeonWidget<'a> {
    world: &'a World,
    show_all: bool,
}

impl<'a> Widget for DungeonWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 填充背景
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf[(x, y)]
                    .set_char(' ')
                    .set_fg(TuiColor::Black)
                    .set_bg(TuiColor::Black);
            }
        }

        // 获取玩家位置和视野数据
        let (player_pos, visible_set, memory_set) = self.get_player_vision();

        // 渲染地图块（Tiles）
        for (_, (pos, tile, renderable)) in
            self.world.query::<(&Position, &Tile, &Renderable)>().iter()
        {
            // 只渲染当前层级
            if let Some(ref player_pos) = player_pos {
                if pos.z != player_pos.z {
                    continue;
                }
            }

            // 检查可见性
            let is_visible = self.show_all || visible_set.contains(&(pos.x, pos.y));
            let is_remembered = memory_set.contains(&(pos.x, pos.y));

            if !is_visible && !is_remembered {
                continue; // 未探索区域不渲染
            }

            // 计算屏幕坐标
            let screen_x = area.left() as i32 + pos.x;
            let screen_y = area.top() as i32 + pos.y;

            if screen_x < area.left() as i32
                || screen_x >= area.right() as i32
                || screen_y < area.top() as i32
                || screen_y >= area.bottom() as i32
            {
                continue; // 超出渲染区域
            }

            let cell = &mut buf[(screen_x as u16, screen_y as u16)];

            // 渲染 Tile
            let (symbol, mut color) = self.get_tile_appearance(tile);

            // 如果是记忆区域（不可见），使用暗色
            if !is_visible {
                color = self.darken_color(color);
            }

            cell.set_char(symbol).set_fg(color);
        }

        // 渲染实体（Actor：玩家和怪物）
        for (_, (pos, renderable, _actor)) in self
            .world
            .query::<(&Position, &Renderable, &Actor)>()
            .iter()
        {
            if let Some(ref player_pos) = player_pos {
                if pos.z != player_pos.z {
                    continue;
                }
            }

            // 实体只在可见时渲染
            let is_visible = self.show_all || visible_set.contains(&(pos.x, pos.y));
            if !is_visible {
                continue;
            }

            let screen_x = area.left() as i32 + pos.x;
            let screen_y = area.top() as i32 + pos.y;

            if screen_x < area.left() as i32
                || screen_x >= area.right() as i32
                || screen_y < area.top() as i32
                || screen_y >= area.bottom() as i32
            {
                continue;
            }

            let cell = &mut buf[(screen_x as u16, screen_y as u16)];
            cell.set_char(renderable.symbol)
                .set_fg(self.convert_color(&renderable.fg_color));
        }
    }
}

impl<'a> DungeonWidget<'a> {
    /// 获取玩家的视野数据
    fn get_player_vision(&self) -> (Option<Position>, HashSet<(i32, i32)>, HashSet<(i32, i32)>) {
        let mut player_pos = None;
        let mut visible_set = HashSet::new();
        let mut memory_set = HashSet::new();

        // 查找玩家实体
        for (_, (pos, viewshed, _player)) in
            self.world.query::<(&Position, &Viewshed, &Player)>().iter()
        {
            player_pos = Some(pos.clone());

            // 构建可见和记忆集合
            for visible_pos in &viewshed.visible_tiles {
                visible_set.insert((visible_pos.x, visible_pos.y));
            }
            for memory_pos in &viewshed.memory {
                memory_set.insert((memory_pos.x, memory_pos.y));
            }

            break; // 只有一个玩家
        }

        (player_pos, visible_set, memory_set)
    }

    /// 获取 Tile 的外观（符号和颜色）
    fn get_tile_appearance(&self, tile: &Tile) -> (char, TuiColor) {
        match tile.terrain_type {
            TerrainType::Wall => ('#', TuiColor::Gray),
            TerrainType::Floor => ('.', TuiColor::DarkGray),
            TerrainType::Door => ('+', TuiColor::Yellow),
            TerrainType::StairsDown => ('>', TuiColor::White),
            TerrainType::StairsUp => ('<', TuiColor::White),
            TerrainType::Water => ('~', TuiColor::Blue),
            TerrainType::Trap => ('^', TuiColor::Red),
            TerrainType::Barrel => ('O', TuiColor::Yellow),
            TerrainType::Empty => (' ', TuiColor::Black),
        }
    }

    /// 将颜色变暗（用于记忆区域）
    fn darken_color(&self, color: TuiColor) -> TuiColor {
        match color {
            TuiColor::Red => TuiColor::Rgb(100, 0, 0),
            TuiColor::Green => TuiColor::Rgb(0, 100, 0),
            TuiColor::Yellow => TuiColor::Rgb(100, 100, 0),
            TuiColor::Blue => TuiColor::Rgb(0, 0, 100),
            TuiColor::Magenta => TuiColor::Rgb(100, 0, 100),
            TuiColor::Cyan => TuiColor::Rgb(0, 100, 100),
            TuiColor::Gray => TuiColor::Rgb(60, 60, 60),
            TuiColor::DarkGray => TuiColor::Rgb(30, 30, 30),
            TuiColor::White => TuiColor::Rgb(100, 100, 100),
            other => other,
        }
    }

    /// 转换 ECS Color 到 Ratatui Color
    fn convert_color(&self, color: &Color) -> TuiColor {
        match color {
            Color::Red => TuiColor::Red,
            Color::Green => TuiColor::Green,
            Color::Yellow => TuiColor::Yellow,
            Color::Blue => TuiColor::Blue,
            Color::Magenta => TuiColor::Magenta,
            Color::Cyan => TuiColor::Cyan,
            Color::Gray => TuiColor::Gray,
            Color::DarkGray => TuiColor::DarkGray,
            Color::White => TuiColor::White,
            Color::Black => TuiColor::Black,
            Color::Reset => TuiColor::Reset,
            Color::Rgb(r, g, b) => TuiColor::Rgb(*r, *g, *b),
        }
    }
}
