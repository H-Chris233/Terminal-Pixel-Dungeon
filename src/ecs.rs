//! ECS (Entity Component System) implementation for the game.

use hecs::{Entity, World};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use rand::rngs::StdRng;
use rand::SeedableRng;

use save::SaveData;
use error::GameError;
use hero::{Hero, Bag};
use items as game_items;
use crate::event_bus::{EventBus, GameEvent, LogLevel, EventHandler, Priority};
use std::sync::{Arc, Mutex};

// 说明：在完全解耦的系统中，这些模块间的通信应该通过事件总线完成
// 例如，保存系统通过监听 GameSaved 事件来保存游戏状态
// 而不是直接依赖其他模块的结构体

/// Main ECS world container
pub struct ECSWorld {
    pub world: World,
    pub resources: Resources,
    pub event_bus: EventBus,
}

impl ECSWorld {
    pub fn new() -> Self {
        let mut ecs_world = Self {
            world: World::new(),
            resources: Resources::default(),
            event_bus: EventBus::new(),
        };

        // 注册默认的事件处理器
        ecs_world.register_default_handlers();

        ecs_world
    }

    /// 注册默认的事件处理器
    fn register_default_handlers(&mut self) {
        // 暂时不注册默认处理器
        // 事件处理将在 process_events 中直接完成
        // 外部模块可以根据需要注册自己的处理器
    }

    pub fn generate_and_set_dungeon(&mut self, max_depth: usize, seed: u64) -> anyhow::Result<()> {
        let dungeon = dungeon::Dungeon::generate(max_depth, seed)?;
        set_dungeon_instance(&mut self.world, dungeon);
        // Re-seed the RNG for consistent randomness across the game
        self.resources.rng = StdRng::seed_from_u64(seed);
        self.resources.game_state.depth = 1;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.world.clear();
        self.resources = Resources::default();
        self.event_bus.clear();
    }

    /// 发布事件到事件总线
    pub fn publish_event(&mut self, event: GameEvent) {
        self.event_bus.publish(event);
    }

    /// 发布延迟事件（下一帧处理）
    pub fn publish_delayed_event(&mut self, event: GameEvent) {
        self.event_bus.publish_delayed(event);
    }

    /// 处理所有待处理的事件
    /// 这个方法在 ECSWorld 级别处理核心游戏状态更新
    /// 外部处理器（通过 subscribe）用于日志、UI 等非核心功能
    pub fn process_events(&mut self) {
        // 事件已通过订阅者模式处理（日志、统计等）
        // 这里处理核心游戏状态的更新
        let events: Vec<GameEvent> = self.event_bus.drain().collect();

        for event in events {
            self.handle_core_event(&event);
        }
    }

    /// 处理核心游戏状态事件（更新 Resources）
    fn handle_core_event(&mut self, event: &GameEvent) {
        match event {
            GameEvent::DamageDealt { damage, is_critical, .. } => {
                let msg = if *is_critical {
                    format!("暴击！造成 {} 点伤害", damage)
                } else {
                    format!("造成 {} 点伤害", damage)
                };
                self.resources.game_state.message_log.push(msg);
            }

            GameEvent::EntityDied { entity_name, .. } => {
                self.resources.game_state.message_log.push(
                    format!("{} 已死亡", entity_name)
                );
            }

            GameEvent::ItemPickedUp { item_name, .. } => {
                self.resources.game_state.message_log.push(
                    format!("拾取了 {}", item_name)
                );
            }

            GameEvent::ItemUsed { item_name, effect, .. } => {
                self.resources.game_state.message_log.push(
                    format!("使用了 {}，{}", item_name, effect)
                );
            }

            GameEvent::LevelChanged { old_level, new_level } => {
                self.resources.game_state.depth = *new_level;
                self.resources.game_state.message_log.push(
                    format!("从第 {} 层进入第 {} 层", old_level, new_level)
                );
            }

            GameEvent::GameOver { reason } => {
                self.resources.game_state.game_state = GameStatus::GameOver;
                self.resources.game_state.message_log.push(
                    format!("游戏结束：{}", reason)
                );
            }

            GameEvent::Victory => {
                self.resources.game_state.game_state = GameStatus::Victory;
                self.resources.game_state.message_log.push(
                    "恭喜！你获得了胜利！".to_string()
                );
            }

            GameEvent::LogMessage { message, level } => {
                let prefix = match level {
                    LogLevel::Debug => "[调试] ",
                    LogLevel::Info => "",
                    LogLevel::Warning => "[警告] ",
                    LogLevel::Error => "[错误] ",
                };
                self.resources.game_state.message_log.push(
                    format!("{}{}", prefix, message)
                );
            }

            GameEvent::TrapTriggered { trap_type, .. } => {
                self.resources.game_state.message_log.push(
                    format!("触发了{}陷阱！", trap_type)
                );
            }

            GameEvent::StatusApplied { status, duration, .. } => {
                self.resources.game_state.message_log.push(
                    format!("受到{}效果影响，持续{}回合", status, duration)
                );
            }

            GameEvent::StatusRemoved { status, .. } => {
                self.resources.game_state.message_log.push(
                    format!("{}效果已消失", status)
                );
            }

            _ => {}
        }
    }

