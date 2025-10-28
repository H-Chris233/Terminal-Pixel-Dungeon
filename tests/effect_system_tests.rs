// tests/effect_system_tests.rs
//! Tests for per-turn status effect lifecycle management

use terminal_pixel_dungeon::ecs::{
    Actor, ECSWorld, Energy, Faction, Position, Stats, StatusEffects, Player,
};
use terminal_pixel_dungeon::systems::{EffectPhase, EffectSystem, SystemResult};
use combat::effect::{Effect, EffectType};

/// Helper function to create a test entity with status effects
fn create_test_entity(world: &mut ECSWorld, name: &str, hp: u32, is_player: bool) -> hecs::Entity {
    let entity = world.world.spawn((
        Position::new(0, 0, 0),
        Actor {
            name: name.to_string(),
            faction: if is_player { Faction::Player } else { Faction::Enemy },
        },
        Stats {
            hp,
            max_hp: 100,
            attack: 10,
            defense: 5,
            accuracy: 80,
            evasion: 20,
            level: 1,
            experience: 0,
            class: None,
        },
        StatusEffects::new(),
        Energy {
            current: 100,
            max: 100,
            regeneration_rate: 1,
        },
    ));
    
    if is_player {
        let _ = world.world.insert_one(entity, Player);
    }
    
    entity
}

#[test]
fn test_poison_effect_damage_over_time() {
    let mut ecs_world = ECSWorld::new();
    let entity = create_test_entity(&mut ecs_world, "Test Entity", 100, false);
    
    // Add poison effect (3 turns, intensity 5 = 15 damage per turn)
    let poison = Effect::with_intensity(EffectType::Poison, 3, 5);
    
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(poison);
    }
    
    // Verify initial HP
    let initial_hp = ecs_world.world.get::<&Stats>(entity).unwrap().hp;
    assert_eq!(initial_hp, 100);
    
    // Process first turn
    ecs_world.resources.clock.turn_count = 1;
    let result = EffectSystem::run_with_events(&mut ecs_world, EffectPhase::EndOfTurn);
    assert!(matches!(result, SystemResult::Continue));
    
    // Check HP after first tick (should be 100 - 15 = 85)
    let hp_after_turn1 = ecs_world.world.get::<&Stats>(entity).unwrap().hp;
    assert_eq!(hp_after_turn1, 85, "Poison should deal 15 damage per turn");
    
    // Verify effect still active with 2 turns remaining
    let remaining_turns = ecs_world.world.get::<&StatusEffects>(entity)
        .unwrap()
        .effects[0]
        .turns();
    assert_eq!(remaining_turns, 2);
    
    // Process second turn
    ecs_world.resources.clock.turn_count = 2;
    let result = EffectSystem::run_with_events(&mut ecs_world, EffectPhase::EndOfTurn);
    assert!(matches!(result, SystemResult::Continue));
    
    // Check HP after second tick (should be 85 - 15 = 70)
    let hp_after_turn2 = ecs_world.world.get::<&Stats>(entity).unwrap().hp;
    assert_eq!(hp_after_turn2, 70);
    
    // Process third turn
    ecs_world.resources.clock.turn_count = 3;
    let result = EffectSystem::run_with_events(&mut ecs_world, EffectPhase::EndOfTurn);
    assert!(matches!(result, SystemResult::Continue));
    
    // Check HP after third tick (should be 70 - 15 = 55)
    let hp_after_turn3 = ecs_world.world.get::<&Stats>(entity).unwrap().hp;
    assert_eq!(hp_after_turn3, 55);
    
    // Effect should now be expired and removed
    let effects = &ecs_world.world.get::<&StatusEffects>(entity).unwrap().effects;
    assert!(effects.is_empty(), "Effect should be removed after expiration");
}

#[test]
fn test_overlapping_buffs_stacking() {
    let mut ecs_world = ECSWorld::new();
    let entity = create_test_entity(&mut ecs_world, "Test Entity", 100, false);
    
    // Add first poison effect
    let poison1 = Effect::with_intensity(EffectType::Poison, 2, 3);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(poison1);
    }
    
    // Add second poison effect (should stack)
    let poison2 = Effect::with_intensity(EffectType::Poison, 3, 4);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(poison2);
    }
    
    // Should have 2 poison effects stacked
    {
        let effect_count = ecs_world.world.get::<&StatusEffects>(entity).unwrap().effects.len();
        assert_eq!(effect_count, 2, "Poison effects should stack");
    }
    
    // Process turn - both should deal damage
    ecs_world.resources.clock.turn_count = 1;
    let result = EffectSystem::run_with_events(&mut ecs_world, EffectPhase::EndOfTurn);
    assert!(matches!(result, SystemResult::Continue));
    
    // Total damage should be (3*3 + 4*3) = 9 + 12 = 21
    let hp_after = ecs_world.world.get::<&Stats>(entity).unwrap().hp;
    assert_eq!(hp_after, 79, "Both poison effects should deal damage");
}

