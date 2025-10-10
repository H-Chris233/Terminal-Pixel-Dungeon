//! ECS Systems for game logic processing.

use crate::ecs::*;
use anyhow;
use hecs::{Entity, World};
use dungeon::{Dungeon as DungeonModule, TileInfo};
use dungeon::InteractionEvent;
use combat::{Combat, Combatant};
use items;

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

/// Input system that bridges to the UI input module
pub struct InputSystemBridge;

impl System for InputSystemBridge {
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // This would use the ui::input module for processing
        // For now, we'll continue using the existing input handling
        
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
                            // Handle the attack - first check positions without holding mutable reference
                            let should_attack = if let (Ok(player_pos_ref), Ok(target_pos_ref)) = (
                                world.get::<&Position>(player_entity),
                                world.get::<&Position>(target_entity)
                            ) {
                                let player_pos = (*player_pos_ref).clone();
                                let target_pos = (*target_pos_ref).clone();
                                let distance = player_pos.distance_to(&target_pos);
                                distance <= 1.5  // Adjacent
                            } else {
                                false
                            };
                            
                            if should_attack {
                                process_attack(world, player_entity, target_entity);
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
        // For now, we handle attacks in the movement system
        SystemResult::Continue
    }
}

/// Combat system that uses the combat module functionality
pub struct CombatSystemBridge;

impl System for CombatSystemBridge {
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Process pending attacks between entities
        // Process pending attacks by collecting them first to avoid borrow conflicts
        let attacks_to_process: Vec<_> = world
            .query::<(&Position, &Stats, &mut Energy, &Actor)>()
            .iter()
            .filter(|(_, (_, _, energy, _))| energy.current >= 10) // Only entities with enough energy
            .map(|(entity, (pos, stats, _, actor))| (entity, pos.clone(), stats.clone(), actor.clone()))
            .collect();

        for (attacker_entity, pos, stats, actor) in attacks_to_process {
            if let Ok(mut energy) = world.get::<&mut Energy>(attacker_entity) {
                energy.current = energy.current.saturating_sub(10); // Cost per attack
            }

            // Check for adjacent enemies to attack
            for (defender_entity, _) in world.query::<&Position>().iter() {
                if attacker_entity == defender_entity { continue; }

                if let (Ok(defender_pos), Ok(defender_actor)) = 
                    (world.get::<&Position>(defender_entity), 
                     world.get::<&Actor>(defender_entity)) {
                    let distance = pos.distance_to(&defender_pos);
                    
                    // Check if entities are adjacent and are on different factions (can attack)
                    if distance <= 1.5 && are_enemies(&actor, &defender_actor) { // Adjacent
                        // Perform combat using the combat module
                        if let (Ok(mut attacker_stats), Ok(mut defender_stats)) = 
                            (world.get::<&mut Stats>(attacker_entity), 
                             world.get::<&mut Stats>(defender_entity)) {
                            
                            // Create temporary Combatant implementations for the attack
                            let mut attacker_combatant = ECSCombatant::new(&mut attacker_stats, &actor);
                            let mut defender_combatant = ECSCombatant::new(&mut defender_stats, &defender_actor);
                            
                            // Determine if this is an ambush attack based on visibility
                            let is_ambush = {
                                // Check if there's a dungeon instance to get terrain info
                                if let Some(ref dungeon) = resources.dungeon_instance {
                                    let is_blocked = |x: i32, y: i32| -> bool {
                                        match dungeon.get_tile(x, y) {
                                            TileInfo { passable: false, .. } => true,  // Impassable tile blocks vision
                                            TileInfo { passable: true, .. } => false, // Passable tile allows vision
                                        }
                                    };
                                    
                                    // Calculate if the defender is visible to the attacker
                                    let fov_range = 8; // Default FOV range
                                    combat::vision::VisionSystem::can_ambush(
                                        &attacker_combatant,
                                        pos.x,
                                        pos.y,
                                        &defender_combatant,
                                        defender_pos.x,
                                        defender_pos.y,
                                        &is_blocked,
                                        fov_range,
                                    )
                                } else {
                                    false // If no dungeon, no ambush possible
                                }
                            };
                            
                            // Use the combat module for actual combat resolution
                            let combat_result = combat::Combat::engage(
                                &mut attacker_combatant,
                                &mut defender_combatant,
                                is_ambush
                            );
                            
                            // Process any combat messages
                            for message in combat_result.logs {
                                resources.game_state.message_log.push(message);
                            }
                            
                            // Check if defender died
                            if combat_result.defeated {
                                // In a real implementation, we might add death effects
                                resources.game_state.message_log.push(
                                    format!("{} defeated {}!", actor.name, defender_actor.name)
                                );
                            }
                        }
                    }
                }
            }
        }

