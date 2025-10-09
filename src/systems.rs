//! ECS Systems for game logic processing.

use crate::ecs::*;
use hecs::{Entity, World};

/// Trait for ECS systems that operate on the world
pub trait System {
    /// Run the system on the ECS world
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult;
}

/// Result of system execution
#[derive(Debug)]
pub enum SystemResult {
    Continue,
    Stop,
    Error(String),
}

/// Input processing system - converts raw input to game commands
pub struct InputSystem;

impl System for InputSystem {
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // In a real implementation, we would poll for input events and convert them to actions
        // This is handled by the input module in our architecture
        SystemResult::Continue
    }
}

/// Movement system handles actor movement and collision detection
pub struct MovementSystem;

impl System for MovementSystem {
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Process movement for all entities that have pending moves
        // For now, we'll process pending player actions from the input buffer
        let mut actions_to_process = std::mem::take(&mut resources.input_buffer.pending_actions);
        
        for action in actions_to_process.drain(..) {
            match action {
                PlayerAction::Move(dir) => {
                    // Find player entity
                    if let Some(player_entity) = find_player(world) {
                        // Get current position
                        if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
                            // Calculate new position based on direction
                            let (dx, dy) = match dir {
                                Direction::North => (0, -1),
                                Direction::South => (0, 1),
                                Direction::East => (1, 0),
                                Direction::West => (-1, 0),
                                Direction::NorthEast => (1, -1),
                                Direction::NorthWest => (-1, -1),
                                Direction::SouthEast => (1, 1),
                                Direction::SouthWest => (-1, 1),
                            };
                            
                            // Calculate target position
                            let target_x = pos.x + dx;
                            let target_y = pos.y + dy;
                            
                            // Check if the target position is passable
                            if is_position_passable(world, target_x, target_y, pos.z) {
                                pos.x = target_x;
                                pos.y = target_y;
                                
                                // Mark viewshed as dirty to recalculate FOV
                                if let Ok(mut viewshed) = world.get::<&mut Viewshed>(player_entity) {
                                    viewshed.dirty = true;
                                }
                            }
                        }
                    }
                }
                PlayerAction::Wait => {
                    // Increment turn count without moving
                    resources.clock.turn_count += 1;
                }
                PlayerAction::Quit => {
                    resources.game_state.game_state = GameStatus::GameOver;
                    return SystemResult::Stop;
                }
                PlayerAction::Attack(target_pos) => {
                    if let Some(player_entity) = find_player(world) {
                        // Attempt to find an entity at the target position
                        if let Some(target_entity) = find_entity_at_position(
                            world, 
                            target_pos.x, 
                            target_pos.y, 
                            target_pos.z
                        ) {
                            // Handle the attack
                            if let (Ok(player_pos), Ok(target_pos_comp)) = (
                                world.get::<&Position>(player_entity),
                                world.get::<&Position>(target_entity)
                            ) {
                                // Check if attack is valid (adjacent target)
                                let distance = player_pos.distance_to(&*target_pos_comp);
                                if distance <= 1.5 { // Adjacent
                                    process_attack(world, player_entity, target_entity);
                                }
                            }
                        }
                    }
                }
                PlayerAction::UseItem(index) => {
                    if let Some(player_entity) = find_player(world) {
                        use_item(world, player_entity, index);
                    }
                }
                PlayerAction::Descend => {
                    if let Ok(mut pos) = world.get::<&mut Position>(find_player(world).unwrap()) {
                        pos.z += 1; // Move to next dungeon level
                        resources.game_state.depth += 1;
                        
                        // May need to generate new level or load existing
                        // For now, just move to next level
                    }
                }
                PlayerAction::Ascend => {
                    if let Ok(mut pos) = world.get::<&mut Position>(find_player(world).unwrap()) {
                        if pos.z > 0 {
                            pos.z -= 1; // Move to previous dungeon level
                            resources.game_state.depth = resources.game_state.depth.saturating_sub(1);
                        }
                    }
                }
                PlayerAction::DropItem(index) => {
                    if let Some(player_entity) = find_player(world) {
                        drop_item(world, player_entity, index);
                    }
                }
            }
        }
        
        // Put back any unprocessed actions
        resources.input_buffer.pending_actions = actions_to_process;
        
        SystemResult::Continue
    }
}

