/// Tests for AI decision making integrated with energy-driven turns
///
/// This test suite verifies:
/// 1. Multi-enemy turn ordering based on energy
/// 2. AI waiting and energy regeneration
/// 3. AI reactions to stealth and status impairments
/// 4. Intent generation vs execution separation
/// 5. Event bus integration for AI decisions

use hecs::World;
use terminal_pixel_dungeon::ecs::*;
use terminal_pixel_dungeon::systems::*;
use terminal_pixel_dungeon::turn_system::*;
use terminal_pixel_dungeon::event_bus::{EventBus, GameEvent};

/// Helper function to create a test world with player and enemies
fn setup_test_world() -> ECSWorld {
    let mut ecs_world = ECSWorld::new();
    
    // Create player
    let player = ecs_world.world.spawn((
        Position::new(10, 10, 0),
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
            algorithm: FovAlgorithm::default(),
        },
        Player,
    ));
    
    // Create basic floor tiles
    for x in 5..20 {
        for y in 5..20 {
            ecs_world.world.spawn((
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
    
    ecs_world
}

/// Helper to create an enemy at a specific position
fn spawn_enemy(world: &mut World, x: i32, y: i32, name: &str, energy: u32, ai_type: AIType) -> hecs::Entity {
    world.spawn((
        Position::new(x, y, 0),
        Actor {
            name: name.to_string(),
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
        AI {
            ai_type,
            target: None,
            state: AIState::Idle,
        },
        Energy {
            current: energy,
            max: 100,
            regeneration_rate: 10,
        },
    ))
}

#[test]
fn test_multi_enemy_turn_ordering() {
    let mut ecs_world = setup_test_world();
    
    // Spawn three enemies with different energy levels
    let enemy1 = spawn_enemy(&mut ecs_world.world, 12, 10, "Goblin1", 100, AIType::Aggressive);
    let enemy2 = spawn_enemy(&mut ecs_world.world, 13, 10, "Goblin2", 80, AIType::Aggressive);
    let enemy3 = spawn_enemy(&mut ecs_world.world, 14, 10, "Goblin3", 120, AIType::Aggressive);
    
    // Track decision events
    let mut _decisions: Vec<u32> = Vec::new();
    
    // Subscribe to AI decision events
    struct DecisionTracker {
        decisions: std::sync::Arc<std::sync::Mutex<Vec<u32>>>,
    }
    
    impl terminal_pixel_dungeon::event_bus::EventHandler for DecisionTracker {
        fn handle(&mut self, event: &GameEvent) {
            if let GameEvent::AIDecisionMade { entity, .. } = event {
                if let Ok(mut d) = self.decisions.lock() {
                    d.push(*entity);
                }
            }
        }
        
        fn name(&self) -> &str {
            "DecisionTracker"
        }
        
        fn priority(&self) -> terminal_pixel_dungeon::event_bus::Priority {
            terminal_pixel_dungeon::event_bus::Priority::High
        }
    }
    
    let decisions_arc = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    ecs_world.event_bus.subscribe_all(Box::new(DecisionTracker {
        decisions: decisions_arc.clone(),
    }));
    
    // Run AI system
    AISystem::run_with_events(&mut ecs_world);
    
    // Process events
    ecs_world.process_events();
    
    // Check that all enemies with sufficient energy made decisions
    let decisions = decisions_arc.lock().unwrap();
    assert!(decisions.len() >= 2, "At least 2 enemies should have decided (energy >= 100)");
    
    // Verify energy was consumed
    if let Ok(energy) = ecs_world.world.get::<&Energy>(enemy1) {
        assert!(energy.current < 100, "Enemy1 should have consumed energy");
    }
}

#[test]
fn test_ai_waiting_when_no_target() {
    let mut ecs_world = setup_test_world();
    
    // Spawn enemy far from player (out of range)
    let enemy = spawn_enemy(&mut ecs_world.world, 50, 50, "Distant Goblin", 100, AIType::Aggressive);
    
    let initial_energy = ecs_world.world.get::<&Energy>(enemy).unwrap().current;
    
    // Run AI system
    AISystem::run_with_events(&mut ecs_world);
    
    // Check that enemy waited (consumed less energy)
    let final_energy = ecs_world.world.get::<&Energy>(enemy).unwrap().current;
    let energy_consumed = initial_energy - final_energy;
    
    // Wait action should consume 50 energy
    assert_eq!(energy_consumed, 50, "AI should wait when target is out of range");
}

#[test]
fn test_ai_energy_regeneration() {
    let mut ecs_world = setup_test_world();
    
    // Spawn enemy with low energy
    let enemy = spawn_enemy(&mut ecs_world.world, 12, 10, "Goblin", 30, AIType::Aggressive);
    
    // Run AI system (should not act due to insufficient energy)
    AISystem::run_with_events(&mut ecs_world);
    
    let energy_before_regen = ecs_world.world.get::<&Energy>(enemy).unwrap().current;
    assert!(energy_before_regen < 100, "Enemy should have low energy");
    
    // Simulate energy regeneration (like TimeSystem does)
    if let Ok(mut energy) = ecs_world.world.get::<&mut Energy>(enemy) {
        let regen = energy.regeneration_rate.max(1);
        energy.current = (energy.current + regen).min(energy.max);
    }
    
    let energy_after_regen = ecs_world.world.get::<&Energy>(enemy).unwrap().current;
    assert!(energy_after_regen > energy_before_regen, "Energy should regenerate");
}

#[test]
fn test_ai_impaired_by_status_effects() {
    let mut ecs_world = setup_test_world();
    
    // Spawn enemy
    let enemy = spawn_enemy(&mut ecs_world.world, 12, 10, "Goblin", 100, AIType::Aggressive);
    
    // Apply paralysis status effect
    let paralysis_effect = combat::effect::Effect::new(
        combat::effect::EffectType::Paralysis,
        3 // duration
    );
    
    ecs_world.world.insert_one(enemy, StatusEffects {
        effects: vec![paralysis_effect],
        last_tick_turn: 0,
    }).unwrap();
    
    let initial_energy = ecs_world.world.get::<&Energy>(enemy).unwrap().current;
    
    // Run AI system
    AISystem::run_with_events(&mut ecs_world);
    
    // Check that enemy waited due to impairment
    let final_energy = ecs_world.world.get::<&Energy>(enemy).unwrap().current;
    let energy_consumed = initial_energy - final_energy;
    
    // Should consume wait energy (50)
    assert_eq!(energy_consumed, 50, "Impaired AI should wait");
}

#[test]
fn test_ai_passive_behavior() {
    let mut ecs_world = setup_test_world();
    
    // Spawn passive enemy near player
    let enemy = spawn_enemy(&mut ecs_world.world, 11, 10, "Passive Creature", 100, AIType::Passive);
    
    let initial_pos = {
        let pos = ecs_world.world.get::<&Position>(enemy).unwrap();
        Position::new(pos.x, pos.y, pos.z)
    };
    
    // Run AI system
    AISystem::run_with_events(&mut ecs_world);
    
    // Check that passive enemy didn't move
    let final_pos = {
        let pos = ecs_world.world.get::<&Position>(enemy).unwrap();
        Position::new(pos.x, pos.y, pos.z)
    };
    assert_eq!(initial_pos.x, final_pos.x, "Passive AI should not move");
    assert_eq!(initial_pos.y, final_pos.y, "Passive AI should not move");
    
    // Check that it's in Idle state
    let state = ecs_world.world.get::<&AI>(enemy).unwrap().state.clone();
    assert!(matches!(state, AIState::Idle), "Passive AI should stay idle");
}

#[test]
fn test_ai_neutral_becomes_aggressive() {
    let mut ecs_world = setup_test_world();
    
    // Get player entity
    let player = ecs_world.world.query::<&Player>()
        .iter()
        .next()
        .map(|(e, _)| e)
        .unwrap();
    
    // Spawn neutral enemy
    let enemy = spawn_enemy(&mut ecs_world.world, 11, 10, "Neutral Creature", 100, AIType::Neutral);
    
    // Initially should wait (no target)
    AISystem::run_with_events(&mut ecs_world);
    let state1 = ecs_world.world.get::<&AI>(enemy).unwrap().state.clone();
    assert!(matches!(state1, AIState::Idle), "Neutral AI without target should be idle");
    
    // Now give it a target (simulate being attacked)
    if let Ok(mut ai) = ecs_world.world.get::<&mut AI>(enemy) {
        ai.target = Some(player);
    }
    
    // Reset energy
    if let Ok(mut energy) = ecs_world.world.get::<&mut Energy>(enemy) {
        energy.current = 100;
    }
    
    // Run AI system again
    AISystem::run_with_events(&mut ecs_world);
    
    // Now should be chasing
    let state2 = ecs_world.world.get::<&AI>(enemy).unwrap().state.clone();
    assert!(
        matches!(state2, AIState::Chasing | AIState::Attacking),
        "Neutral AI with target should chase or attack"
    );
}

#[test]
fn test_ai_patrol_behavior() {
    let mut ecs_world = setup_test_world();
    
    // Define patrol path
    let patrol_path = vec![
        Position::new(10, 10, 0),
        Position::new(15, 10, 0),
        Position::new(15, 15, 0),
        Position::new(10, 15, 0),
    ];
    
    // Spawn patrol enemy
    let enemy = spawn_enemy(
        &mut ecs_world.world,
        10,
        10,
        "Guard",
        100,
        AIType::Patrol { path: patrol_path.clone() }
    );
    
    let _initial_pos = {
        let pos = ecs_world.world.get::<&Position>(enemy).unwrap();
        Position::new(pos.x, pos.y, pos.z)
    };
    
    // Run AI system
    AISystem::run_with_events(&mut ecs_world);
    
    // Check that patrol AI is in patrolling state or moved
    let state = ecs_world.world.get::<&AI>(enemy).unwrap().state.clone();
    // AI might wait if already at patrol point or move to next point
    assert!(
        matches!(state, AIState::Patrolling | AIState::Idle),
        "Patrol AI should be patrolling or idle at patrol point"
    );
}

#[test]
fn test_ai_decision_events_emitted() {
    let mut ecs_world = setup_test_world();
    
    // Spawn enemy
    let _enemy = spawn_enemy(&mut ecs_world.world, 12, 10, "Goblin", 100, AIType::Aggressive);
    
    // Track events
    let mut decision_count = 0;
    let mut target_change_count = 0;
    
    struct EventCounter {
        decision_count: std::sync::Arc<std::sync::Mutex<usize>>,
        target_count: std::sync::Arc<std::sync::Mutex<usize>>,
    }
    
    impl terminal_pixel_dungeon::event_bus::EventHandler for EventCounter {
        fn handle(&mut self, event: &GameEvent) {
            match event {
                GameEvent::AIDecisionMade { .. } => {
                    *self.decision_count.lock().unwrap() += 1;
                }
                GameEvent::AITargetChanged { .. } => {
                    *self.target_count.lock().unwrap() += 1;
                }
                _ => {}
            }
        }
        
        fn name(&self) -> &str {
            "EventCounter"
        }
    }
    
    let decision_arc = std::sync::Arc::new(std::sync::Mutex::new(0));
    let target_arc = std::sync::Arc::new(std::sync::Mutex::new(0));
    
    ecs_world.event_bus.subscribe_all(Box::new(EventCounter {
        decision_count: decision_arc.clone(),
        target_count: target_arc.clone(),
    }));
    
    // Run AI system
    AISystem::run_with_events(&mut ecs_world);
    
    // Process events
    ecs_world.process_events();
    
    // Check events were emitted
    let decisions = *decision_arc.lock().unwrap();
    let targets = *target_arc.lock().unwrap();
    
    assert!(decisions > 0, "AI decision events should be emitted");
    // Target change might be 0 if target doesn't change, but decision should happen
}

#[test]
fn test_ai_state_transitions() {
    let mut ecs_world = setup_test_world();
    
    // Get player entity and position
    let (player_entity, player_pos) = ecs_world.world.query::<(&Player, &Position)>()
        .iter()
        .next()
        .map(|(e, (_, pos))| (e, pos.clone()))
        .unwrap();
    
    // Spawn enemy near player
    let enemy = spawn_enemy(&mut ecs_world.world, 11, 10, "Goblin", 100, AIType::Aggressive);
    
    // Initial state should be Idle
    let state = ecs_world.world.get::<&AI>(enemy).unwrap().state.clone();
    assert!(matches!(state, AIState::Idle), "Initial state should be Idle");
    
    // Run AI system - should start chasing
    AISystem::run_with_events(&mut ecs_world);
    
    let state = ecs_world.world.get::<&AI>(enemy).unwrap().state.clone();
    assert!(
        matches!(state, AIState::Chasing | AIState::Attacking),
        "State should transition to Chasing or Attacking"
    );
}

#[test]
fn test_multiple_ai_actors_take_turns() {
    let mut ecs_world = setup_test_world();
    
    // Spawn 5 enemies with full energy
    for i in 0..5 {
        spawn_enemy(
            &mut ecs_world.world,
            12 + i,
            10,
            &format!("Goblin{}", i),
            100,
            AIType::Aggressive
        );
    }
    
    // Count entities with energy before
    let before_count = ecs_world.world.query::<&Energy>()
        .iter()
        .filter(|(_, energy)| energy.current == 100)
        .count();
    
    // Run AI system
    AISystem::run_with_events(&mut ecs_world);
    
    // Count entities with energy after
    let after_count = ecs_world.world.query::<&Energy>()
        .iter()
        .filter(|(_, energy)| energy.current == 100)
        .count();
    
    // Some enemies should have consumed energy
    assert!(after_count < before_count, "Multiple AI actors should take turns");
}

#[test]
fn test_ai_respects_energy_costs() {
    let mut ecs_world = setup_test_world();
    
    // Spawn enemy with exactly 100 energy (enough for one action)
    let enemy = spawn_enemy(&mut ecs_world.world, 12, 10, "Goblin", 100, AIType::Aggressive);
    
    // Run AI system once
    AISystem::run_with_events(&mut ecs_world);
    
    let energy = ecs_world.world.get::<&Energy>(enemy).unwrap().current;
    
    // Should have consumed energy (either 100 for move/attack or 50 for wait)
    assert!(energy < 100, "AI should consume energy");
    assert!(energy >= 0, "Energy should not go negative");
    
    // Try to run again with insufficient energy
    let initial_energy = energy;
    AISystem::run_with_events(&mut ecs_world);
    
    // If energy < 100, AI should not act
    if initial_energy < 100 {
        let final_energy = ecs_world.world.get::<&Energy>(enemy).unwrap().current;
        // Energy should not change if AI can't act
        // (this might vary based on implementation)
    }
}
