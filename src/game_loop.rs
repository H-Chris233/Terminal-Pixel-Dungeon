//! Game loop implementation that orchestrates ECS systems.

use crate::ecs::*;
use crate::renderer::*;
use crate::systems::*;
use crate::input::*;
use crate::turn_system::TurnSystem;
use save::{SaveSystem, AutoSave};
use anyhow;
use hecs::World;
use std::time::{Duration, Instant};
use crate::core::GameEngine;

/// Main game loop that runs the ECS systems in order
pub struct GameLoop<R: Renderer, I: InputSource, C: Clock> {
    pub game_engine: GameEngine,
    pub ecs_world: ECSWorld,
    pub renderer: R,
    pub input_source: I,
    pub clock: C,
    pub systems: Vec<Box<dyn System>>,
    pub turn_system: TurnSystem,
    pub is_running: bool,
    pub save_system: Option<AutoSave>,
}

impl<R: Renderer, I: InputSource<Event = crate::input::InputEvent>, C: Clock> GameLoop<R, I, C> {
    pub fn new(
        renderer: R,
        input_source: I,
        clock: C,
    ) -> Self {
        let systems: Vec<Box<dyn System>> = vec![
            Box::new(InputSystem),
            Box::new(TimeSystem),
            Box::new(MovementSystem),
            Box::new(AISystem),
            Box::new(CombatSystem),
            Box::new(FOVSystem),
            Box::new(EffectSystem),
            Box::new(EnergySystem),
            Box::new(InventorySystem),
            Box::new(DungeonSystem),
            Box::new(RenderingSystem),
        ];
        

        
        let mut ecs_world = ECSWorld::new();
        let game_engine = GameEngine::new();
        
        let save_system = match SaveSystem::new("saves", 10) {
            Ok(save_sys) => Some(AutoSave::new(save_sys, std::time::Duration::from_secs(300))),
            Err(e) => {
                eprintln!("Failed to initialize save system: {}", e);
                None
            }
        };

        Self {
            game_engine,
            ecs_world,
            renderer,
            input_source,
            clock,
            systems,
            turn_system: TurnSystem::new(),
            is_running: true,
            save_system,
        }
    }
    
    /// Initialize the game state
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        self.renderer.init()?;
        
        // Add initial entities to the world
        self.initialize_entities();
        