/// Combat system handles combat encounters and damage calculations
pub struct CombatSystem;

impl System for CombatSystem {
    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        // Process attacks and combat interactions
        // This system would normally check for entities with pending attacks
        // For now, we handle attacks in the movement system
        SystemResult::Continue
    }
}

/// Field of View (FOV) system calculates what areas are visible to actors
pub struct FOVSystem;

impl System for FOVSystem {
    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        // Update field of view for all entities with Viewshed component
        for (entity, mut viewshed) in world.query_mut::<&mut Viewshed>() {
            if viewshed.dirty {
                // Calculate visible tiles using FOV algorithm
                let pos = &*world.get::<&Position>(entity).unwrap();
                viewshed.visible_tiles = calculate_fov(world, &*pos, viewshed.range);
                viewshed.dirty = false;
            }
        }
        
        SystemResult::Continue
    }
}

/// AI system processes non-player actors' behavior
pub struct AISystem;

impl System for AISystem {
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Process AI for all entities with AI component
        for (entity, (mut ai, mut pos, mut energy)) in 
            world.query_mut::<(&mut AI, &mut Position, &mut Energy)>() 
        {
            // Only process AI if entity has enough energy
            if energy.current < 10 {
                continue; // Not enough energy to act
            }
            
            // Determine target (usually player)
            let player_pos = if let Some(player_entity) = find_player(world) {
                world.get::<&Position>(player_entity).ok().map(|p| &*p)
            } else {
                None
            };
            
            // Update AI state based on environment and target
            match &mut ai.ai_type {
                AIType::Aggressive => {
                    if let Some(target_pos) = player_pos {
                        // Move towards player if in range (chasing)
                        let distance = pos.distance_to(&*target_pos);
                        
                        if distance <= 1.5 {
                            // Attack the player
                            if let Some(player_entity) = find_player(world) {
                                process_attack(world, entity, player_entity);
                            }
                        } else if distance <= ai.range() as f32 {
                            // Move towards player
                            let dx = (target_pos.x - pos.x).signum();
                            let dy = (target_pos.y - pos.y).signum();
                            
                            let new_x = pos.x + dx;
                            let new_y = pos.y + dy;
                            
                            if is_position_passable(world, new_x, new_y, pos.z) {
                                pos.x = new_x;
                                pos.y = new_y;
                            }
                        }
                    } else {
                        // No target found, stay in place
                    }
                }
                AIType::Passive => {
                    // Passive NPCs don't move or attack unless attacked
                }
                AIType::Neutral => {
                    // Neutral NPCs may react to nearby events
                }
                AIType::Patrol { path } => {
                    // Follow patrol path
                    if !path.is_empty() {
                        // Simple path following: move to next point in path
                        // This is simplified for the example
                    }
                }
            }
            
            // Consume energy for the AI action
            energy.current = energy.current.saturating_sub(10);
        }
        
        SystemResult::Continue
    }
}

/// Effect system processes active effects on actors
pub struct EffectSystem;

impl System for EffectSystem {
    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        // Process active effects on all entities
        for (entity, mut effects) in world.query_mut::<&mut Effects>() {
            let mut effects_to_remove = Vec::new();
            
            for (idx, effect) in effects.active_effects.iter_mut().enumerate() {
                // Handle timed effects
                if effect.duration > 0 {
                    effect.duration -= 1;
                    
                    // Apply effect
                    apply_effect_to_entity(world, entity, effect);
                    
                    if effect.duration == 0 {
                        effects_to_remove.push(idx);
                    }
                } else {
                    // Permanent effects are always active
                    apply_effect_to_entity(world, entity, effect);
                }
            }
            
            // Remove expired effects
            for &idx in effects_to_remove.iter().rev() {
                effects.active_effects.remove(idx);
            }
        }
        
        SystemResult::Continue
    }
}

/// Energy system manages entity energy and turn scheduling
pub struct EnergySystem;