        SystemResult::Continue
    }
}

/// Helper function to determine if two actors are enemies
fn are_enemies(attacker: &Actor, defender: &Actor) -> bool {
    match (&attacker.faction, &defender.faction) {
        (Faction::Player, Faction::Enemy) => true,
        (Faction::Enemy, Faction::Player) => true,
        _ => false, // For simplicity, only player vs enemy is considered an attack
    }
}

/// Temporary struct to implement Combatant for ECS Stats
struct ECSCombatant<'a> {
    stats: &'a mut Stats,
    actor: &'a Actor,
    // We'll add more fields if needed to properly implement Combatant
}

impl<'a> ECSCombatant<'a> {
    fn new(stats: &'a mut Stats, actor: &'a Actor) -> Self {
        Self { stats, actor }
    }
}

impl<'a> Combatant for ECSCombatant<'a> {
    fn hp(&self) -> u32 { self.stats.hp }
    fn max_hp(&self) -> u32 { self.stats.max_hp }
    fn attack_power(&self) -> u32 { self.stats.attack }
    fn defense(&self) -> u32 { self.stats.defense }
    fn accuracy(&self) -> u32 { self.stats.accuracy }
    fn evasion(&self) -> u32 { self.stats.evasion }
    fn crit_bonus(&self) -> f32 { 0.1 } // Default crit bonus
    fn weapon(&self) -> Option<&items::Weapon> { None } // Not implemented for ECS combatants yet
    fn is_alive(&self) -> bool { self.stats.hp > 0 }
    fn name(&self) -> &str { &self.actor.name }
    fn attack_distance(&self) -> u32 { 1 } // Default attack distance
    fn take_damage(&mut self, amount: u32) -> bool {
        self.stats.hp = self.stats.hp.saturating_sub(amount);
        self.is_alive()
    }
    fn heal(&mut self, amount: u32) {
        self.stats.hp = (self.stats.hp + amount).min(self.stats.max_hp);
    }
    fn strength(&self) -> u8 { 10 } // Default strength
    fn dexterity(&self) -> u8 { 10 } // Default dexterity
    fn intelligence(&self) -> u8 { 10 } // Default intelligence
}

/// Field of View (FOV) system calculates what areas are visible to actors
pub struct FOVSystem;

impl System for FOVSystem {
    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        // Update field of view for all entities with Viewshed component
        // First, collect all entities that need FOV updates and their data
        let entities_to_process: Vec<_> = world
            .query::<(&Viewshed, &Position)>()
            .iter()
            .filter(|(_, (viewshed, _))| viewshed.dirty)
            .map(|(entity, (viewshed, pos))| (entity, viewshed.range, pos.clone()))
            .collect();
        
