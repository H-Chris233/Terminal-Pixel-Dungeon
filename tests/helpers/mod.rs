#![cfg(feature = "legacy-tests")]

//! Test helpers and builders for creating ECS worlds with scripted turn sequences.
//!
//! This module provides utilities for setting up deterministic test scenarios
//! across multiple systems (movement, combat, AI, status effects, etc.).

use hecs::{Entity, World};
use rand::SeedableRng;
use rand::rngs::StdRng;
use terminal_pixel_dungeon::ecs::*;
use terminal_pixel_dungeon::event_bus::{EventBus, GameEvent};
use terminal_pixel_dungeon::turn_system::TurnSystem;

/// Builder for creating test ECS worlds with deterministic setups
pub struct TestWorldBuilder {
    world: World,
    resources: Resources,
    event_bus: EventBus,
    seed: u64,
}

impl TestWorldBuilder {
    /// Create a new test world builder with a deterministic seed
    pub fn new(seed: u64) -> Self {
        let mut resources = Resources::default();
        resources.rng = StdRng::seed_from_u64(seed);
        
        Self {
            world: World::new(),
            resources,
            event_bus: EventBus::new(),
            seed,
        }
    }

    /// Add a player entity at the specified position
    pub fn with_player(mut self, x: i32, y: i32, z: i32) -> Self {
        let player_entity = self.world.spawn((
            Position::new(x, y, z),
            Actor {
                name: "Test Player".to_string(),
                faction: Faction::Player,
            },
            Player,
            Stats {
                hp: 100,
                max_hp: 100,
                attack: 10,
                defense: 5,
                accuracy: 80,
                evasion: 20,
                strength: 10,
                dexterity: 10,
                intelligence: 10,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 20,
            },
            Inventory {
                items: Vec::new(),
                capacity: 20,
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            },
            Renderable {
                glyph: '@',
                fg: Color::Yellow,
                bg: Color::Black,
            },
            Hunger {
                current: 1000,
                max: 1500,
                starving_threshold: 100,
                hungry_threshold: 300,
            },
            Wealth { gold: 0 },
            PlayerProgress {
                experience: 0,
                level: 1,
                kills: 0,
                depth_reached: 1,
            },
        ));

        // Store player entity for quick access
        self.resources.player_entity = Some(player_entity);
        self
    }

    /// Add an AI enemy at the specified position
    pub fn with_enemy(
        mut self,
        x: i32,
        y: i32,
        z: i32,
        name: &str,
        faction: Faction,
    ) -> (Self, Entity) {
        let enemy_entity = self.world.spawn((
            Position::new(x, y, z),
            Actor {
                name: name.to_string(),
                faction,
            },
            Stats {
                hp: 50,
                max_hp: 50,
                attack: 8,
                defense: 3,
                accuracy: 70,
                evasion: 15,
                strength: 8,
                dexterity: 8,
                intelligence: 8,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 20,
            },
            AI {
                ai_type: AIType::Hostile,
                target: None,
                path: Vec::new(),
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: 6,
                dirty: true,
            },
            Renderable {
                glyph: 'r',
                fg: Color::Red,
                bg: Color::Black,
            },
        ));

        (self, enemy_entity)
    }

    /// Add a tile at the specified position
    pub fn with_tile(mut self, x: i32, y: i32, z: i32, terrain: TerrainType) -> Self {
        self.world.spawn((
            Position::new(x, y, z),
            Tile {
                terrain,
                is_passable: matches!(terrain, TerrainType::Floor | TerrainType::StairDown | TerrainType::StairUp),
                blocks_vision: matches!(terrain, TerrainType::Wall),
            },
        ));
        self
    }

    /// Add a simple dungeon floor layout (5x5 room centered at origin)
    pub fn with_simple_dungeon(mut self, z: i32) -> Self {
        // Create a simple 10x10 floor
        for x in 0..10 {
            for y in 0..10 {
                let terrain = if x == 0 || y == 0 || x == 9 || y == 9 {
                    TerrainType::Wall
                } else {
                    TerrainType::Floor
                };
                self = self.with_tile(x, y, z, terrain);
            }
        }
        self
    }

    /// Set the game state
    pub fn with_game_state(mut self, state: GameStatus) -> Self {
        self.resources.game_state.game_state = state;
        self
    }

    /// Add an item to the world
    pub fn with_item(mut self, x: i32, y: i32, z: i32, item: ECSItem) -> (Self, Entity) {
        let item_entity = self.world.spawn((
            Position::new(x, y, z),
            item,
            Renderable {
                glyph: '!',
                fg: Color::Cyan,
                bg: Color::Black,
            },
        ));
        (self, item_entity)
    }

    /// Build the test world
    pub fn build(self) -> TestWorld {
        TestWorld {
            ecs_world: ECSWorld {
                world: self.world,
                resources: self.resources,
                event_bus: self.event_bus,
            },
            turn_system: TurnSystem::new(),
        }
    }
}

/// A complete test world with ECS and turn system
pub struct TestWorld {
    pub ecs_world: ECSWorld,
    pub turn_system: TurnSystem,
}

