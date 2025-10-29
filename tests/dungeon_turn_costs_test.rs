// Tests for dungeon/environment interaction turn costs

use terminal_pixel_dungeon::ecs::*;
use terminal_pixel_dungeon::systems::*;
use terminal_pixel_dungeon::turn_system::{energy_costs, TurnSystem};
use hecs::World;

/// Helper to create a basic test world with floor tiles
fn create_test_world() -> (World, Resources) {
    let mut world = World::new();
    let resources = Resources::default();

    // Create basic floor tiles
    for x in 0..20 {
        for y in 0..20 {
            world.spawn((
                Position::new(x, y, 0),
                Tile {
                    terrain_type: TerrainType::Floor,
                    is_passable: true,
                    blocks_sight: false,
                    has_items: false,
                    has_monster: false,
                },
            ));
        }
    }

    (world, resources)
}

/// Helper to create a player entity
fn create_player(world: &mut World, x: i32, y: i32, z: i32) -> hecs::Entity {
    world.spawn((
        Position::new(x, y, z),
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
            class: None,
        },
        Viewshed {
            range: 8,
            visible_tiles: vec![],
            memory: vec![],
            dirty: true,
            algorithm: FovAlgorithm::default(),
        },
        Energy {
            current: 100,
            max: 100,
            regeneration_rate: 10,
        },
        Player,
    ))
}

