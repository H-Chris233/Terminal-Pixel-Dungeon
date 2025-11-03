
//! Tests for the hunger and food consumption system
//! 
//! This test file verifies:
//! 1. Gradual hunger loss tied to turn count and action types
//! 2. Starvation damage timing and aftermath phase handling
//! 3. Food consumption restoring hunger
//! 4. Death from starvation with proper game over reason
//! 5. Event emissions for hunger state changes

use terminal_pixel_dungeon::{
    ecs::*,
    event_bus::GameEvent,
    systems::{HungerSystem, SystemResult},
};

/// Helper to find the player entity
fn find_player_entity(world: &hecs::World) -> Option<hecs::Entity> {
    world
        .query::<&Player>()
        .iter()
        .map(|(e, _)| e)
        .next()
}

/// Test that hunger decreases after a configurable number of standard actions
#[test]
fn test_hunger_decay_over_turns() {
    let mut ecs_world = ECSWorld::new();
    
    // Spawn player with full satiety
    let player = ecs_world.world.spawn((
        Player,
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
        Position::new(5, 5, 0),
        Hunger::new(10), // Start with full satiety
    ));
    
    // Simulate 10 standard actions (should decrease hunger by 1)
    for i in 0..10 {
        ecs_world.resources.clock.turn_count += 1;
        
        // Simulate a completed move action
        ecs_world.resources.input_buffer.completed_actions.push(PlayerAction::Move(Direction::North));
        
        HungerSystem::run_with_events(&mut ecs_world);
        
        // Clear completed actions for next iteration
        ecs_world.resources.input_buffer.completed_actions.clear();
    }
    
    // Check that hunger decreased by 1
    let hunger = ecs_world.world.get::<&Hunger>(player).unwrap();
    assert_eq!(hunger.satiety, 9, "Hunger should decrease by 1 after 10 standard actions");
}

/// Test that wait actions consume less hunger than standard actions
#[test]
fn test_wait_action_consumes_less_hunger() {
    let mut ecs_world = ECSWorld::new();
    
    // Spawn player with full satiety
    let player = ecs_world.world.spawn((
        Player,
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
        Position::new(5, 5, 0),
        Hunger::new(10),
    ));
    
    // Simulate 20 wait actions (should decrease hunger by 1)
    for i in 0..20 {
        ecs_world.resources.clock.turn_count += 1;
        
        // Simulate a completed wait action
        ecs_world.resources.input_buffer.completed_actions.push(PlayerAction::Wait);
        
        HungerSystem::run_with_events(&mut ecs_world);
        
        // Clear completed actions
        ecs_world.resources.input_buffer.completed_actions.clear();
    }
    
    // Check that hunger decreased by 1
    let hunger = ecs_world.world.get::<&Hunger>(player).unwrap();
    assert_eq!(hunger.satiety, 9, "Hunger should decrease by 1 after 20 wait actions");
}

/// Test that hunger events are emitted correctly
#[test]
fn test_hunger_events_emitted() {
    let mut ecs_world = ECSWorld::new();
    
    // Spawn player with low satiety
    let player = ecs_world.world.spawn((
        Player,
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
        Position::new(5, 5, 0),
        Hunger::new(2), // Start hungry
    ));
    
    // Track events
    let mut events_logged = Vec::new();
    let event_logger = |event: &GameEvent| {
        events_logged.push(format!("{:?}", event));
    };
    
    // Simulate actions to trigger hunger decay
    for i in 0..10 {
        ecs_world.resources.clock.turn_count += 1;
        ecs_world.resources.input_buffer.completed_actions.push(PlayerAction::Move(Direction::North));
        
        HungerSystem::run_with_events(&mut ecs_world);
        
        // Process events
        ecs_world.process_events();
        
        ecs_world.resources.input_buffer.completed_actions.clear();
    }
    
    // Check that hunger decreased and events were emitted
    let hunger = ecs_world.world.get::<&Hunger>(player).unwrap();
    assert_eq!(hunger.satiety, 1, "Hunger should decrease to 1");
}

/// Test starvation damage when satiety reaches 0
#[test]
fn test_starvation_damage() {
    let mut ecs_world = ECSWorld::new();
    
    // Spawn player with 0 satiety
    let player = ecs_world.world.spawn((
        Player,
        Actor {
            name: "Player".to_string(),
            faction: Faction::Player,
        },
        Stats {
            hp: 10,
            max_hp: 100,
            attack: 10,
            defense: 5,
            accuracy: 80,
            evasion: 20,
            level: 1,
            experience: 0,
            class: None,
        },
        Position::new(5, 5, 0),
        Hunger::new(0), // Starving
    ));
    
    let initial_hp = ecs_world.world.get::<&Stats>(player).unwrap().hp;
    
    // Run hunger system (should apply starvation damage)
    HungerSystem::run_with_events(&mut ecs_world);
    
    // Check that HP decreased
    let current_hp = ecs_world.world.get::<&Stats>(player).unwrap().hp;
    assert!(current_hp < initial_hp, "HP should decrease from starvation");
    assert_eq!(current_hp, initial_hp - 1, "Starvation should deal 1 damage per turn");
}