#[test]
fn test_mutually_exclusive_effects() {
    let mut ecs_world = ECSWorld::new();
    let entity = create_test_entity(&mut ecs_world, "Test Entity", 100, false);
    
    // Add burning effect
    let burning = Effect::with_intensity(EffectType::Burning, 3, 5);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(burning);
    }
    
    // Verify burning is active
    assert!(ecs_world.world.get::<&StatusEffects>(entity)
        .unwrap()
        .has_effect(EffectType::Burning));
    
    // Try to add frost (should remove burning)
    let frost = Effect::with_intensity(EffectType::Frost, 2, 3);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(frost);
    }
    
    // Burning should be removed, frost should be active
    let status_effects = ecs_world.world.get::<&StatusEffects>(entity).unwrap();
    assert!(!status_effects.has_effect(EffectType::Burning), "Burning should be removed");
    assert!(status_effects.has_effect(EffectType::Frost), "Frost should be active");
}

#[test]
fn test_haste_slow_conflict() {
    let mut ecs_world = ECSWorld::new();
    let entity = create_test_entity(&mut ecs_world, "Test Entity", 100, false);
    
    // Add haste effect
    let haste = Effect::new(EffectType::Haste, 3);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(haste);
    }
    
    assert!(ecs_world.world.get::<&StatusEffects>(entity)
        .unwrap()
        .has_effect(EffectType::Haste));
    
    // Add slow (should remove haste)
    let slow = Effect::new(EffectType::Slow, 2);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(slow);
    }
    
    let status_effects = ecs_world.world.get::<&StatusEffects>(entity).unwrap();
    assert!(!status_effects.has_effect(EffectType::Haste), "Haste should be removed");
    assert!(status_effects.has_effect(EffectType::Slow), "Slow should be active");
}

#[test]
fn test_effect_removal_on_death() {
    let mut ecs_world = ECSWorld::new();
    let entity = create_test_entity(&mut ecs_world, "Test Entity", 20, false);
    
    // Add high intensity poison that will kill the entity
    let poison = Effect::with_intensity(EffectType::Poison, 5, 10); // 30 damage per turn
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(poison);
    }
    
    // Process turn - entity should die
    ecs_world.resources.clock.turn_count = 1;
    let result = EffectSystem::run_with_events(&mut ecs_world, EffectPhase::EndOfTurn);
    assert!(matches!(result, SystemResult::Continue));
    
    // Entity should be despawned
    assert!(ecs_world.world.get::<&Stats>(entity).is_err(), "Entity should be despawned on death");
}

#[test]
fn test_non_stackable_effect_updates_duration() {
    let mut ecs_world = ECSWorld::new();
    let entity = create_test_entity(&mut ecs_world, "Test Entity", 100, false);
    
    // Add invisibility effect (non-stackable)
    let invisibility1 = Effect::with_intensity(EffectType::Invisibility, 2, 3);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(invisibility1);
    }
    
    // Add another invisibility with longer duration
    let invisibility2 = Effect::with_intensity(EffectType::Invisibility, 5, 5);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(invisibility2);
    }
    
    // Should only have 1 invisibility effect
    let status_effects = ecs_world.world.get::<&StatusEffects>(entity).unwrap();
    assert_eq!(status_effects.effects.len(), 1, "Non-stackable effects should not stack");
    
    // Should have max duration and intensity (5 turns, intensity 5)
    let effect = &status_effects.effects[0];
    assert_eq!(effect.turns(), 5, "Should keep max duration");
    assert_eq!(effect.intensity(), 5, "Should keep max intensity");
}