impl System for EnergySystem {
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Regenerate energy for all entities with Energy component
        // This could be based on a time-based system or turn-based
        for (_, mut energy) in world.query_mut::<&mut Energy>() {
            if energy.current < energy.max {
                energy.current = std::cmp::min(energy.max, energy.current + energy.regeneration_rate);
            }
        }
        
        // Increment turn counter
        resources.clock.turn_count += 1;
        
        SystemResult::Continue
    }
}

/// Inventory system manages items and inventory operations
pub struct InventorySystem;

impl System for InventorySystem {
    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        // Process inventory operations like item identification,
        // equipment management, and consumable usage
        // This system would handle ongoing inventory effects
        
        SystemResult::Continue
    }
}

/// Rendering system prepares data for rendering
pub struct RenderingSystem;

impl System for RenderingSystem {
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Prepare rendering data - this would typically collect entities
        // that need to be rendered and pass them to the renderer
        resources.clock.elapsed_time += resources.clock.tick_rate;
        resources.clock.current_time = std::time::SystemTime::now();
        
        SystemResult::Continue
    }
}

/// Time system manages game time progression
pub struct TimeSystem;

impl System for TimeSystem {
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Update game time
        resources.clock.elapsed_time += resources.clock.tick_rate;
        resources.clock.current_time = std::time::SystemTime::now();
        
        SystemResult::Continue
    }
}

/// Dungeon system manages dungeon generation and level management
pub struct DungeonSystem;

impl System for DungeonSystem {
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Handle dungeon generation, level transitions, and level-specific events
        
        // Check if we need to generate a new level
        if resources.game_state.depth >= resources.config.max_depth {
            resources.game_state.game_state = GameStatus::Victory;
        }
        
        SystemResult::Continue
    }
}

/// Helper function to find the player entity
fn find_player(world: &World) -> Option<Entity> {
    for (entity, _) in world.query::<&Actor>().iter() {
        return Some(entity);
    }
    None
}

/// Helper function to check if a position is passable
fn is_position_passable(world: &World, x: i32, y: i32, z: i32) -> bool {
    // Find tile at position
    for (entity, (pos, tile)) in world.query::<(&Position, &Tile)>().iter() {
        if pos.x == x && pos.y == y && pos.z == z {
            return tile.is_passable;
        }
    }
    
    // If no tile found at position, assume it's not passable
    false
}

/// Helper function to find an entity at a specific position
fn find_entity_at_position(world: &World, x: i32, y: i32, z: i32) -> Option<Entity> {
    for (entity, pos) in world.query::<&Position>().iter() {
        if pos.x == x && pos.y == y && pos.z == z {
            return Some(entity);
        }
    }
    None
}

/// Helper function to calculate FOV from a position
fn calculate_fov(_world: &World, _pos: &Position, _range: u8) -> Vec<Position> {
    // This would implement an actual FOV algorithm (like ray casting or shadow casting)
    // For now, returning a simple set of positions for demonstration
    Vec::new()
}

/// Helper function to process an attack between two entities
fn process_attack(world: &mut World, attacker: Entity, defender: Entity) {
    if let (Ok(attacker_stats), Ok(mut defender_stats)) = 
        (world.get::<&Stats>(attacker), world.get::<&mut Stats>(defender)) {
        
        // Calculate damage based on attacker's stats and defender's defense
        let damage = std::cmp::max(1, attacker_stats.attack as i32 - defender_stats.defense as i32);
        
        // Apply damage
        defender_stats.hp = std::cmp::max(0, defender_stats.hp as i32 - damage) as u32;
        
        // Add message to game log
        // In a real implementation, we'd add this to the message log
        
        // Check if defender died
        if defender_stats.hp == 0 {
            // Handle defender death
            // For now, just log it
        }
    }
}