#[test]
fn test_stair_use_energy_cost() {
    let mut ecs_world = ECSWorld::new();
    let player = create_player(&mut ecs_world.world, 10, 10, 0);

    // Place stairs down at player position
    ecs_world.world.spawn((
        Position::new(10, 10, 0),
        Tile {
            terrain_type: TerrainType::StairsDown,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    // Create stairs up at the destination level
    ecs_world.world.spawn((
        Position::new(10, 10, 1),
        Tile {
            terrain_type: TerrainType::StairsUp,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    // Get initial energy
    let initial_energy = ecs_world.world.get::<&Energy>(player).unwrap().current;
    
    // Queue descend action
    ecs_world.resources.input_buffer.pending_actions.push(PlayerAction::Descend);

    // Run dungeon system with events
    DungeonSystem::run_with_events(&mut ecs_world);

    // Verify action was completed
    assert_eq!(ecs_world.resources.input_buffer.completed_actions.len(), 1);
    assert!(matches!(
        ecs_world.resources.input_buffer.completed_actions[0],
        PlayerAction::Descend
    ));

    // Create turn system and consume energy
    let mut turn_system = TurnSystem::new();
    turn_system.consume_player_energy(&mut ecs_world.world, &PlayerAction::Descend).unwrap();

    // Verify energy was deducted
    let final_energy = ecs_world.world.get::<&Energy>(player).unwrap().current;
    assert_eq!(initial_energy - final_energy, energy_costs::STAIR_USE);
}

#[test]
fn test_stair_descend_resets_viewshed() {
    let mut ecs_world = ECSWorld::new();
    let player = create_player(&mut ecs_world.world, 10, 10, 0);

    // Place stairs down at player position
    ecs_world.world.spawn((
        Position::new(10, 10, 0),
        Tile {
            terrain_type: TerrainType::StairsDown,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    // Set viewshed to not dirty and add some visible tiles
    if let Ok(mut viewshed) = ecs_world.world.get::<&mut Viewshed>(player) {
        viewshed.dirty = false;
        viewshed.visible_tiles = vec![
            Position::new(9, 9, 0),
            Position::new(10, 10, 0),
            Position::new(11, 11, 0)
        ];
    }

    // Queue descend action
    ecs_world.resources.input_buffer.pending_actions.push(PlayerAction::Descend);

    // Run dungeon system with events
    DungeonSystem::run_with_events(&mut ecs_world);

    // Verify viewshed was reset
    let viewshed = ecs_world.world.get::<&Viewshed>(player).unwrap();
    assert!(viewshed.dirty, "Viewshed should be marked dirty");
    assert_eq!(viewshed.visible_tiles.len(), 0, "Visible tiles should be cleared");
}

#[test]
fn test_stair_descend_publishes_events() {
    let mut ecs_world = ECSWorld::new();
    let player = create_player(&mut ecs_world.world, 10, 10, 0);

    // Place stairs down
    ecs_world.world.spawn((
        Position::new(10, 10, 0),
        Tile {
            terrain_type: TerrainType::StairsDown,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    // Queue descend action
    ecs_world.resources.input_buffer.pending_actions.push(PlayerAction::Descend);

    // Run dungeon system
    DungeonSystem::run_with_events(&mut ecs_world);

    // Process events
    ecs_world.process_events();

    // Verify depth changed
    assert_eq!(ecs_world.resources.game_state.depth, 1);

    // Verify player position changed
    let pos = ecs_world.world.get::<&Position>(player).unwrap();
    assert_eq!(pos.z, 1);
}

#[test]
fn test_stair_ascend_from_top_level_fails() {
    let mut ecs_world = ECSWorld::new();
    let player = create_player(&mut ecs_world.world, 10, 10, 0);

    // Place stairs up at level 0
    ecs_world.world.spawn((
        Position::new(10, 10, 0),
        Tile {
            terrain_type: TerrainType::StairsUp,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    // Queue ascend action
    ecs_world.resources.input_buffer.pending_actions.push(PlayerAction::Ascend);

    // Run dungeon system
    DungeonSystem::run_with_events(&mut ecs_world);

    // Verify depth didn't change
    assert_eq!(ecs_world.resources.game_state.depth, 0);

    // Verify player position didn't change
    let pos = ecs_world.world.get::<&Position>(player).unwrap();
    assert_eq!(pos.z, 0);

    // Verify action was not completed
    assert_eq!(ecs_world.resources.input_buffer.completed_actions.len(), 0);
}

#[test]
fn test_terrain_movement_costs() {
    // Test floor terrain
    assert_eq!(
        energy_costs::terrain_movement_cost(&TerrainType::Floor),
        energy_costs::TERRAIN_FLOOR
    );

    // Test water terrain (slower)
    assert_eq!(
        energy_costs::terrain_movement_cost(&TerrainType::Water),
        energy_costs::TERRAIN_WATER
    );
    assert!(energy_costs::TERRAIN_WATER > energy_costs::TERRAIN_FLOOR);

    // Test stairs terrain (normal cost)
    assert_eq!(
        energy_costs::terrain_movement_cost(&TerrainType::StairsUp),
        energy_costs::TERRAIN_FLOOR
    );
    assert_eq!(
        energy_costs::terrain_movement_cost(&TerrainType::StairsDown),
        energy_costs::TERRAIN_FLOOR
    );
}

#[test]
fn test_get_terrain_energy_cost_at_position() {
    let mut world = World::new();

    // Place water tile at specific position
    world.spawn((
        Position::new(5, 5, 0),
        Tile {
            terrain_type: TerrainType::Water,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    // Place floor tile at a different position
    world.spawn((
        Position::new(3, 3, 0),
        Tile {
            terrain_type: TerrainType::Floor,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    // Check cost at water position
    let pos = Position::new(5, 5, 0);
    let cost = DungeonSystem::get_terrain_energy_cost(&world, &pos);
    assert_eq!(cost, energy_costs::TERRAIN_WATER);

    // Check cost at normal floor position
    let pos2 = Position::new(3, 3, 0);
    let cost2 = DungeonSystem::get_terrain_energy_cost(&world, &pos2);
    assert_eq!(cost2, energy_costs::TERRAIN_FLOOR);
}

#[test]
fn test_trap_activation_order() {
    let mut ecs_world = ECSWorld::new();
    let player = create_player(&mut ecs_world.world, 10, 10, 0);

    // Create floor tile first
    ecs_world.world.spawn((
        Position::new(11, 10, 0),
        Tile {
            terrain_type: TerrainType::Floor,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    // Place trap at destination
    ecs_world.world.spawn((
        Position::new(11, 10, 0),
        Tile {
            terrain_type: TerrainType::Trap,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    // Get initial HP
    let initial_hp = ecs_world.world.get::<&Stats>(player).unwrap().hp;

    // Trigger trap
    let pos = Position::new(11, 10, 0);
    DungeonSystem::check_and_trigger_trap(&mut ecs_world, player, &pos);

    // Process events
    ecs_world.process_events();

    // Verify damage was dealt
    let final_hp = ecs_world.world.get::<&Stats>(player).unwrap().hp;
    assert!(final_hp < initial_hp, "Player should take damage from trap");
}

#[test]
fn test_door_opening_with_events() {
    let mut ecs_world = ECSWorld::new();
    let player = create_player(&mut ecs_world.world, 10, 10, 0);

    // Place a closed door
    ecs_world.world.spawn((
        Position::new(11, 10, 0),
        Tile {
            terrain_type: TerrainType::Door,
            is_passable: false,
            blocks_sight: true,
            has_items: false,
            has_monster: false,
        },
    ));

    // Open the door
    let pos = Position::new(11, 10, 0);
    let opened = DungeonSystem::check_and_open_door(&mut ecs_world, player, &pos);

    assert!(opened, "Door should be opened");

    // Process events
    ecs_world.process_events();

    // Verify door tile was updated
    for (_, (tile_pos, tile)) in ecs_world.world.query::<(&Position, &Tile)>().iter() {
        if tile_pos.x == 11 && tile_pos.y == 10 && tile_pos.z == 0 {
            if matches!(tile.terrain_type, TerrainType::Door) {
                assert!(tile.is_passable, "Door should be passable after opening");
                assert!(!tile.blocks_sight, "Door should not block sight after opening");
            }
        }
    }
}

#[test]
fn test_level_transition_preserves_initiative_order() {
    let mut ecs_world = ECSWorld::new();
    let player = create_player(&mut ecs_world.world, 10, 10, 0);

    // Create an enemy
    let enemy = ecs_world.world.spawn((
        Position::new(12, 12, 0),
        Actor {
            name: "Goblin".to_string(),
            faction: Faction::Enemy,
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
            class: None,
        },
        Energy {
            current: 80,
            max: 100,
            regeneration_rate: 10,
        },
    ));

    // Get initial energy values
    let player_energy_before = ecs_world.world.get::<&Energy>(player).unwrap().current;
    let enemy_energy_before = ecs_world.world.get::<&Energy>(enemy).unwrap().current;

    // Place stairs
    ecs_world.world.spawn((
        Position::new(10, 10, 0),
        Tile {
            terrain_type: TerrainType::StairsDown,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    // Descend
    ecs_world.resources.input_buffer.pending_actions.push(PlayerAction::Descend);
    DungeonSystem::run_with_events(&mut ecs_world);

    // Verify energy values are preserved
    let player_energy_after = ecs_world.world.get::<&Energy>(player).unwrap().current;
    let enemy_energy_after = ecs_world.world.get::<&Energy>(enemy).unwrap().current;

    assert_eq!(player_energy_before, player_energy_after, "Player energy should be preserved");
    assert_eq!(enemy_energy_before, enemy_energy_after, "Enemy energy should be preserved");
}

#[test]
fn test_stair_use_without_standing_on_stairs() {
    let mut ecs_world = ECSWorld::new();
    let player = create_player(&mut ecs_world.world, 10, 10, 0);

    // Create only floor tiles, no stairs
    ecs_world.world.spawn((
        Position::new(10, 10, 0),
        Tile {
            terrain_type: TerrainType::Floor,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    // Try to descend without stairs
    ecs_world.resources.input_buffer.pending_actions.push(PlayerAction::Descend);
    DungeonSystem::run_with_events(&mut ecs_world);

    // Verify action was not completed
    assert_eq!(ecs_world.resources.input_buffer.completed_actions.len(), 0);

    // Verify position didn't change
    let pos = ecs_world.world.get::<&Position>(player).unwrap();
    assert_eq!(pos.z, 0);

    // Verify depth didn't change
    assert_eq!(ecs_world.resources.game_state.depth, 0);
}

#[test]
fn test_deterministic_trap_processing() {
    let mut ecs_world = ECSWorld::new();
    let player = create_player(&mut ecs_world.world, 10, 10, 0);

    // Place multiple traps and test they trigger in consistent order
    ecs_world.world.spawn((
        Position::new(11, 10, 0),
        Tile {
            terrain_type: TerrainType::Trap,
            is_passable: true,
            blocks_sight: false,
            has_items: false,
            has_monster: false,
        },
    ));

    let initial_hp = ecs_world.world.get::<&Stats>(player).unwrap().hp;

    // Trigger trap multiple times - should be consistent
    for _ in 0..3 {
        // Reset HP
        if let Ok(mut stats) = ecs_world.world.get::<&mut Stats>(player) {
            stats.hp = initial_hp;
        }

        let pos = Position::new(11, 10, 0);
        DungeonSystem::check_and_trigger_trap(&mut ecs_world, player, &pos);
        ecs_world.process_events();

        let hp_after = ecs_world.world.get::<&Stats>(player).unwrap().hp;
        let damage = initial_hp - hp_after;
        
        // Damage should be consistent (10 points as per simplified trap)
        assert_eq!(damage, 10, "Trap damage should be deterministic");
    }
}