        Ok(())
    }
    
    /// Initialize starting entities
    fn initialize_entities(&mut self) {
        // Determine player start position from dungeon if available
        let (start_x, start_y, start_z) = if let Some(dungeon) = crate::ecs::get_dungeon_clone(&self.ecs_world.world) {
            let lvl = dungeon.current_level();
            (lvl.stair_up.0, lvl.stair_up.1, dungeon.depth as i32 - 1)
        } else {
            (10, 10, 0)
        };

        // Add player entity
        let player_entity = self.ecs_world.world.spawn((
            Position::new(start_x, start_y, start_z),
            Actor {
                name: "Player".to_string(),
                faction: Faction::Player,
            },
            Renderable {
                symbol: '@',
                fg_color: Color::Yellow,
                bg_color: Some(Color::Black),
                order: 10,
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
            Inventory {
                items: vec![],
                max_slots: 10,
            },
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
            crate::ecs::Player, // Player marker component
        ));
        
        // Add some test enemies
        self.ecs_world.world.spawn((
            Position::new(15, 10, 0),
            Actor {
                name: "Goblin".to_string(),
                faction: Faction::Enemy,
            },
            Renderable {
                symbol: 'g',
                fg_color: Color::Green,
                bg_color: Some(Color::Black),
                order: 5,
            },
            Stats {
                hp: 30,
                max_hp: 30,
                attack: 5,
                defense: 2,
                accuracy: 70,
                evasion: 10,
                level: 1,
                experience: 10,
            },
            AI {
                ai_type: AIType::Aggressive,
                target: Some(player_entity),
                state: AIState::Idle,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 1,
            },
        ));
        
        // Add some items
        self.ecs_world.world.spawn((
            Position::new(12, 12, 0),
            Renderable {
                symbol: '!',
                fg_color: Color::Red,
                bg_color: Some(Color::Black),
                order: 1,
            },
            ECSItem {
                name: "Health Potion".to_string(),
                item_type: ItemType::Consumable {
                    effect: ConsumableEffect::Healing { amount: 20 },
                },
                value: 10,
                identified: true,
            },
            Tile {
                terrain_type: TerrainType::Empty,
                is_passable: true,
                blocks_sight: false,
                has_items: true,
                has_monster: false,
            },
        ));
        
        // Add some basic dungeon tiles (simplified)
        for x in 5..25 {
            for y in 5..25 {
                self.ecs_world.world.spawn((
                    Position::new(x, y, 0),
                    Tile {
                        terrain_type: if x == 5 || x == 24 || y == 5 || y == 24 {
                            TerrainType::Wall
                        } else {
                            TerrainType::Floor
                        },
                        is_passable: x != 5 && x != 24 && y != 5 && y != 24,
                        blocks_sight: x == 5 || x == 24 || y == 5 || y == 24,
                        has_items: false,
                        has_monster: false,
                    },
                    Renderable {
                        symbol: if x == 5 || x == 24 || y == 5 || y == 24 { '#' } else { '.' },
                        fg_color: if x == 5 || x == 24 || y == 5 || y == 24 { Color::Gray } else { Color::White },
                        bg_color: Some(Color::Black),
                        order: 0,
                    },
                ));
            }
        }
    }
    
    /// Main game loop
    pub fn run(&mut self) -> anyhow::Result<()> {
        while self.is_running {
            // Check game state before processing
            match self.ecs_world.resources.game_state.game_state {
                crate::ecs::GameStatus::GameOver => {
                    self.is_running = false;
                    break;
                }
                crate::ecs::GameStatus::Victory => {
                    self.is_running = false;
                    break;
                }
                _ => {} // Continue normal game processing
            }
            
            // Handle input
            self.handle_input()?;
            
            // Update game state based on turns
            self.update_turn()?;
            
            // Check game state again after update
            match self.ecs_world.resources.game_state.game_state {
                crate::ecs::GameStatus::GameOver => {
                    self.is_running = false;
                    break;
                }
                crate::ecs::GameStatus::Victory => {
                    self.is_running = false;
                    break;
                }
                _ => {} // Continue normal game processing
            }
            
            // Render the game
            self.render()?;
            
            // Small delay to prevent busy looping
            self.clock.sleep(Duration::from_millis(1));
        }
        
        self.cleanup()?;
        Ok(())
    }
    
    /// Handle user input
    fn handle_input(&mut self) -> anyhow::Result<()> {
        // Poll for input with a small timeout
        if let Ok(Some(event)) = self.input_source.poll(Duration::from_millis(50)) {
            match event {
                InputEvent::Key(key_event) => {
                    // Convert key event to player action
                    if let Some(action) = key_event_to_player_action_from_internal(key_event) {
                        self.ecs_world.resources.input_buffer.pending_actions.push(action);
                    }
                },
                InputEvent::Resize(width, height) => {
                    // Handle terminal resize
                    self.renderer.resize(&mut self.ecs_world.resources, width, height)?;
                },
                _ => {} // Other events currently ignored
            }
        }
        
        Ok(())
    }
    
    /// Update game state by running all systems for a turn
    fn update_turn(&mut self) -> anyhow::Result<()> {
        // Run systems based on turn state
        if self.turn_system.is_player_turn() {
            // Run non-input systems
            for system in &mut self.systems {
                // Skip EnergySystem as we're managing energy through turn system now
                if system.is_energy_system() {
                    continue;
                }
                
                match system.run(&mut self.ecs_world.world, &mut self.ecs_world.resources) {
                    SystemResult::Continue => continue,
                    SystemResult::Stop => {
                        self.is_running = false;
                        return Ok(());
                    }
                    SystemResult::Error(msg) => {
                        eprintln!("System error: {}", msg);
                        return Err(anyhow::anyhow!(msg));
                    }
                }
            }
            
            // Process the player's turn
            self.turn_system.process_turn_cycle(&mut self.ecs_world.world, &mut self.ecs_world.resources)?;
        } else {
            // Process AI turns without player input
            // Run non-input systems
            for system in &mut self.systems {
                // Skip EnergySystem as we're managing energy through turn system now
                if system.is_energy_system() {
                    continue;
                }
                
                match system.run(&mut self.ecs_world.world, &mut self.ecs_world.resources) {
                    SystemResult::Continue => continue,
                    SystemResult::Stop => {
                        self.is_running = false;
                        return Ok(());
                    }
                    SystemResult::Error(msg) => {
                        eprintln!("System error: {}", msg);
                        return Err(anyhow::anyhow!(msg));
                    }
                }
            }
            
            // Process AI turns
            self.turn_system.process_turn_cycle(&mut self.ecs_world.world, &mut self.ecs_world.resources)?;
        }
        
        // Check for auto-save
        if let Some(auto_save) = &mut self.save_system {
            if let Ok(save_data) = self.ecs_world.to_save_data() {
                if let Err(e) = auto_save.try_save(&save_data) {
                    eprintln!("Auto-save failed: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Update game state by running all systems
    fn update(&mut self) -> anyhow::Result<()> {
        for system in &mut self.systems {
            match system.run(&mut self.ecs_world.world, &mut self.ecs_world.resources) {
                SystemResult::Continue => continue,
                SystemResult::Stop => {
                    self.is_running = false;
                    break;
                }
                SystemResult::Error(msg) => {
                    eprintln!("System error: {}", msg);
                    return Err(anyhow::anyhow!(msg));
                }
            }
        }
        
        // Check for auto-save
        if let Some(auto_save) = &mut self.save_system {
            if let Ok(save_data) = self.ecs_world.to_save_data() {
                if let Err(e) = auto_save.try_save(&save_data) {
                    eprintln!("Auto-save failed: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Render the current game state
    fn render(&mut self) -> anyhow::Result<()> {
        self.renderer.draw(&mut self.ecs_world)?;
        Ok(())
    }
    
    /// Clean up resources
    fn cleanup(&mut self) -> anyhow::Result<()> {
        self.renderer.cleanup()?;
        Ok(())
    }
    
    /// Save the current game state
    pub fn save_game(&mut self, slot: usize) -> anyhow::Result<()> {
        if let Some(auto_save) = &mut self.save_system {
            let save_data = self.ecs_world.to_save_data()?;
            auto_save.save_system.save_game(slot, &save_data)?;
        }
        Ok(())
    }
    
    /// Load a saved game state
    pub fn load_game(&mut self, slot: usize) -> anyhow::Result<()> {
        if let Some(auto_save) = &mut self.save_system {
            let save_data = auto_save.save_system.load_game(slot)?;
            self.ecs_world.from_save_data(save_data)?;
        }
        Ok(())
    }
}

/// Headless game loop for testing purposes
pub struct HeadlessGameLoop {
    pub game_engine: GameEngine,
    pub ecs_world: ECSWorld,
    pub systems: Vec<Box<dyn System>>,
    pub is_running: bool,
    pub save_system: Option<AutoSave>,
}

impl HeadlessGameLoop {
    pub fn new() -> Self {
        let systems: Vec<Box<dyn System>> = vec![
            Box::new(InputSystem),
            Box::new(TimeSystem),
            Box::new(MovementSystem),
            Box::new(AISystem),
            Box::new(CombatSystem),
            Box::new(FOVSystem),
            Box::new(EffectSystem),
            Box::new(EnergySystem),
            Box::new(InventorySystem),
            Box::new(DungeonSystem),
            Box::new(RenderingSystem),
        ];
        
        let mut ecs_world = ECSWorld::new();
        let game_engine = GameEngine::new();
        
        let save_system = match SaveSystem::new("saves", 10) {
            Ok(save_sys) => Some(AutoSave::new(save_sys, std::time::Duration::from_secs(300))), // 5 min auto-save
            Err(e) => {
                eprintln!("Failed to initialize save system: {}", e);
                None
            }
        };
        
        Self {
            game_engine,
            ecs_world,
            systems,
            is_running: true,
            save_system,
        }
    }
    
    /// Run the game loop without rendering (for testing)
    pub fn run_for_ticks(&mut self, ticks: u32) -> anyhow::Result<()> {
        for _ in 0..ticks {
            if !self.is_running {
                break;
            }
            
            self.update()?;
        }
        
        Ok(())
    }
    
    /// Save the current game state
    pub fn save_game(&mut self, slot: usize) -> anyhow::Result<()> {
        if let Some(auto_save) = &mut self.save_system {
            let save_data = self.ecs_world.to_save_data()?;
            auto_save.save_system.save_game(slot, &save_data)?;
        }
        Ok(())
    }
    
    /// Load a saved game state
    pub fn load_game(&mut self, slot: usize) -> anyhow::Result<()> {
        if let Some(auto_save) = &mut self.save_system {
            let save_data = auto_save.save_system.load_game(slot)?;
            self.ecs_world.from_save_data(save_data)?;
        }
        Ok(())
    }
    
    /// Update game state by running all systems
    fn update(&mut self) -> anyhow::Result<()> {
        for system in &mut self.systems {
            match system.run(&mut self.ecs_world.world, &mut self.ecs_world.resources) {
                SystemResult::Continue => continue,
                SystemResult::Stop => {
                    self.is_running = false;
                    break;
                }
                SystemResult::Error(msg) => {
                    eprintln!("System error: {}", msg);
                    return Err(anyhow::anyhow!(msg));
                }
            }
        }
        
        // Check for auto-save
        if let Some(auto_save) = &mut self.save_system {
            if let Ok(save_data) = self.ecs_world.to_save_data() {
                if let Err(e) = auto_save.try_save(&save_data) {
                    eprintln!("Auto-save failed: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

/// Helper function to convert internal key event to player action
fn key_event_to_player_action_from_internal(key_event: KeyEvent) -> Option<PlayerAction> {
    match (key_event.code, key_event.modifiers.shift) {
        // Movement keys
        (KeyCode::Char('k'), false) | (KeyCode::Up, false) => Some(PlayerAction::Move(Direction::North)),
        (KeyCode::Char('j'), false) | (KeyCode::Down, false) => Some(PlayerAction::Move(Direction::South)),
        (KeyCode::Char('h'), false) | (KeyCode::Left, false) => Some(PlayerAction::Move(Direction::West)),
        (KeyCode::Char('l'), false) | (KeyCode::Right, false) => Some(PlayerAction::Move(Direction::East)),
        (KeyCode::Char('y'), false) => Some(PlayerAction::Move(Direction::NorthWest)),
        (KeyCode::Char('u'), false) => Some(PlayerAction::Move(Direction::NorthEast)),
        (KeyCode::Char('b'), false) => Some(PlayerAction::Move(Direction::SouthWest)),
        (KeyCode::Char('n'), false) => Some(PlayerAction::Move(Direction::SouthEast)),
        
        // Wait/skip turn
        (KeyCode::Char('.'), false) => Some(PlayerAction::Wait),
        
        // Stairs
        (KeyCode::Char('>'), false) => Some(PlayerAction::Descend),
        (KeyCode::Char('<'), false) => Some(PlayerAction::Ascend),
        
        // Attack via direction
        (KeyCode::Char('K'), true) => Some(PlayerAction::Attack(Position { x: 0, y: -1, z: 0 })),
        (KeyCode::Char('J'), true) => Some(PlayerAction::Attack(Position { x: 0, y: 1, z: 0 })),
        (KeyCode::Char('H'), true) => Some(PlayerAction::Attack(Position { x: -1, y: 0, z: 0 })),
        (KeyCode::Char('L'), true) => Some(PlayerAction::Attack(Position { x: 1, y: 0, z: 0 })),
        (KeyCode::Char('Y'), true) => Some(PlayerAction::Attack(Position { x: -1, y: -1, z: 0 })),
        (KeyCode::Char('U'), true) => Some(PlayerAction::Attack(Position { x: 1, y: -1, z: 0 })),
        (KeyCode::Char('B'), true) => Some(PlayerAction::Attack(Position { x: -1, y: 1, z: 0 })),
        (KeyCode::Char('N'), true) => Some(PlayerAction::Attack(Position { x: 1, y: 1, z: 0 })),
        
        // Game control
        (KeyCode::Char('q'), false) => Some(PlayerAction::Quit),
        
        // Number keys for items/spells
        (KeyCode::Char('1'), false) => Some(PlayerAction::UseItem(0)),
        (KeyCode::Char('2'), false) => Some(PlayerAction::UseItem(1)),
        (KeyCode::Char('3'), false) => Some(PlayerAction::UseItem(2)),
        (KeyCode::Char('4'), false) => Some(PlayerAction::UseItem(3)),
        (KeyCode::Char('5'), false) => Some(PlayerAction::UseItem(4)),
        (KeyCode::Char('6'), false) => Some(PlayerAction::UseItem(5)),
        (KeyCode::Char('7'), false) => Some(PlayerAction::UseItem(6)),
        (KeyCode::Char('8'), false) => Some(PlayerAction::UseItem(7)),
        (KeyCode::Char('9'), false) => Some(PlayerAction::UseItem(8)),
        
        // Drop item
        (KeyCode::Char('d'), false) => Some(PlayerAction::DropItem(0)),
        
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::RatatuiRenderer;
    use crate::input::ConsoleInput;
    use crate::renderer::GameClock;
    
    #[test]
    fn test_game_loop_creation() -> anyhow::Result<()> {
        let renderer = RatatuiRenderer::new()?;
        let input_source = ConsoleInput::new();
        let clock = GameClock::new(16); // ~60 FPS
        
        let mut game_loop = GameLoop::new(renderer, input_source, clock);
        
        // Initialize the game loop
        game_loop.initialize()?;
        
        // Check that entities were initialized
        assert!(game_loop.ecs_world.world.iter().count() > 0);
        
        Ok(())
    }
}