impl TestWorld {
    /// Execute a player action and process the turn
    pub fn execute_player_action(&mut self, action: PlayerAction) {
        self.ecs_world.resources.input_buffer.pending_actions.push(action);
    }

    /// Process a complete turn cycle
    pub fn process_turn(&mut self) -> Result<(), anyhow::Error> {
        self.turn_system.process_turn_cycle(
            &mut self.ecs_world.world,
            &mut self.ecs_world.resources,
        )
    }

    /// Get the player entity
    pub fn player_entity(&self) -> Option<Entity> {
        self.ecs_world.resources.player_entity
    }

    /// Get player stats
    pub fn player_stats(&self) -> Option<Stats> {
        if let Some(player) = self.player_entity() {
            self.ecs_world.world.get::<&Stats>(player).ok().map(|s| s.clone())
        } else {
            None
        }
    }

    /// Get player position
    pub fn player_position(&self) -> Option<Position> {
        if let Some(player) = self.player_entity() {
            self.ecs_world.world.get::<&Position>(player).ok().map(|p| p.clone())
        } else {
            None
        }
    }

    /// Get player energy
    pub fn player_energy(&self) -> Option<Energy> {
        if let Some(player) = self.player_entity() {
            self.ecs_world.world.get::<&Energy>(player).ok().map(|e| e.clone())
        } else {
            None
        }
    }

    /// Check if an entity is alive
    pub fn is_entity_alive(&self, entity: Entity) -> bool {
        self.ecs_world.world.get::<&Stats>(entity)
            .map(|stats| stats.hp > 0)
            .unwrap_or(false)
    }

    /// Get entity stats
    pub fn entity_stats(&self, entity: Entity) -> Option<Stats> {
        self.ecs_world.world.get::<&Stats>(entity).ok().map(|s| s.clone())
    }

    /// Collect all events from the event bus
    pub fn collect_events(&mut self) -> Vec<GameEvent> {
        self.ecs_world.event_bus.drain().collect()
    }

    /// Set entity HP directly (for testing edge cases)
    pub fn set_entity_hp(&mut self, entity: Entity, hp: u32) {
        if let Ok(mut stats) = self.ecs_world.world.get::<&mut Stats>(entity) {
            stats.hp = hp;
        }
    }

    /// Set entity energy directly (for testing energy mechanics)
    pub fn set_entity_energy(&mut self, entity: Entity, energy: u32) {
        if let Ok(mut e) = self.ecs_world.world.get::<&mut Energy>(entity) {
            e.current = energy;
        }
    }

    /// Count entities with a specific component
    pub fn count_entities_with<T: 'static>(&self) -> usize {
        self.ecs_world.world.query::<&T>().iter().count()
    }

    /// Get the current turn state
    pub fn turn_state(&self) -> &terminal_pixel_dungeon::turn_system::TurnState {
        &self.turn_system.state
    }
}

/// Scripted turn sequence builder for complex test scenarios
pub struct TurnSequenceBuilder {
    actions: Vec<PlayerAction>,
}

impl TurnSequenceBuilder {
    pub fn new() -> Self {
        Self { actions: Vec::new() }
    }

    /// Add a movement action
    pub fn move_direction(mut self, direction: Direction) -> Self {
        self.actions.push(PlayerAction::Move(direction));
        self
    }

    /// Add a wait action
    pub fn wait(mut self) -> Self {
        self.actions.push(PlayerAction::Wait);
        self
    }

    /// Add an attack action
    pub fn attack(mut self, direction: Direction) -> Self {
        self.actions.push(PlayerAction::Attack(direction));
        self
    }

    /// Add an item use action
    pub fn use_item(mut self, slot: ItemSlot) -> Self {
        self.actions.push(PlayerAction::UseItem(slot));
        self
    }

    /// Build the action sequence
    pub fn build(self) -> Vec<PlayerAction> {
        self.actions
    }

    /// Execute the sequence on a test world
    pub fn execute(self, test_world: &mut TestWorld) -> Result<(), anyhow::Error> {
        for action in self.actions {
            test_world.execute_player_action(action);
            test_world.process_turn()?;
        }
        Ok(())
    }
}

impl Default for TurnSequenceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_builder_creates_player() {
        let world = TestWorldBuilder::new(42)
            .with_player(5, 5, 0)
            .build();

        assert!(world.player_entity().is_some());
        let pos = world.player_position().unwrap();
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
        assert_eq!(pos.z, 0);
    }

    #[test]
    fn test_world_builder_creates_enemy() {
        let (builder, enemy) = TestWorldBuilder::new(42)
            .with_enemy(3, 3, 0, "Test Rat", Faction::Enemy);
        let world = builder.build();

        assert!(world.is_entity_alive(enemy));
    }

    #[test]
    fn test_turn_sequence_builder() {
        let sequence = TurnSequenceBuilder::new()
            .move_direction(Direction::North)
            .wait()
            .move_direction(Direction::East)
            .build();

        assert_eq!(sequence.len(), 3);
    }
}