#[test]
fn test_multiple_dot_effects_simultaneous() {
    let mut ecs_world = ECSWorld::new();
    let entity = create_test_entity(&mut ecs_world, "Test Entity", 100, false);
    
    // Add burning, poison, and bleeding
    let burning = Effect::with_intensity(EffectType::Burning, 3, 2); // 4 damage/turn
    let poison = Effect::with_intensity(EffectType::Poison, 3, 3); // 9 damage/turn
    let bleeding = Effect::with_intensity(EffectType::Bleeding, 3, 2); // 8 damage/turn
    
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(burning);
        status_effects.add_effect(poison);
        status_effects.add_effect(bleeding);
    }
    
    // All three should be active
    {
        let status_effects = ecs_world.world.get::<&StatusEffects>(entity).unwrap();
        assert_eq!(status_effects.effects.len(), 3, "All DoT effects should be active");
    }
    
    // Process turn
    ecs_world.resources.clock.turn_count = 1;
    let result = EffectSystem::run_with_events(&mut ecs_world, EffectPhase::EndOfTurn);
    assert!(matches!(result, SystemResult::Continue));
    
    // Total damage: 4 + 9 + 8 = 21
    let hp_after = ecs_world.world.get::<&Stats>(entity).unwrap().hp;
    assert_eq!(hp_after, 79, "All DoT effects should deal damage");
}

#[test]
fn test_effect_expiration_timing() {
    let mut ecs_world = ECSWorld::new();
    let entity = create_test_entity(&mut ecs_world, "Test Entity", 100, false);
    
    // Add effect with 1 turn duration
    let poison = Effect::with_intensity(EffectType::Poison, 1, 3);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(poison);
    }
    
    // Process turn 1 - should deal damage and expire
    ecs_world.resources.clock.turn_count = 1;
    let result = EffectSystem::run_with_events(&mut ecs_world, EffectPhase::EndOfTurn);
    assert!(matches!(result, SystemResult::Continue));
    
    // Check that damage was applied
    let hp = ecs_world.world.get::<&Stats>(entity).unwrap().hp;
    assert_eq!(hp, 91, "Should deal damage on the last turn");
    
    // Effect should be removed
    let effects = &ecs_world.world.get::<&StatusEffects>(entity).unwrap().effects;
    assert!(effects.is_empty(), "Effect should expire after 1 turn");
}

#[test]
fn test_paralysis_prevents_no_damage() {
    let mut ecs_world = ECSWorld::new();
    let entity = create_test_entity(&mut ecs_world, "Test Entity", 100, false);
    
    // Add paralysis effect (should not deal damage)
    let paralysis = Effect::with_intensity(EffectType::Paralysis, 3, 5);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(paralysis);
    }
    
    // Process turn
    ecs_world.resources.clock.turn_count = 1;
    let result = EffectSystem::run_with_events(&mut ecs_world, EffectPhase::EndOfTurn);
    assert!(matches!(result, SystemResult::Continue));
    
    // HP should not change (paralysis doesn't deal damage)
    let hp = ecs_world.world.get::<&Stats>(entity).unwrap().hp;
    assert_eq!(hp, 100, "Paralysis should not deal damage");
    
    // Effect should still have 2 turns remaining
    let remaining_turns = ecs_world.world.get::<&StatusEffects>(entity)
        .unwrap()
        .effects[0]
        .turns();
    assert_eq!(remaining_turns, 2);
}

#[test]
fn test_event_publishing_on_effect_tick() {
    let mut ecs_world = ECSWorld::new();
    let entity = create_test_entity(&mut ecs_world, "Test Entity", 100, false);
    
    // Add poison effect
    let poison = Effect::with_intensity(EffectType::Poison, 2, 5);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(poison);
    }
    
    // Process turn (should publish StatusEffectTicked event)
    ecs_world.resources.clock.turn_count = 1;
    let result = EffectSystem::run_with_events(&mut ecs_world, EffectPhase::EndOfTurn);
    assert!(matches!(result, SystemResult::Continue));
    
    // Process events to update message log
    ecs_world.process_events();
    
    // Check message log for effect tick message
    let messages = &ecs_world.resources.game_state.message_log;
    assert!(messages.iter().any(|m| m.contains("Poison") && m.contains("造成")), 
        "Should log effect damage message");
}

#[test]
fn test_effect_expiration_event() {
    let mut ecs_world = ECSWorld::new();
    let entity = create_test_entity(&mut ecs_world, "Test Entity", 100, false);
    
    // Add short duration effect
    let poison = Effect::with_intensity(EffectType::Poison, 1, 3);
    if let Ok(mut status_effects) = ecs_world.world.get::<&mut StatusEffects>(entity) {
        status_effects.add_effect(poison);
    }
    
    // Process turn (effect should expire)
    ecs_world.resources.clock.turn_count = 1;
    let result = EffectSystem::run_with_events(&mut ecs_world, EffectPhase::EndOfTurn);
    assert!(matches!(result, SystemResult::Continue));
    
    // Process events to update message log
    ecs_world.process_events();
    
    // Check message log for expiration message
    let messages = &ecs_world.resources.game_state.message_log;
    assert!(messages.iter().any(|m| m.contains("消失")), 
        "Should log effect expiration message");
}