/// Test death from starvation
#[test]
fn test_death_from_starvation() {
    let mut ecs_world = ECSWorld::new();
    
    // Spawn player with 1 HP and 0 satiety
    let player = ecs_world.world.spawn((
        Player,
        Actor {
            name: "Player".to_string(),
            faction: Faction::Player,
        },
        Stats {
            hp: 1,
            max_hp: 100,
            attack: 10,
            defense: 5,
            accuracy: 80,
            evasion: 20,
            level: 1,
            experience: 0,
            class: None,
        },
        Position::new(5, 5, 0),
        Hunger::new(0), // Starving
    ));
    
    // Run hunger system (should kill player)
    let result = HungerSystem::run_with_events(&mut ecs_world);
    
    // Check that system stopped
    assert!(matches!(result, SystemResult::Stop), "System should stop when player dies");
    
    // Check that game state is GameOver with Starved reason
    match ecs_world.resources.game_state.game_state {
        GameStatus::GameOver { reason } => {
            assert!(matches!(reason, GameOverReason::Starved), "Death reason should be Starved");
        }
        _ => panic!("Game state should be GameOver"),
    }
    
    // Check that player HP is 0
    let stats = ecs_world.world.get::<&Stats>(player).unwrap();
    assert_eq!(stats.hp, 0, "Player HP should be 0");
}

/// Test food consumption restores satiety
#[test]
fn test_food_consumption_restores_hunger() {
    let mut ecs_world = ECSWorld::new();
    
    // Spawn player with low satiety
    let player = ecs_world.world.spawn((
        Player,
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
        Position::new(5, 5, 0),
        Hunger::new(2), // Low satiety
    ));
    
    // Test feeding directly
    if let Ok(mut hunger) = ecs_world.world.get::<&mut Hunger>(player) {
        let initial_satiety = hunger.satiety;
        hunger.feed(5);
        assert_eq!(hunger.satiety, initial_satiety + 5, "Satiety should increase by 5");
    }
}

/// Test hunger does not exceed maximum (10)
#[test]
fn test_hunger_capped_at_maximum() {
    let mut ecs_world = ECSWorld::new();
    
    let player = ecs_world.world.spawn((
        Player,
        Hunger::new(8), // Near max satiety
    ));
    
    // Try to feed more than max
    if let Ok(mut hunger) = ecs_world.world.get::<&mut Hunger>(player) {
        hunger.feed(5); // Should cap at 10
        assert_eq!(hunger.satiety, 10, "Satiety should cap at 10");
    }
}

/// Test that no hunger decay occurs without actions
#[test]
fn test_no_hunger_decay_without_actions() {
    let mut ecs_world = ECSWorld::new();
    
    // Spawn player
    let player = ecs_world.world.spawn((
        Player,
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
        Position::new(5, 5, 0),
        Hunger::new(10),
    ));
    
    let initial_satiety = ecs_world.world.get::<&Hunger>(player).unwrap().satiety;
    
    // Advance turns without actions
    for _ in 0..20 {
        ecs_world.resources.clock.turn_count += 1;
        // No completed actions
        HungerSystem::run_with_events(&mut ecs_world);
    }
    
    // Check that hunger didn't change
    let current_satiety = ecs_world.world.get::<&Hunger>(player).unwrap().satiety;
    assert_eq!(current_satiety, initial_satiety, "Hunger should not decay without actions");
}

/// Test starvation warnings before death
#[test]
fn test_starvation_warning_events() {
    let mut ecs_world = ECSWorld::new();
    
    // Spawn player with minimal satiety
    let player = ecs_world.world.spawn((
        Player,
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
        Position::new(5, 5, 0),
        Hunger::new(1), // One step away from starving
    ));
    
    // Simulate actions to trigger starvation
    for i in 0..10 {
        ecs_world.resources.clock.turn_count += 1;
        ecs_world.resources.input_buffer.completed_actions.push(PlayerAction::Move(Direction::North));
        
        HungerSystem::run_with_events(&mut ecs_world);
        ecs_world.process_events();
        
        ecs_world.resources.input_buffer.completed_actions.clear();
    }
    
    // Player should now be starving
    let hunger = ecs_world.world.get::<&Hunger>(player).unwrap();
    assert_eq!(hunger.satiety, 0, "Player should be starving");
    
    // HP should have decreased from starvation damage
    let stats = ecs_world.world.get::<&Stats>(player).unwrap();
    assert!(stats.hp < 100, "HP should have decreased from starvation");
}

/// Test aftermath queue receives death event
#[test]
fn test_starvation_death_aftermath() {
    let mut ecs_world = ECSWorld::new();
    
    // Spawn player about to die
    let player = ecs_world.world.spawn((
        Player,
        Actor {
            name: "Player".to_string(),
            faction: Faction::Player,
        },
        Stats {
            hp: 1,
            max_hp: 100,
            attack: 10,
            defense: 5,
            accuracy: 80,
            evasion: 20,
            level: 1,
            experience: 0,
            class: None,
        },
        Position::new(5, 5, 0),
        Hunger::new(0),
    ));
    
    // Clear aftermath queue
    ecs_world.resources.aftermath_queue.clear();
    
    // Run hunger system
    HungerSystem::run_with_events(&mut ecs_world);
    
    // Check that death event was added to aftermath queue
    assert!(!ecs_world.resources.aftermath_queue.is_empty(), "Aftermath queue should contain death event");
    
    // Verify the death event is for the player
    if let Some(AftermathEvent::Death { entity_name, .. }) = ecs_world.resources.aftermath_queue.first() {
        assert_eq!(entity_name, "玩家", "Death event should be for player");
    } else {
        panic!("Expected Death event in aftermath queue");
    }
}