/// Helper function to use an item from inventory
fn use_item(world: &mut World, entity: Entity, index: usize) {
    if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
        if index < inventory.items.len() {
            if let Some(item_slot) = inventory.items.get_mut(index) {
                if let Some(item) = item_slot.item.clone() {
                    // Use the item based on its type
                    match item.item_type {
                        ItemType::Consumable { effect } => {
                            // Apply the consumable effect to the entity
                            apply_consumable_effect(world, entity, effect);
                            
                            // Reduce item count
                            if item_slot.quantity > 1 {
                                item_slot.quantity -= 1;
                            } else {
                                item_slot.item = None;
                            }
                        }
                        ItemType::Weapon { damage } => {
                            // Equip weapon (simplified)
                            if let Ok(mut stats) = world.get::<&mut Stats>(entity) {
                                stats.attack = damage;
                            }
                        }
                        ItemType::Armor { defense } => {
                            // Equip armor (simplified)
                            if let Ok(mut stats) = world.get::<&mut Stats>(entity) {
                                stats.defense = defense;
                            }
                        }
                        ItemType::Key => {
                            // Handle key usage
                        }
                        ItemType::Quest => {
                            // Handle quest item usage
                        }
                    }
                }
            }
        }
    }
}

/// Helper function to drop an item from inventory
fn drop_item(world: &mut World, entity: Entity, index: usize) {
    if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
        if index < inventory.items.len() {
            if let Some(item_slot) = inventory.items.get_mut(index) {
                if let Some(item) = item_slot.item.take() {
                    // In a real implementation, we would spawn an item entity at the entity's position
                    // For now, just remove the item
                    if item_slot.quantity > 1 {
                        item_slot.quantity -= 1;
                        item_slot.item = Some(item); // Put item back if there are more
                    }
                }
            }
        }
    }
}

/// Helper function to apply a consumable effect
fn apply_consumable_effect(world: &mut World, entity: Entity, effect: ConsumableEffect) {
    match effect {
        ConsumableEffect::Healing { amount } => {
            if let Ok(mut stats) = world.get::<&mut Stats>(entity) {
                stats.hp = std::cmp::min(stats.max_hp, stats.hp + amount);
            }
        }
        ConsumableEffect::Damage { amount } => {
            if let Ok(mut stats) = world.get::<&mut Stats>(entity) {
                stats.hp = stats.hp.saturating_sub(amount);
            }
        }
        ConsumableEffect::Buff { stat, value, duration } => {
            // Add the buff as an active effect
            if let Ok(mut effects) = world.get::<&mut Effects>(entity) {
                effects.active_effects.push(ActiveEffect {
                    effect_type: match stat {
                        StatType::Hp => EffectType::Healing,
                        StatType::Attack => EffectType::Healing, // This should map to a specific effect
                        StatType::Defense => EffectType::Healing, // This should map to a specific effect
                        StatType::Accuracy => EffectType::Healing, // This should map to a specific effect
                        StatType::Evasion => EffectType::Healing, // This should map to a specific effect
                    },
                    duration,
                    intensity: value as u32,
                });
            }
        }
        ConsumableEffect::Teleport => {
            // Handle teleportation
            // This would move the entity to a random valid position
        }
        ConsumableEffect::Identify => {
            // Handle item identification
            // This would identify items in the inventory
        }
    }
}

/// Helper function to apply an effect to an entity
fn apply_effect_to_entity(world: &mut World, entity: Entity, effect: &ActiveEffect) {
    // Apply the effect to the entity based on effect type
    match effect.effect_type {
        EffectType::Healing => {
            if let Ok(mut stats) = world.get::<&mut Stats>(entity) {
                stats.hp = std::cmp::min(stats.max_hp, stats.hp + effect.intensity);
            }
        }
        EffectType::Poison => {
            if let Ok(mut stats) = world.get::<&mut Stats>(entity) {
                stats.hp = stats.hp.saturating_sub(effect.intensity);
            }
        }
        EffectType::Burning => {
            if let Ok(mut stats) = world.get::<&mut Stats>(entity) {
                stats.hp = stats.hp.saturating_sub(effect.intensity);
            }
        }
        EffectType::Paralysis => {
            // Paralysis would prevent entity from acting
            // This would be handled in the energy system
        }
        EffectType::Rooted => {
            // Rooted would prevent movement
            // This would be handled during movement processing
        }
        EffectType::Confusion => {
            // Confusion might cause erratic movement
        }
        EffectType::Invisibility => {
            // Invisibility might affect rendering or detection
        }
        EffectType::Levitation => {
            // Levitation might allow movement over certain terrain
        }
    }
}