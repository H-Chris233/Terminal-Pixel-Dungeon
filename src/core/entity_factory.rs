use crate::ecs::{Position, Renderable, Actor, Stats, Energy, Viewshed, AI, Inventory, Faction, TerrainType as EcsTerrainType};
use dungeon::level::tiles::TerrainType as DungeonTerrainType;
use hero::class::Class;
use hecs::{Entity, World};
use items::Item;

/// 实体工厂，用于创建各种游戏实体
pub struct EntityFactory;

impl EntityFactory {
    pub fn new() -> Self {
        Self
    }

    /// 创建玩家实体
    pub fn create_player(&self, world: &mut World, x: i32, y: i32, class: Class) -> Entity {
        world.spawn((
            Position { x, y, z: 0 },
            Renderable {
                symbol: '@',
                fg_color: crate::ecs::Color::White,
                bg_color: Some(crate::ecs::Color::Black),
                order: 10,
            },
            Actor {
                name: "Player".to_string(),
                faction: Faction::Player,
            },
            Stats {
                hp: 100,
                max_hp: 100,
                attack: 10,
                defense: 5,
                accuracy: 10,
                evasion: 10,
                level: 1,
                experience: 0,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 10,
            },
            Viewshed {
                range: 8,
                visible_tiles: vec![],
                memory: vec![],
                dirty: true,
            },
            Inventory {
                items: vec![],
                max_slots: 20, // 玩家背包容量为20
            },
        ))
    }

    /// 创建怪物实体
    pub fn create_monster(&self, world: &mut World, x: i32, y: i32, monster_type: &str) -> Entity {
        let (symbol, name, hp, attack, defense) = match monster_type {
            "goblin" => ('g', "Goblin", 30, 8, 2),
            "orc" => ('o', "Orc", 50, 12, 4),
            "rat" => ('r', "Rat", 15, 5, 1),
            _ => ('m', "Monster", 25, 7, 2), // 默认怪物
        };

        world.spawn((
            Position { x, y, z: 0 },
            Renderable {
                symbol,
                fg_color: crate::ecs::Color::Gray,
                bg_color: Some(crate::ecs::Color::Black),
                order: 5,
            },
            Actor {
                name: name.to_string(),
                faction: Faction::Enemy,
            },
            Stats {
                hp,
                max_hp: hp,
                attack,
                defense,
                accuracy: 10,
                evasion: 10,
                level: 1,
                experience: 10,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 10,
            },
            Viewshed {
                range: 5,
                visible_tiles: vec![],
                memory: vec![],
                dirty: true,
            },
            AI {
                ai_type: crate::ecs::AIType::Aggressive,
                target: None,
                state: crate::ecs::AIState::Idle,
            },
        ))
    }

    /// 创建物品实体
    pub fn create_item(&self, world: &mut World, x: i32, y: i32, item: Item) -> Entity {
        let symbol = '!';
        let name = item.name();
        world.spawn((
            Position { x, y, z: 0 },
            Renderable {
                symbol,
                fg_color: crate::ecs::Color::Yellow,
                bg_color: Some(crate::ecs::Color::Black),
                order: 1,
            },
            item,
        ))
    }

    /// 创建地形实体
    pub fn create_terrain(&self, world: &mut World, x: i32, y: i32, terrain_type: DungeonTerrainType) -> Entity {
        let (symbol, is_passable, blocks_sight) = match terrain_type {
            DungeonTerrainType::Floor => ('.', true, false),
            DungeonTerrainType::Wall => ('#', false, true),
            DungeonTerrainType::Door(_) => ('+', true, true), // 门是可通行的但阻挡视线
            DungeonTerrainType::Stair(dungeon::level::tiles::StairDirection::Down) => ('>', true, false),
            DungeonTerrainType::Stair(dungeon::level::tiles::StairDirection::Up) => ('<', true, false),
            _ => ('.', true, false), // 默认为地板
        };

        world.spawn((
            Position { x, y, z: 0 },
            Renderable {
                symbol,
                fg_color: match terrain_type {
                    DungeonTerrainType::Wall => crate::ecs::Color::Gray,
                    _ => crate::ecs::Color::White,
                },
                bg_color: Some(crate::ecs::Color::Black),
                order: 0,
            },
            crate::ecs::Tile {
                terrain_type: match terrain_type {
                    DungeonTerrainType::Floor => EcsTerrainType::Floor,
                    DungeonTerrainType::Wall => EcsTerrainType::Wall,
                    DungeonTerrainType::Door(_) => EcsTerrainType::Door,
                    DungeonTerrainType::Stair(dungeon::level::tiles::StairDirection::Down) => EcsTerrainType::StairsDown,
                    DungeonTerrainType::Stair(dungeon::level::tiles::StairDirection::Up) => EcsTerrainType::StairsUp,
                    _ => EcsTerrainType::Floor, // 默认
                },
                is_passable,
                blocks_sight,
                has_items: false,
                has_monster: false,
            },
        ))
    }
}