        // Then process and update each entity
        for (entity, range, pos) in entities_to_process {
            if let Ok(mut viewshed) = world.get::<&mut Viewshed>(entity) {
                viewshed.visible_tiles = calculate_fov_simple(&pos, range);
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
        // First, get all necessary information without holding borrows
        let (player_entity_opt, player_pos_opt) = {
            let mut player_entity_opt = None;
            let mut player_pos_opt = None;
            
            if let Some(player_entity) = find_player(world) {
                player_entity_opt = Some(player_entity);
                // Get the position and extract the value to avoid borrow conflicts
                if let Ok(pos_ref) = world.get::<&Position>(player_entity) {
                    // Extract the value and drop the reference immediately
                    player_pos_opt = Some((*pos_ref).clone());  // Dereference to get Position, then clone
                }
            }
            (player_entity_opt, player_pos_opt)
        };
        
        // Collect all AI entities that need processing, along with their info
        let entities_to_process: Vec<_> = world
            .query::<(&AI, &Position, &Energy)>()
            .iter()
            .filter(|(_, (_, _, energy))| energy.current >= 10) // Only process if enough energy
            .map(|(entity, (ai, pos, _))| (entity, ai.clone(), pos.clone()))
            .collect();
        
        // Process AI for each entity and collect actions to perform later
        let mut entities_to_attack = Vec::new();
        let mut entity_moves = Vec::new(); // Store movement for later application
        
        for (entity, mut ai, mut pos) in entities_to_process {
            match &mut ai.ai_type {
                AIType::Aggressive => {
                    if let Some(ref target_pos) = player_pos_opt {
                        // Move towards player if in range (chasing)
                        let distance = pos.distance_to(target_pos);
                        
                        if distance <= 1.5 {
                            // Schedule attack for later to avoid borrow conflicts
                            if let Some(player_entity) = player_entity_opt {
                                entities_to_attack.push((entity, player_entity));
                            }
                        } else if distance <= ai.range() as f32 {
                            // Move towards player
                            let dx = (target_pos.x - pos.x).signum();
                            let dy = (target_pos.y - pos.y).signum();
                            
                            let new_x = pos.x + dx;
                            let new_y = pos.y + dy;
                            
                            if is_position_passable(world, new_x, new_y, pos.z) {
                                entity_moves.push((entity, new_x, new_y));
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
        }
        
        // Now update the positions for entities that need to move
        for (entity, new_x, new_y) in entity_moves {
            if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                pos.x = new_x;
                pos.y = new_y;
            }
        }
        
        // Update energy for all AI entities (all get -10 energy)
        for (entity, _) in world.query::<&AI>().iter() {
            if let Ok(mut energy) = world.get::<&mut Energy>(entity) {
                energy.current = energy.current.saturating_sub(10);
            }
        }
        
        // Finally, process all the scheduled attacks
        for (attacker, defender) in entities_to_attack {
            process_attack(world, attacker, defender);
        }
        
        SystemResult::Continue
    }
}

/// Effect system processes active effects on actors
pub struct EffectSystem;

impl System for EffectSystem {
    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        // Process active effects on all entities
        // First collect all entities with effects and their effect data to avoid borrow conflicts
        let entities_with_effects: Vec<_> = world
            .query::<&Effects>()
            .iter()
            .map(|(entity, effects)| {
                let effects_to_process: Vec<_> = effects.active_effects
                    .iter()
                    .cloned()
                    .collect();
                let entity_effects_to_expire: Vec<_> = effects.active_effects
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, effect)| {
                        if effect.duration > 0 && effect.duration == 1 {  // This will expire next tick
                            Some((entity, idx))
                        } else {
                            None
                        }
                    })
                    .collect();
                (entity, effects_to_process, entity_effects_to_expire)
            })
            .collect();
        
        // Apply all effects to their respective entities
        for (entity, effects_to_apply, _) in &entities_with_effects {
            for effect in effects_to_apply {
                apply_effect_to_entity(world, *entity, effect);
            }
        }
        
        // Update the effect components to reflect changes (like expiring timed effects)
        for (entity, _, effects_to_expire) in entities_with_effects {
            if let Ok(mut effects) = world.get::<&mut Effects>(entity) {
                for (entity_to_expire, idx_to_remove) in effects_to_expire.iter().rev() {
                    if *entity_to_expire == entity && *idx_to_remove < effects.active_effects.len() {
                        effects.active_effects.remove(*idx_to_remove);
                    }
                }
                
                // Decrement duration for timed effects
                for effect in &mut effects.active_effects {
                    if effect.duration > 0 {
                        effect.duration -= 1;
                    }
                }
            }
        }
        
        SystemResult::Continue
    }
}

/// Effect system that bridges to the combat effect module
pub struct EffectSystemBridge;

impl System for EffectSystemBridge {
    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        // This would integrate with the combat::effect module
        // For now, we just process the ECS effects
        
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

/// Inventory system that bridges to the items module
pub struct InventorySystemBridge;

impl System for InventorySystemBridge {
    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        // Process pending inventory operations
        // For example, equipping items, using consumables, etc.
        // In a real implementation, we'd process specific inventory commands
        
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

/// Rendering system that bridges to the UI rendering module
pub struct RenderingSystemBridge;

impl System for RenderingSystemBridge {
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // This would use the ui::render module for rendering
        // Query ECS for entities that need to be rendered
        // and pass the data to the appropriate UI renderers
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
        if resources.dungeon_instance.is_none() {
            // Initialize the dungeon if not already done
            match DungeonModule::generate(resources.config.max_depth, resources.rng) {
                Ok(dungeon) => {
                    resources.dungeon_instance = Some(dungeon);
                }
                Err(e) => {
                    eprintln!("Failed to initialize dungeon: {}", e);
                    return SystemResult::Error(e.to_string());
                }
            }
        }
        
        // Process dungeon interactions if we have a dungeon instance
        if let Some(dungeon) = &mut resources.dungeon_instance {
            // Update dungeon state based on player position
            if let Some(player_entity) = find_player(world) {
                if let Ok(player_pos) = world.get::<&Position>(player_entity) {
                    let player_x = player_pos.x;
                    let player_y = player_pos.y;
                    let player_z = player_pos.z as usize;
                    
                    // Update visibility based on player position
                    dungeon.update_visibility(player_x, player_y, resources.config.fov_range);
                    
                    // Process tile interactions when player moves
                    let interactions = dungeon.on_hero_enter(player_x, player_y);
                    for interaction in interactions {
                        match interaction {
                            InteractionEvent::ItemFound(item) => {
                                // Add item to player inventory
                                if let Ok(mut inventory) = world.get::<&mut Inventory>(player_entity) {
                                    // For simplicity, add to first available slot
                                    for slot in &mut inventory.items {
                                        if slot.item.is_none() {
                                            // Convert game_items::Item to ECSItem
                                            let ecs_item = ECSItem {
                                                name: item.name.clone(),
                                                item_type: ItemType::Consumable { 
                                                    effect: ConsumableEffect::Healing { amount: 20 } // Default conversion
                                                },
                                                value: item.value(),  // Assuming it has a value() method
                                                identified: !item.needs_identify(),
                                            };
                                            slot.item = Some(ecs_item);
                                            slot.quantity = 1;
                                            break;
                                        }
                                    }
                                }
                            }
                            InteractionEvent::EnemyEncounter(enemy) => {
                                // In a real implementation, this would trigger combat
                                // For now, we just log it
                                resources.game_state.message_log.push(format!("Encountered {}", enemy.name()));
                            }
                            InteractionEvent::StairsDown => {
                                // Handle descending stairs
                                if dungeon.can_descend(player_x, player_y) {
                                    if dungeon.descend().is_ok() {
                                        // Update player position to next level
                                        if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
                                            pos.z += 1;
                                        }
                                    }
                                }
                            }
                            InteractionEvent::StairsUp => {
                                // Handle ascending stairs
                                if dungeon.can_ascend(player_x, player_y) {
                                    if dungeon.ascend().is_ok() {
                                        // Update player position to previous level
                                        if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
                                            pos.z = std::cmp::max(0, pos.z - 1);
                                        }
                                    }
                                }
                            }
                            _ => {} // Other interactions
                        }
                    }
                }
            }
            
            // Check if we need to generate a new level
            if resources.game_state.depth >= resources.config.max_depth {
                resources.game_state.game_state = GameStatus::Victory;
            }
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

/// Simple FOV calculation that doesn't require world access
fn calculate_fov_simple(pos: &Position, range: u8) -> Vec<Position> {
    // This would implement an actual FOV algorithm (like ray casting or shadow casting)
    // For now, returning positions in a circle around the given position
    let mut visible_tiles = Vec::new();
    let range = range as i32;
    
    for dx in -range..=range {
        for dy in -range..=range {
            if dx * dx + dy * dy <= range * range {
                visible_tiles.push(Position::new(pos.x + dx, pos.y + dy, pos.z));
            }
        }
    }
    
    visible_tiles
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
    // First, check if we can use the item and get its type without mutably borrowing inventory
    let item_to_use = {
        if let Ok(inventory) = world.get::<&Inventory>(entity) {
            if index < inventory.items.len() {
                inventory.items.get(index)
                    .and_then(|item_slot| item_slot.item.clone())
            } else {
                None
            }
        } else {
            None
        }
    };
    
    // Process consumable items first to handle borrow conflicts
    if let Some(item) = item_to_use {
        if let ItemType::Consumable { effect } = item.item_type {
            // Apply the consumable effect to the entity first
            apply_consumable_effect(world, entity, effect);
            
            // Then update the inventory to reduce item count
            if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
                if index < inventory.items.len() {
                    if let Some(item_slot) = inventory.items.get_mut(index) {
                        if item_slot.quantity > 1 {
                            item_slot.quantity -= 1;
                        } else {
                            item_slot.item = None;
                        }
                    }
                }
            }
        } else {
            // Handle non-consumable items
            match item.item_type {
                ItemType::Weapon { damage } => {
                    // Update stats
                    if let Ok(mut stats) = world.get::<&mut Stats>(entity) {
                        stats.attack = damage;
                    }
                }
                ItemType::Armor { defense } => {
                    // Update stats
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
                ItemType::Consumable { .. } => {
                    // This case is already handled above
                }
            }
            
            // Update inventory for non-consumables if needed
            if let Ok(mut inventory) = world.get::<&mut Inventory>(entity) {
                if index < inventory.items.len() {
                    if let Some(item_slot) = inventory.items.get_mut(index) {
                        // Simply using the item might consume it depending on implementation
                        // For non-consumables, we might not consume them
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

/// System to sync ECS components with Hero module structures
pub struct HeroSyncSystem;

impl System for HeroSyncSystem {
    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        // In a complete implementation, this would sync between ECS components and Hero structs
        // For example, changes to Stats component would be reflected in Hero struct and vice versa
        
        // Find the player entity and sync its data
        if let Some(player_entity) = find_player(world) {
            // This could involve converting the ECS components to Hero struct when needed
            // and vice versa, depending on which system needs to be updated
        }
        
        SystemResult::Continue
    }
}