    /// 帧结束时调用，准备处理下一帧事件
    pub fn next_frame(&mut self) {
        self.event_bus.next_frame();

        // 同步消息日志到 resources
        self.sync_message_log();
    }

    /// 同步事件处理器的消息日志到 Resources
    fn sync_message_log(&mut self) {
        // 这里可以从事件处理器获取日志并同步到 Resources
        // 目前保持简单实现
    }
}

// ========== 事件处理器实现 ==========

/// 游戏状态事件处理器
/// 负责处理游戏状态相关的事件，如伤害、死亡、物品使用等
pub struct GameStateHandler {
    message_log: Arc<Mutex<Vec<String>>>,
}

impl EventHandler for GameStateHandler {
    fn handle(&mut self, event: &GameEvent) {
        let message = match event {
            GameEvent::DamageDealt { damage, is_critical, .. } => {
                Some(if *is_critical {
                    format!("暴击！造成 {} 点伤害", damage)
                } else {
                    format!("造成 {} 点伤害", damage)
                })
            }

            GameEvent::EntityDied { entity_name, .. } => {
                Some(format!("{} 已死亡", entity_name))
            }

            GameEvent::ItemPickedUp { item_name, .. } => {
                Some(format!("拾取了 {}", item_name))
            }

            GameEvent::ItemUsed { item_name, effect, .. } => {
                Some(format!("使用了 {}，{}", item_name, effect))
            }

            GameEvent::LevelChanged { old_level, new_level } => {
                Some(format!("从第 {} 层进入第 {} 层", old_level, new_level))
            }

            GameEvent::GameOver { reason } => {
                Some(format!("游戏结束：{}", reason))
            }

            GameEvent::Victory => {
                Some("恭喜！你获得了胜利！".to_string())
            }

            GameEvent::LogMessage { message, level } => {
                let prefix = match level {
                    LogLevel::Debug => "[调试] ",
                    LogLevel::Info => "",
                    LogLevel::Warning => "[警告] ",
                    LogLevel::Error => "[错误] ",
                };
                Some(format!("{}{}", prefix, message))
            }

            GameEvent::TrapTriggered { trap_type, .. } => {
                Some(format!("触发了{}陷阱！", trap_type))
            }

            GameEvent::StatusApplied { status, duration, .. } => {
                Some(format!("受到{}效果影响，持续{}回合", status, duration))
            }

            GameEvent::StatusRemoved { status, .. } => {
                Some(format!("{}效果已消失", status))
            }

            _ => None,
        };

        if let Some(msg) = message {
            if let Ok(mut log) = self.message_log.lock() {
                log.push(msg);
            }
        }
    }

