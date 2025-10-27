use terminal_pixel_dungeon::ecs::*;
use terminal_pixel_dungeon::turn_system::{TurnState, TurnSystem};
use save::{SaveSystem, SAVE_VERSION};
use hero::class::Class;
use std::path::PathBuf;

#[test]
fn test_save_and_restore_turn_state() {
    // Create a temporary save directory
    let temp_dir = std::env::temp_dir().join("tpd_test_saves");
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    
    // Initialize save system
    let save_system = SaveSystem::new(&temp_dir, 10).expect("Failed to create save system");
    
    // Create ECS world and turn system
    let mut ecs_world = ECSWorld::new();
    let mut turn_system = TurnSystem::new();
    
    // Initialize game world
    ecs_world.generate_and_set_dungeon(5, 12345).expect("Failed to generate dungeon");
    
    // Create player entity
    let player_entity = ecs_world.world.spawn((
        Position::new(10, 10, 0),
        Actor {
            name: "TestHero".to_string(),
            faction: Faction::Player,
        },
        Stats {
            hp: 75,
            max_hp: 100,
            attack: 15,
            defense: 8,
            accuracy: 85,
            evasion: 25,
            level: 3,
            experience: 150,
            class: Some(Class::Warrior),
        },
        Inventory {
            items: vec![],
            max_slots: 20,
        },
        Hunger {
            satiety: 7,
            last_hunger_turn: 10,
        },
        Wealth {
            gold: 250,
        },
        PlayerProgress {
            turns: 42,
            strength: 12,
            class: Class::Warrior,
            skill_state: hero::class::SkillState::default(),
        },
        Viewshed {
            range: 8,
            visible_tiles: vec![],
            memory: vec![],
            dirty: true,
            algorithm: FovAlgorithm::default(),
        },
        Energy {
            current: 50,
            max: 100,
            regeneration_rate: 1,
        },
        Player,
    ));
    
    // Set clock state
    ecs_world.resources.clock.turn_count = 42;
    ecs_world.resources.clock.elapsed_time = std::time::Duration::from_secs(300);
    
    // Set turn system to AI turn (mid-combat state)
    turn_system.set_state(TurnState::AITurn, true);
    
    // Save the game
    let save_data = ecs_world.to_save_data(&turn_system).expect("Failed to create save data");
    save_system.save_game(0, &save_data).expect("Failed to save game");
    
    // Store original state for comparison
    let original_turn_count = ecs_world.resources.clock.turn_count;
    let original_elapsed = ecs_world.resources.clock.elapsed_time.as_secs();
    let original_turn_state = turn_system.state.clone();
    
    // Clear the world
    ecs_world.clear();
    turn_system = TurnSystem::new(); // Reset turn system
    
    // Load the game
    let loaded_data = save_system.load_game(0).expect("Failed to load game");
    assert_eq!(loaded_data.version, SAVE_VERSION, "Save version mismatch");
    
    let (restored_turn_state, restored_action_taken) = ecs_world.from_save_data(loaded_data)
        .expect("Failed to restore save data");
    turn_system.set_state(restored_turn_state.clone(), restored_action_taken);
    
    // Verify turn state was restored
    assert_eq!(turn_system.state, original_turn_state, "Turn state not restored correctly");
    assert_eq!(turn_system.player_action_taken(), true, "Player action taken flag not restored");
    
    // Verify clock state was restored
    assert_eq!(ecs_world.resources.clock.turn_count, original_turn_count, "Turn count not restored");
    assert_eq!(ecs_world.resources.clock.elapsed_time.as_secs(), original_elapsed, "Elapsed time not restored");
    
    // Verify player entity was restored with correct components
    let mut player_found = false;
    for (entity, _) in ecs_world.world.query::<&Player>().iter() {
        player_found = true;
        
        // Check stats
        let stats = ecs_world.world.get::<&Stats>(entity).expect("Player missing Stats");
        assert_eq!(stats.hp, 75, "HP not restored");
        assert_eq!(stats.level, 3, "Level not restored");
        
        // Check energy
        let energy = ecs_world.world.get::<&Energy>(entity).expect("Player missing Energy");
        assert_eq!(energy.current, 50, "Energy not restored");
        
        // Check hunger
        let hunger = ecs_world.world.get::<&Hunger>(entity).expect("Player missing Hunger");
        assert_eq!(hunger.satiety, 7, "Hunger not restored");
        assert_eq!(hunger.last_hunger_turn, 10, "Hunger turn not restored");
        
        // Check progress
        let progress = ecs_world.world.get::<&PlayerProgress>(entity).expect("Player missing Progress");
        assert_eq!(progress.turns, 42, "Progress turns not restored");
        assert_eq!(progress.strength, 12, "Strength not restored");
    }
    
    assert!(player_found, "Player entity not found after loading");
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_backward_compatibility_v1_saves() {
    // This test ensures old saves (v1) can be loaded with default turn state
    let temp_dir = std::env::temp_dir().join("tpd_test_saves_v1");
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    
    let save_system = SaveSystem::new(&temp_dir, 10).expect("Failed to create save system");
    
    // Create a minimal v1-style save (without turn state)
    let mut ecs_world = ECSWorld::new();
    ecs_world.generate_and_set_dungeon(3, 54321).expect("Failed to generate dungeon");
    
    let player_entity = ecs_world.world.spawn((
        Position::new(5, 5, 0),
        Actor {
            name: "LegacyHero".to_string(),
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
            class: Some(Class::Mage),
        },
        Inventory { items: vec![], max_slots: 20 },
        Hunger::default(),
        Wealth::default(),
        PlayerProgress {
            turns: 0,
            strength: 10,
            class: Class::Mage,
            skill_state: hero::class::SkillState::default(),
        },
        Viewshed {
            range: 8,
            visible_tiles: vec![],
            memory: vec![],
            dirty: true,
            algorithm: FovAlgorithm::default(),
        },
        Energy { current: 100, max: 100, regeneration_rate: 1 },
        Player,
    ));
    
    let turn_system = TurnSystem::new();
    let mut save_data = ecs_world.to_save_data(&turn_system).expect("Failed to create save data");
    
    // Simulate v1 save by setting version to 1
    save_data.version = 1;
    
    save_system.save_game(1, &save_data).expect("Failed to save v1 game");
    
    // Load and verify migration
    let mut loaded_data = save_system.load_game(1).expect("Failed to load v1 game");
    
    // Migration should have upgraded to v2
    assert_eq!(loaded_data.version, SAVE_VERSION, "Version not migrated");
    
    // Turn state should be at defaults after migration
    assert_eq!(loaded_data.turn_state.current_phase, save::TurnPhase::PlayerTurn);
    assert_eq!(loaded_data.turn_state.player_action_taken, false);
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_save_mid_combat_with_enemies() {
    let temp_dir = std::env::temp_dir().join("tpd_test_saves_combat");
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    
    let save_system = SaveSystem::new(&temp_dir, 10).expect("Failed to create save system");
    let mut ecs_world = ECSWorld::new();
    let mut turn_system = TurnSystem::new();
    
    ecs_world.generate_and_set_dungeon(3, 99999).expect("Failed to generate dungeon");
    
    // Create player
    ecs_world.world.spawn((
        Position::new(10, 10, 0),
        Actor {
            name: "CombatHero".to_string(),
            faction: Faction::Player,
        },
        Stats {
            hp: 80,
            max_hp: 100,
            attack: 12,
            defense: 6,
            accuracy: 80,
            evasion: 20,
            level: 2,
            experience: 50,
            class: Some(Class::Rogue),
        },
        Inventory { items: vec![], max_slots: 20 },
        Hunger { satiety: 5, last_hunger_turn: 20 },
        Wealth { gold: 100 },
        PlayerProgress {
            turns: 50,
            strength: 11,
            class: Class::Rogue,
            skill_state: hero::class::SkillState::default(),
        },
        Viewshed {
            range: 8,
            visible_tiles: vec![],
            memory: vec![],
            dirty: true,
            algorithm: FovAlgorithm::default(),
        },
        Energy { current: 30, max: 100, regeneration_rate: 1 },
        Player,
    ));
    
    // Create enemy with energy state
    ecs_world.world.spawn((
        Position::new(12, 10, 0),
        Actor {
            name: "Goblin".to_string(),
            faction: Faction::Enemy,
        },
        Stats {
            hp: 25,
            max_hp: 30,
            attack: 8,
            defense: 3,
            accuracy: 70,
            evasion: 15,
            level: 1,
            experience: 0,
            class: None,
        },
        Renderable {
            symbol: 'g',
            fg_color: Color::Green,
            bg_color: None,
            order: 5,
        },
        Energy { current: 75, max: 100, regeneration_rate: 1 },
        AI {
            ai_type: AIType::Aggressive,
            target: None,
            state: AIState::Chasing,
        },
        Viewshed {
            range: 6,
            visible_tiles: vec![],
            memory: vec![],
            dirty: true,
            algorithm: FovAlgorithm::default(),
        },
        Effects { active_effects: vec![] },
    ));
    
    // Set to AI turn (mid-combat)
    turn_system.set_state(TurnState::AITurn, true);
    ecs_world.resources.clock.turn_count = 50;
    
    // Save
    let save_data = ecs_world.to_save_data(&turn_system).expect("Failed to save mid-combat");
    save_system.save_game(2, &save_data).expect("Failed to save to slot 2");
    
    // Verify entities were captured in save data
    assert_eq!(save_data.entities.len(), 1, "Enemy not captured in save");
    assert_eq!(save_data.entities[0].name, "Goblin");
    assert_eq!(save_data.entities[0].hp, 25);
    assert_eq!(save_data.entities[0].energy_current, 75);
    
    // Load and verify
    let loaded = save_system.load_game(2).expect("Failed to load combat save");
    let (restored_state, _) = ecs_world.from_save_data(loaded).expect("Failed to restore");
    
    assert_eq!(restored_state, TurnState::AITurn);
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}
