//! ECS (Entity Component System) implementation for the game.

use hecs::{Entity, World};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use save::SaveData;
use error::GameError;
use hero::{Hero, Bag};
use items as game_items;
use dungeon::Dungeon;

/// Main ECS world container
pub struct ECSWorld {
    pub world: World,
    pub resources: Resources,
}

impl ECSWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            resources: Resources {
                clock: GameClock::default(),
                game_state: GameState::default(),
                input_buffer: InputBuffer::default(),
                config: GameConfig::new(),
                rng: 12345, // default seed
                dungeon: None,
                dungeon_instance: None,
            },
        }
    }
    
    pub fn clear(&mut self) {
        self.world.clear();
        self.resources = Resources {
            clock: GameClock::default(),
            game_state: GameState::default(),
            input_buffer: InputBuffer::default(),
            config: GameConfig::new(),
            rng: 12345, // default seed
            dungeon: None,
            dungeon_instance: None,
        };
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
    
    /// Actual dungeon instance
    pub dungeon_instance: Option<dungeon::Dungeon>,
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
        let dungeon = self.resources.dungeon_instance.clone()
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
            game_seed: self.resources.rng,
        };
        
        Ok(save_data)
    }
    
    /// Load data from save into ECS world
    pub fn from_save_data(&mut self, save_data: SaveData) -> Result<(), GameError> {
        // Clear current world
        self.clear();
        
        // Set up resources from save data
        self.resources.rng = save_data.game_seed;
        self.resources.game_state.depth = save_data.metadata.dungeon_depth;
        self.resources.dungeon_instance = Some(save_data.dungeon);
        
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

impl From<&Inventory> for Bag {
    fn from(inventory: &Inventory) -> Self {
        let mut bag = Bag::new();
        
        for item_slot in &inventory.items {
            if let Some(item) = &item_slot.item {
                // Convert the ECS Item to Hero module Item
                let hero_item = game_items::Item::new(match &item.item_type {
                    ItemType::Weapon { damage } => game_items::ItemKind::Weapon(
                        game_items::Weapon::new(1, game_items::weapon::WeaponKind::Dagger) // Using tier=1 and existing weapon kind
                    ),
                    ItemType::Armor { defense } => game_items::ItemKind::Armor(
                        game_items::Armor::new(1) // Using tier=1 only (no second parameter)
                    ),
                    ItemType::Consumable { effect } => {
                        // Map to appropriate consumable type based on effect
                        match effect {
                            ConsumableEffect::Healing { amount } => {
                                game_items::ItemKind::Potion(
                                    game_items::Potion::new_alchemy(game_items::potion::PotionKind::Healing)
                                )
                            }
                            _ => game_items::ItemKind::Potion(
                                game_items::Potion::new_alchemy(game_items::potion::PotionKind::Healing)
                            ),
                        }
                    }
                    ItemType::Key => game_items::ItemKind::Misc(
                        game_items::MiscItem::new(game_items::misc::MiscKind::Torch) // Using existing kind as fallback
                    ),
                    ItemType::Quest => game_items::ItemKind::Misc(
                        game_items::MiscItem::new(game_items::misc::MiscKind::Gold(10)) // Using existing kind with value
                    ),
                });
                
                // Add the item to the bag
                let _ = bag.add_item(hero_item);
            }
        }
        
        bag
    }
}

impl From<&Bag> for Inventory {
    fn from(bag: &Bag) -> Self {
        // Since Bag has complex internal structure, we'll just return an empty inventory for now
        // A full implementation would need to iterate through all the various inventory types
        // weapons, armors, potions, etc. and convert each item
        
        Self {
            items: Vec::new(), // Using empty vector for now
            max_slots: 20, // Using a default size
        }
    }
}