// src/dungeon/src/level/level.rs

use bincode::{Decode, Encode};
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg32;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod rooms;
pub mod tiles;

use crate::level::tiles::{DoorState, StairDirection, TerrainType, Tile, TileInfo};
use crate::trap::{Trap, TrapKind};
use crate::TrapEffect;
use combat::enemy::{Enemy, EnemyKind};
use items::{
    Armor, Food, Item, ItemKind, MiscItem, Potion, Ring, Scroll, Seed, Stone, Wand, Weapon,
};

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Level {
    pub rooms: Vec<Room>,
    pub corridors: Vec<Corridor>,
    pub enemies: Vec<Enemy>,
    pub items: Vec<Item>,
    pub stair_down: (i32, i32),
    pub stair_up: (i32, i32),
    pub tiles: Vec<Tile>,
    pub width: i32,
    pub height: i32,
    pub visible_tiles: HashSet<(i32, i32)>,
    pub explored_tiles: HashSet<(i32, i32)>,
}

impl Level {
    /// 生成一个新的地牢层级
    pub fn generate(seed: u64) -> anyhow::Result<Self> {
        let mut rng = Pcg32::seed_from_u64(seed);
        let width = rng.random_range(50..100);
        let height = rng.random_range(50..100);

        // 初始化所有瓦片为墙壁
        let mut tiles = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                tiles.push(Tile::new(
                    x,
                    y,
                    TileInfo::new(
                        false, // passable
                        true,  // blocks_sight
                        TerrainType::Wall,
                    ),
                ));
            }
        }

        // 生成房间和走廊
        let (rooms, corridors) = Self::generate_dungeon_layout(&mut rng, width, height);

        // 放置楼梯
        let stair_up = rooms.first().map(|r| r.center()).unwrap_or((1, 1));
        let stair_down = rooms
            .last()
            .map(|r| r.center())
            .unwrap_or((width - 2, height - 2));

        // 放置敌人和物品
        let (enemies, items) = Self::place_entities(&mut rng, &rooms);

        // 创建地牢实例
        let mut level = Self {
            rooms,
            corridors,
            enemies,
            items,
            stair_down,
            stair_up,
            tiles,
            width,
            height,
            visible_tiles: HashSet::new(),
            explored_tiles: HashSet::new(),
        };

        // 应用生成的布局到瓦片
        level.apply_layout_to_tiles(&mut rng);

        Ok(level)
    }

    /// 将生成的布局应用到瓦片
    fn apply_layout_to_tiles(&mut self, rng: &mut impl Rng) {
        // 先收集所有需要修改的坐标
        let mut positions = Vec::new();

        // 处理房间
        for room in &self.rooms {
            for y in room.y..room.y + room.height {
                for x in room.x..room.x + room.width {
                    positions.push((x, y, TerrainType::Floor));
                }
            }
        }

        // 处理走廊
        for corridor in &self.corridors {
            for &(x, y) in &corridor.tiles {
                positions.push((x, y, TerrainType::Floor));
            }
        }

        // 批量修改瓦片
        for (x, y, terrain) in positions {
            if let Some(tile) = self.get_tile_mut(x, y) {
                tile.info = TileInfo::new(true, false, terrain);
            }
        }

        // 处理门和陷阱（同样先收集位置）
        let mut door_positions = Vec::new();
        let mut trap_positions = Vec::new();

        for room in &self.rooms {
            if rng.random_bool(0.5) {
                door_positions.push(room.random_point_on_edge(rng));
            }

            if rng.random_bool(0.3) {
                trap_positions.push(room.random_point(rng));
            }
        }

        // 批量处理门
        for (x, y) in door_positions {
            if let Some(tile) = self.get_tile_mut(x, y) {
                tile.info.terrain_type = TerrainType::Door(DoorState::Closed);
                tile.info.passable = false;
                tile.info.blocks_sight = true;
            }
        }

        // 批量处理陷阱
        for (x, y) in trap_positions {
            if let Some(tile) = self.get_tile_mut(x, y) {
                let trap_type = match rng.random_range(0..10) {
                    0 => TrapKind::Dart {
                        damage: rng.random_range(1..5),
                    },
                    1 => TrapKind::Poison {
                        damage: rng.random_range(1..3),
                        duration: rng.random_range(3..6),
                    },
                    2 => TrapKind::Alarm,
                    3 => TrapKind::Teleport,
                    4 => TrapKind::Paralyze {
                        duration: rng.random_range(2..5),
                    },
                    5 => TrapKind::Summon,
                    6 => TrapKind::Fire {
                        damage: rng.random_range(2..6),
                    },
                    7 => TrapKind::Pitfall,
                    8 => TrapKind::Gripping {
                        duration: rng.random_range(2..4),
                    },
                    _ => TrapKind::Disarming,
                };
                tile.info.terrain_type = TerrainType::Trap(Trap::from(trap_type));
            }
        }

        // 处理楼梯（这部分不需要修改，因为不涉及迭代）
        if let Some(tile) = self.get_tile_mut(self.stair_up.0, self.stair_up.1) {
            tile.info.terrain_type = TerrainType::Stair(StairDirection::Up);
        }
        if let Some(tile) = self.get_tile_mut(self.stair_down.0, self.stair_down.1) {
            tile.info.terrain_type = TerrainType::Stair(StairDirection::Down);
        }
    }

    /// 生成地牢布局 (房间和走廊)
    fn generate_dungeon_layout(
        rng: &mut impl Rng,
        width: i32,
        height: i32,
    ) -> (Vec<Room>, Vec<Corridor>) {
        let mut rooms = Vec::new();
        let mut corridors = Vec::new();

        let room_count = rng.random_range(5..10);

        for _ in 0..room_count {
            let room_width = rng.random_range(5..12);
            let room_height = rng.random_range(5..12);
            let x = rng.random_range(1..(width - room_width - 1));
            let y = rng.random_range(1..(height - room_height - 1));

            let new_room = Room::new(x, y, room_width, room_height);

            let overlaps = rooms.iter().any(|other| new_room.intersects(other));

            if !overlaps {
                // 保存中心点用于连接
                let new_center = new_room.center();
                rooms.push(new_room);

                if rooms.len() > 1 {
                    let prev_center = rooms[rooms.len() - 2].center();
                    corridors.push(Corridor::new(prev_center, new_center));
                }
            }
        }

        (rooms, corridors)
    }

    /// 放置敌人和物品
    fn place_entities(rng: &mut impl Rng, rooms: &[Room]) -> (Vec<Enemy>, Vec<Item>) {
        let mut enemies = Vec::new();
        let mut items = Vec::new();

        // 跳过第一个房间(玩家出生点)
        for (i, room) in rooms.iter().enumerate().skip(1) {
            // 放置1-3个敌人
            let enemy_count = rng.random_range(1..=3);
            for _ in 0..enemy_count {
                let (x, y) = room.random_point(rng);

                // 根据深度决定敌人类型
                let kind = match i {
                    0..=2 => EnemyKind::Rat,
                    3..=5 => EnemyKind::Snake,
                    6..=8 => EnemyKind::Gnoll,
                    _ => EnemyKind::default(),
                };

                enemies.push(Enemy::new(kind, x, y));
            }

            // 10%几率放置物品
            if rng.random_bool(0.1) {
                let (x, y) = room.random_point(rng);

                // 创建随机物品
                let item = match rng.random_range(0..10) {
                    0 => Item::new(ItemKind::Weapon(Weapon::random_new())),
                    1 => Item::new(ItemKind::Armor(Armor::random_new())),
                    2 => Item::new(ItemKind::Potion(Potion::random_new())),
                    3 => Item::new(ItemKind::Scroll(Scroll::random_new())),
                    4 => Item::new(ItemKind::Food(Food::random_new())),
                    5 => Item::new(ItemKind::Wand(Wand::random_new())),
                    6 => Item::new(ItemKind::Ring(Ring::random_new())),
                    7 => Item::new(ItemKind::Seed(Seed::random_new())),
                    8 => Item::new(ItemKind::Stone(Stone::random_new())),
                    _ => Item::new(ItemKind::Misc(MiscItem::random_new())),
                };

                // 设置物品位置
                let mut item = item;
                item.x = x;
                item.y = y;
                items.push(item);
            }
        }

        (enemies, items)
    }

    /// 获取指定位置的瓦片(可变引用)
    pub fn get_tile_mut(&mut self, x: i32, y: i32) -> Option<&mut Tile> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        self.tiles.iter_mut().find(|t| t.x == x && t.y == y)
    }

    /// 获取指定位置的瓦片
    pub fn get_tile(&self, x: i32, y: i32) -> Option<&Tile> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        self.tiles.iter().find(|t| t.x == x && t.y == y)
    }

    /// 检查位置是否可通行
    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        self.get_tile(x, y).is_some_and(|t| t.is_passable())
    }

    pub fn is_door(&self, x: i32, y: i32) -> bool {
        self.get_tile(x, y).is_some_and(|t| t.is_door())
    }

    pub fn has_trap(&self, x: i32, y: i32) -> bool {
        self.get_tile(x, y).is_some_and(|t| t.has_trap())
    }

    pub fn get_trap(&self, x: i32, y: i32) -> Option<Trap> {
        self.get_tile(x, y).and_then(|t| t.get_trap().cloned())
    }

    /// 获取指定位置的敌人(可变引用)
    pub fn enemy_at_mut(&mut self, x: i32, y: i32) -> Option<&mut Enemy> {
        self.enemies.iter_mut().find(|e| e.x == x && e.y == y)
    }

    /// 获取指定位置的敌人
    pub fn enemy_at(&self, x: i32, y: i32) -> Option<&Enemy> {
        self.enemies.iter().find(|e| e.x == x && e.y == y)
    }

    /// 获取指定位置的物品名称
    pub fn get_item_name(&self, x: i32, y: i32) -> Option<&Item> {
        self.items.iter().find(|i| i.x == x && i.y == y)
    }

    /// 从位置拾取物品(移除并返回)
    pub fn take_item(&mut self, x: i32, y: i32) -> Option<Item> {
        if let Some(pos) = self
            .items
            .iter()
            .position(|item| item.x == x && item.y == y)
        {
            let item = self.items.remove(pos);
            if let Some(tile) = self.get_tile_mut(x, y) {
                tile.info.has_item = false;
            }
            Some(item)
        } else {
            None
        }
    }

    /// 获取指定位置的物品(不移除)
    pub fn get_item(&self, x: i32, y: i32) -> Option<&Item> {
        self.items.iter().find(|item| item.x == x && item.y == y)
    }

    /// 更新可见区域(基于玩家位置和视野半径)
    pub fn update_visibility(&mut self, x: i32, y: i32, radius: u8) {
        // 首先重置所有瓦片的可见性
        for tile in &mut self.tiles {
            tile.reset_visibility();
        }

        self.visible_tiles.clear();

        // 圆形视野
        for dy in -(radius as i32)..=radius as i32 {
            for dx in -(radius as i32)..=radius as i32 {
                let nx = x + dx;
                let ny = y + dy;

                // 检查是否在视野半径内且在边界内
                if dx * dx + dy * dy <= (radius as i32).pow(2)
                    && nx >= 0
                    && ny >= 0
                    && nx < self.width
                    && ny < self.height
                {
                    // 检查视线是否被阻挡
                    if self.has_line_of_sight(x, y, nx, ny) {
                        self.visible_tiles.insert((nx, ny));
                        self.explored_tiles.insert((nx, ny));

                        if let Some(tile) = self.get_tile_mut(nx, ny) {
                            tile.set_visible(true);
                        }
                    }
                }
            }
        }
    }

    /// 检查两点之间是否有视线(无阻挡)
    fn has_line_of_sight(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
        // Bresenham算法实现
        let mut x = x1;
        let mut y = y1;
        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            // 检查当前点是否阻挡视线
            if let Some(tile) = self.get_tile(x, y) {
                if tile.blocks_sight() && (x != x1 || y != y1) && (x != x2 || y != y2) {
                    return false;
                }
            }

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }

        true
    }

    /// 检查位置是否有敌人
    pub fn has_monster(&self, x: i32, y: i32) -> bool {
        self.enemies.iter().any(|e| e.x == x && e.y == y)
    }

    /// 检查位置是否是楼梯
    pub fn is_stair(&self, x: i32, y: i32) -> bool {
        self.get_tile(x, y).is_some_and(|t| {
            matches!(t.info.terrain_type, TerrainType::Stair(_))
        })
    }

    /// 检查位置是否已被探索
    pub fn is_explored(&self, x: i32, y: i32) -> bool {
        self.get_tile(x, y).is_some_and(|t| t.info.explored)
    }

    /// 检查位置是否可见
    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        self.visible_tiles.contains(&(x, y))
    }

    /// 尝试开门
    pub fn try_open_door(&mut self, x: i32, y: i32) -> bool {
        if let Some(tile) = self.get_tile_mut(x, y) {
            tile.try_open_door()
        } else {
            false
        }
    }

    /// 触发陷阱
    pub fn trigger_trap(&mut self, x: i32, y: i32) -> Option<TrapEffect> {
        if let Some(tile) = self.get_tile_mut(x, y) {
            tile.trigger_trap()
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Room {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// 获取房间中心点
    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    /// 获取房间内的随机点
    pub fn random_point(&self, rng: &mut impl Rng) -> (i32, i32) {
        let x = rng.random_range(self.x + 1..self.x + self.width - 1);
        let y = rng.random_range(self.y + 1..self.y + self.height - 1);
        (x, y)
    }

    /// 获取房间边缘的随机点
    pub fn random_point_on_edge(&self, rng: &mut impl Rng) -> (i32, i32) {
        match rng.random_range(0..4) {
            // 上边缘
            0 => (rng.random_range(self.x..self.x + self.width), self.y),
            // 右边缘
            1 => (
                self.x + self.width - 1,
                rng.random_range(self.y..self.y + self.height),
            ),
            // 下边缘
            2 => (
                rng.random_range(self.x..self.x + self.width),
                self.y + self.height - 1,
            ),
            // 左边缘
            _ => (self.x, rng.random_range(self.y..self.y + self.height)),
        }
    }

    /// 检查房间是否与另一个房间相交
    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Corridor {
    pub start: (i32, i32),
    pub end: (i32, i32),
    pub tiles: Vec<(i32, i32)>,
}

impl Corridor {
    pub fn new(start: (i32, i32), end: (i32, i32)) -> Self {
        let tiles = Self::create_corridor_tiles(start, end);
        Self { start, end, tiles }
    }

    /// 创建连接两个点的走廊瓦片
    fn create_corridor_tiles(start: (i32, i32), end: (i32, i32)) -> Vec<(i32, i32)> {
        let mut tiles = Vec::new();
        let (mut x, mut y) = start;
        let (end_x, end_y) = end;

        // 简单的曼哈顿走廊
        while x != end_x {
            x += if end_x > x { 1 } else { -1 };
            tiles.push((x, y));
        }

        while y != end_y {
            y += if end_y > y { 1 } else { -1 };
            tiles.push((x, y));
        }

        tiles
    }
}