    fn name(&self) -> &str {
        "GameStateHandler"
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

/// Global resources that are shared across systems
pub struct Resources {
    /// Game time tracking
    pub clock: GameClock,

    /// Current game state
    pub game_state: GameState,

    /// Player input buffer
    pub input_buffer: InputBuffer,

    /// Game configuration
    pub config: GameConfig,

    /// Random number generator state
    pub rng: StdRng,

    /// Dungeon state marker entity (actual dungeon stored as a component)
    pub dungeon: Option<hecs::Entity>,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            clock: GameClock::default(),
            game_state: GameState::default(),
            input_buffer: InputBuffer::default(),
            config: GameConfig::new(),
            rng: StdRng::seed_from_u64(12345), // default seed
            dungeon: None,
        }
    }
}

impl Resources {
    /// Create a new Resources with a specific seed
    pub fn with_seed(seed: u64) -> Self {
        Self {
            clock: GameClock::default(),
            game_state: GameState::default(),
            input_buffer: InputBuffer::default(),
            config: GameConfig::new(),
            rng: StdRng::seed_from_u64(seed),
            dungeon: None,
        }
    }

    /// Re-seed the RNG (useful for save/load)
    pub fn reseed_rng(&mut self, seed: u64) {
        self.rng = StdRng::seed_from_u64(seed);
    }
}

pub struct GameClock {
    pub current_time: std::time::SystemTime,
    pub elapsed_time: Duration,
    pub turn_count: u32,
    pub tick_rate: Duration,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            current_time: std::time::SystemTime::now(),
            elapsed_time: Duration::from_secs(0),
            turn_count: 0,
            tick_rate: Duration::from_millis(16), // ~60 FPS
        }
    }
}

#[derive(Default)]
pub struct GameState {
    pub game_state: GameStatus,
    pub depth: usize,
    pub message_log: Vec<String>,
    pub terminal_width: u16,
    pub terminal_height: u16,
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub enum GameStatus {
    #[default]
    Running,
    Paused,
    GameOver,
    Victory,
}

#[derive(Default)]
pub struct InputBuffer {
    pub pending_actions: Vec<PlayerAction>,
    /// Actions that were successfully completed this frame and need energy deduction
    pub completed_actions: Vec<PlayerAction>,
}

#[derive(Clone)]
pub enum PlayerAction {
    Move(Direction),
    Attack(Position),
    UseItem(usize),
    DropItem(usize),
    Descend,
    Ascend,
    Wait,
    Quit,
}

#[derive(Clone, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

#[derive(Default)]
pub struct GameConfig {
    pub fov_range: u8,
    pub max_depth: usize,
    pub save_directory: String,
}

impl GameConfig {
    pub fn new() -> Self {
        Self {
            fov_range: 8,
            max_depth: 10,
            save_directory: "saves".to_string(),
        }
    }
}

// Player marker component
#[derive(Clone, Debug)]
pub struct Player;

// Basic Components
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32, // dungeon level
}



