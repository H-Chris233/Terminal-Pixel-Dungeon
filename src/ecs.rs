//! ECS (Entity Component System) implementation for the game.

use hecs::{Entity, World};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Main ECS world container
pub struct ECSWorld {
    pub world: World,
    pub resources: Resources,
}

impl ECSWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            resources: Resources::default(),
        }
    }
    
    pub fn clear(&mut self) {
        self.world.clear();
        self.resources = Resources::default();
    }
}

/// Global resources that are shared across systems
#[derive(Default)]
pub struct Resources {
    /// Game time tracking
    pub clock: GameClock,
    
    /// Current game state
    pub game_state: GameState,
    
    /// Player input buffer
    pub input_buffer: InputBuffer,
    
    /// Game configuration
    pub config: GameConfig,
    
    /// RNG state
    pub rng: u64,
    
    /// Dungeon state (may be moved to components later)
    pub dungeon: Option<hecs::Entity>,
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

#[derive(Default, Clone, Copy, PartialEq)]
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


#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub item: Option<Item>,
    pub quantity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
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