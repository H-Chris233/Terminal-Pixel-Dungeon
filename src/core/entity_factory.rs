use crate::ecs::{
    AI, Actor, Energy, Faction, Inventory, Position, Renderable, Stats,
    TerrainType as EcsTerrainType, Viewshed,
};
use dungeon::level::tiles::TerrainType as DungeonTerrainType;
use hecs::{Entity, World};
use hero::class::Class;
use items::Item;

/// 实体工厂，用于创建各种游戏实体
pub struct EntityFactory;

impl EntityFactory {
    pub fn new() -> Self {
        Self
    }

    /// 创建玩家实体
    pub fn create_player(&self, world: &mut World, x: i32, y: i32, class: Class) -> Entity {
        // 使用职业特定的基础属性
        let base_hp = class.base_hp();
        let attack_mod = class.attack_mod();
        let defense_mod = class.defense_mod();
        
        // 基础攻击和防御值
        let base_attack = 10;
        let base_defense = 5;
        
        // 应用职业修正
        let attack = (base_attack as f32 * attack_mod) as u32;
        let defense = (base_defense as f32 * defense_mod) as u32;
        
        // 创建包含初始装备的物品栏
        let starting_kit = class.starting_kit();
        let mut inventory_items = Vec::new();
        
        for item in starting_kit {
            if let Ok(ecs_item) = crate::ecs::ECSItem::from_items_item(&item) {
                inventory_items.push(crate::ecs::ItemSlot {
                    item: Some(ecs_item),
                    quantity: item.quantity,
                });
            }
        }
        
        world.spawn((
            Position { x, y, z: 0 },
            Renderable {
                symbol: '@',
                fg_color: crate::ecs::Color::White,
                bg_color: Some(crate::ecs::Color::Black),
                order: 10,
            },
            Actor {
                name: format!("{} ({})", "Player", class),
                faction: Faction::Player,
            },
            Stats {
                hp: base_hp,
                max_hp: base_hp,
                attack,
                defense,
                accuracy: 80,
                evasion: 20,
                level: 1,
                experience: 0,
                class: Some(class.clone()),
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
                algorithm: crate::ecs::FovAlgorithm::default(),
            },
            Inventory {
                items: inventory_items,
                max_slots: 20, // 玩家背包容量为20
            },
            crate::ecs::Hunger::new(5), // 初始饱食度为5（半饱）
            crate::ecs::Wealth::new(0), // 初始金币为0
            crate::ecs::PlayerProgress::new(10, class.clone(), hero::class::SkillState::default()), // 使用选中的职业
            crate::ecs::Player, // Player marker component
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
                class: None,
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
                algorithm: crate::ecs::FovAlgorithm::default(),
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
    pub fn create_terrain(
        &self,
        world: &mut World,
        x: i32,
        y: i32,
        terrain_type: DungeonTerrainType,
    ) -> Entity {
        let (symbol, is_passable, blocks_sight) = match terrain_type {
            DungeonTerrainType::Floor => ('.', true, false),
            DungeonTerrainType::Wall => ('#', false, true),
            DungeonTerrainType::Door(_) => ('+', true, true), // 门是可通行的但阻挡视线
            DungeonTerrainType::Stair(dungeon::level::tiles::StairDirection::Down) => {
                ('>', true, false)
            }
            DungeonTerrainType::Stair(dungeon::level::tiles::StairDirection::Up) => {
                ('<', true, false)
            }
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
                    DungeonTerrainType::Stair(dungeon::level::tiles::StairDirection::Down) => {
                        EcsTerrainType::StairsDown
                    }
                    DungeonTerrainType::Stair(dungeon::level::tiles::StairDirection::Up) => {
                        EcsTerrainType::StairsUp
                    }
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