impl Position {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub terrain_type: TerrainType,
    pub is_passable: bool,
    pub blocks_sight: bool,
    pub has_items: bool,
    pub has_monster: bool,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TerrainType {
    Floor,
    Wall,
    Door,
    StairsDown,
    StairsUp,
    Water,
    Trap,
    Barrel,
    Empty,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Renderable {
    pub symbol: char,
    pub fg_color: Color,
    pub bg_color: Option<Color>,
    pub order: u8, // rendering order
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    White,
    Black,
    Reset,
    Rgb(u8, u8, u8),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actor {
    pub name: String,
    pub faction: Faction,
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Faction {
    Player,
    Enemy,
    Neutral,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub hp: u32,
    pub max_hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub accuracy: u32,
    pub evasion: u32,
    pub level: u32,
    pub experience: u32,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<ItemSlot>,
    pub max_slots: usize,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemSlot {
    pub item: Option<ECSItem>,
    pub quantity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ECSItem {
    pub name: String,
    pub item_type: ItemType,
    pub value: u32,
    pub identified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ItemType {
    Weapon { damage: u32 },
    Armor { defense: u32 },
    Consumable { effect: ConsumableEffect },
    Key,
    Quest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConsumableEffect {
    Healing { amount: u32 },
    Damage { amount: u32 },
    Buff { stat: StatType, value: i32, duration: u32 },
    Teleport,
    Identify,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatType {
    Hp,
    Attack,
    Defense,
    Accuracy,
    Evasion,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Viewshed {
    pub range: u8,
    pub visible_tiles: Vec<Position>,
    pub memory: Vec<Position>, // previously seen tiles
    pub dirty: bool,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Energy {
    pub current: u32,
    pub max: u32,
    pub regeneration_rate: u32,
}


#[derive(Clone, Debug)]
pub struct AI {
    pub ai_type: AIType,
    pub target: Option<Entity>,
    pub state: AIState,
}

impl AI {
    pub fn range(&self) -> u8 {
        match &self.ai_type {
            AIType::Aggressive => 10, // Default aggressive range
            AIType::Passive => 2,
            AIType::Neutral => 5,
            AIType::Patrol { .. } => 10,
        }
    }
}

// AI cannot be serialized due to Entity type


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AIType {
    Passive,
    Aggressive,
    Neutral,
    Patrol { path: Vec<Position> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AIState {
    Idle,
    Patrolling,
    Chasing,
    Fleeing,
    Attacking,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Effects {
    pub active_effects: Vec<ActiveEffect>,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub effect_type: EffectType,
    pub duration: u32,
    pub intensity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EffectType {
    Poison,
    Burning,
    Paralysis,
    Rooted,
    Confusion,
    Invisibility,
    Levitation,
    Healing,
}

// Functions to convert between ECS components and hero module structures
impl From<&Stats> for Hero {
    fn from(stats: &Stats) -> Self {
        let mut hero = Hero::with_seed(hero::class::Class::Warrior, 12345);
        hero.hp = stats.hp;
        hero.max_hp = stats.max_hp;
        hero.base_attack = stats.attack;
        hero.base_defense = stats.defense;
        hero.level = stats.level;
        hero.experience = stats.experience;
        hero
    }
}

impl From<&Hero> for Stats {
    fn from(hero: &Hero) -> Self {
        Self {
            hp: hero.hp,
            max_hp: hero.max_hp,
            attack: hero.base_attack,
            defense: hero.base_defense,
            accuracy: 80, // Default accuracy
            evasion: 20,  // Default evasion
            level: hero.level,
            experience: hero.experience,
        }
    }
}

impl ECSWorld {
    /// Convert ECS world to save data
    pub fn to_save_data(&self) -> Result<SaveData, GameError> {
        // Extract hero data from ECS
        let mut hero: Option<Hero> = None;
        
        // Find the player entity and convert to hero
        for (entity, (_player_marker, stats, inventory)) in self.world.query::<(&Player, &Stats, &Inventory)>().iter() {
            // Convert ECS components to Hero
            let mut new_hero = Hero::from(stats);
            new_hero.bag = Bag::from(inventory);
            
            // Update hero's position based on ECS Position
            if let Ok(pos) = self.world.get::<&Position>(entity) {
                new_hero.x = pos.x;
                new_hero.y = pos.y;
            }
            
            hero = Some(new_hero);
            break;
        }
        
        // Extract dungeon data
        let dungeon = get_dungeon_clone(&self.world)
            .ok_or_else(|| GameError::InvalidLevelData)?;
        
        // Create save data
        let save_data = SaveData {
            metadata: save::SaveMetadata {
                timestamp: std::time::SystemTime::now(),
                dungeon_depth: self.resources.game_state.depth,
                hero_name: hero.as_ref().map_or("Unknown".to_string(), |h| h.name.clone()),
                hero_class: hero.as_ref().map_or("Unknown".to_string(), |h| format!("{:?}", h.class)),
                play_time: self.resources.clock.elapsed_time.as_secs_f64(),
            },
            hero: hero.ok_or_else(|| GameError::InvalidHeroData)?,
            dungeon,
            game_seed: 0, // 需要保存实际的种子值
        };

        Ok(save_data)
    }
    
    /// Load data from save into ECS world
    pub fn from_save_data(&mut self, save_data: SaveData) -> Result<(), GameError> {
        // Clear current world
        self.clear();

        // Set up resources from save data
        self.resources.rng = StdRng::seed_from_u64(save_data.game_seed);
        self.resources.game_state.depth = save_data.metadata.dungeon_depth;
        set_dungeon_instance(&mut self.world, save_data.dungeon);

        // Convert hero to ECS components and spawn player entity
        let hero = save_data.hero;
        let stats: Stats = (&hero).into();
        let inventory: Inventory = (&hero.bag).into();

        // Spawn player entity with converted components
        self.world.spawn((
            Position::new(hero.x, hero.y, save_data.metadata.dungeon_depth as i32),
            Actor {
                name: hero.name.clone(),
                faction: Faction::Player,
            },
            Renderable {
                symbol: '@',
                fg_color: Color::Yellow,
                bg_color: Some(Color::Black),
                order: 10,
            },
            stats,
            inventory,
            Viewshed {
                range: 8,
                visible_tiles: vec![],
                memory: vec![],
                dirty: true,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 1,
            },
            Player, // Player marker component
        ));

        Ok(())
    }
}

// Dungeon component and helper APIs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DungeonComponent(pub dungeon::Dungeon);

/// Get a cloned dungeon instance from the world if present
pub fn get_dungeon_clone(world: &World) -> Option<dungeon::Dungeon> {
    for (entity, dungeon_comp) in world.query::<&DungeonComponent>().iter() {
        return Some(dungeon_comp.0.clone());
    }
    None
}

/// Set or replace the dungeon instance in the world. If no dungeon entity exists, one is created.
pub fn set_dungeon_instance(world: &mut World, dungeon: dungeon::Dungeon) {
    // Collect entity ids into a temporary vector to avoid holding a QueryBorrow while mutating
    let existing_entities: Vec<_> = world.query::<&DungeonComponent>().iter().map(|(e, _)| e).collect();
    if let Some(&entity) = existing_entities.first() {
        let _ = world.remove_one::<DungeonComponent>(entity);
        let _ = world.insert_one(entity, DungeonComponent(dungeon));
        return;
    }

    // No existing dungeon component, spawn a new entity with it
    let _ = world.spawn((DungeonComponent(dungeon),));
}

impl From<&Inventory> for Bag {
    fn from(inventory: &Inventory) -> Self {
        let mut bag = Bag::new();

        fn map_item(item: &ECSItem) -> game_items::ItemKind {
            match &item.item_type {
                ItemType::Weapon { damage: _ } => {
                    game_items::ItemKind::Weapon(game_items::Weapon::new(1, game_items::weapon::WeaponKind::Dagger))
                }
                ItemType::Armor { defense: _ } => {
                    game_items::ItemKind::Armor(game_items::Armor::new(1))
                }
                ItemType::Consumable { effect: _ } => {
                    game_items::ItemKind::Potion(game_items::Potion::new_alchemy(game_items::potion::PotionKind::Healing))
                }
                ItemType::Key => {
                    game_items::ItemKind::Misc(game_items::MiscItem::new(game_items::misc::MiscKind::Torch))
                }
                ItemType::Quest => {
                    game_items::ItemKind::Misc(game_items::MiscItem::new(game_items::misc::MiscKind::Gold(10)))
                }
            }
        }

        for item_slot in &inventory.items {
            if let Some(item) = &item_slot.item {
                let kind = map_item(item);
                let hero_item = game_items::Item::new(kind);
                let _ = bag.add_item(hero_item);
            }
        }

        bag
    }
}

/// Convenience helper to get a mutable dungeon reference and run a closure on it
pub fn with_dungeon_mut<F>(world: &mut World, f: F)
where
    F: FnOnce(&mut dungeon::Dungeon),
{
    // Collect entity ids to avoid holding the query borrow while mutating
    let ids: Vec<_> = world.query::<&DungeonComponent>().iter().map(|(e, _)| e).collect();
    if let Some(&entity) = ids.first() {
        if let Ok(mut comp) = world.get::<&mut DungeonComponent>(entity) {
            f(&mut comp.0);
        }
    }
}


impl From<&Bag> for Inventory {
    fn from(bag: &Bag) -> Self {
        // Conservative fallback: create empty inventory to avoid depending on Bag internals
        Self {
            items: Vec::new(),
            max_slots: 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::GameEvent;

    #[test]
    fn test_event_bus_integration() {
        let mut world = ECSWorld::new();

        // 测试事件发布
        world.publish_event(GameEvent::LogMessage {
            message: "测试消息".to_string(),
            level: LogLevel::Info,
        });

        assert_eq!(world.event_bus.len(), 1);

        // 测试事件处理
        world.process_events();

        // 检查日志是否被添加
        assert_eq!(world.resources.game_state.message_log.len(), 1);
        assert_eq!(world.resources.game_state.message_log[0], "测试消息");

        // 事件应该被清空
        assert_eq!(world.event_bus.len(), 0);
    }

    #[test]
    fn test_combat_events() {
        let mut world = ECSWorld::new();

        // 创建玩家和敌人实体
        let player = world.world.spawn((
            Position::new(0, 0, 0),
            Actor {
                name: "Player".to_string(),
                faction: Faction::Player,
            },
            Stats {
                hp: 100,
                max_hp: 100,
                attack: 10,
                defense: 5,
                accuracy: 80,
                evasion: 20,
                level: 1,
                experience: 0,
            },
        ));

        let enemy = world.world.spawn((
            Position::new(1, 0, 0),
            Actor {
                name: "Goblin".to_string(),
                faction: Faction::Enemy,
            },
            Stats {
                hp: 30,
                max_hp: 30,
                attack: 5,
                defense: 2,
                accuracy: 60,
                evasion: 10,
                level: 1,
                experience: 0,
            },
        ));

        // 发布战斗开始事件
        world.publish_event(GameEvent::CombatStarted {
            attacker: player.id(),
            defender: enemy.id(),
        });

        // 发布伤害事件
        world.publish_event(GameEvent::DamageDealt {
            attacker: player.id(),
            victim: enemy.id(),
            damage: 10,
            is_critical: false,
        });

        // 处理事件
        world.process_events();

        // 检查日志
        assert!(world.resources.game_state.message_log.len() > 0);
        assert!(world.resources.game_state.message_log[0].contains("造成 10 点伤害"));
    }

    #[test]
    fn test_delayed_events() {
        let mut world = ECSWorld::new();

        // 发布延迟事件
        world.publish_delayed_event(GameEvent::LogMessage {
            message: "延迟消息".to_string(),
            level: LogLevel::Info,
        });

        // 当前帧应该没有事件
        assert_eq!(world.event_bus.len(), 0);

        // 移到下一帧
        world.next_frame();

        // 现在应该有事件了
        assert_eq!(world.event_bus.len(), 1);

        // 处理事件
        world.process_events();

        // 检查日志
        assert_eq!(world.resources.game_state.message_log.len(), 1);
        assert_eq!(world.resources.game_state.message_log[0], "延迟消息");
    }

    #[test]
    fn test_game_over_event() {
        let mut world = ECSWorld::new();

        // 初始状态应该是 Running
        assert_eq!(world.resources.game_state.game_state, GameStatus::Running);

        // 发布游戏结束事件
        world.publish_event(GameEvent::GameOver {
            reason: "测试失败".to_string(),
        });

        // 处理事件
        world.process_events();

        // 检查游戏状态
        assert_eq!(world.resources.game_state.game_state, GameStatus::GameOver);
        assert!(world.resources.game_state.message_log.iter().any(|msg| msg.contains("游戏结束")));
    }

    #[test]
    fn test_level_change_event() {
        let mut world = ECSWorld::new();

        // 初始深度为 0
        assert_eq!(world.resources.game_state.depth, 0);

        // 发布层级变化事件
        world.publish_event(GameEvent::LevelChanged {
            old_level: 1,
            new_level: 2,
        });

        // 处理事件
        world.process_events();

        // 检查深度是否更新
        assert_eq!(world.resources.game_state.depth, 2);
        assert!(world.resources.game_state.message_log.iter().any(|msg| msg.contains("从第 1 层进入第 2 层")));
    }
}