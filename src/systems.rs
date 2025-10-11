use crate::ecs::{AI, Actor, Energy, Player, Position, Resources, Viewshed, 
                PlayerAction, Direction, Tile, TerrainType, Renderable, 
                Faction, Stats, Inventory, ItemSlot, ECSItem, ItemType, 
                ConsumableEffect, StatType, GameStatus, Color, ECSWorld};
use hecs::{Entity, World};
use std::error::Error;
use combat::Combatant;
use rand;

pub enum SystemResult {
    Continue,
    Stop,
    Error(String),
}

pub trait System: Send {
    fn name(&self) -> &str;
    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult;
    fn is_energy_system(&self) -> bool {
        false
    }
}

pub struct InputSystem;

impl System for InputSystem {
    fn name(&self) -> &str {
        "InputSystem"
    }

    fn run(&mut self, _world: &mut World, _resources: &mut Resources) -> SystemResult {
        SystemResult::Continue
    }
}

pub struct TimeSystem;

impl System for TimeSystem {
    fn name(&self) -> &str {
        "TimeSystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        resources.clock.turn_count = resources.clock.turn_count.saturating_add(1);
        for (_, energy) in world.query::<&mut Energy>().iter() {
            let regen = energy.regeneration_rate.max(1);
            energy.current = (energy.current + regen).min(energy.max);
        }
        SystemResult::Continue
    }
}

pub struct MovementSystem;

impl System for MovementSystem {
    fn name(&self) -> &str {
        "MovementSystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Process pending player actions for movement
        let mut actions_to_process = std::mem::take(&mut resources.input_buffer.pending_actions);
        let mut new_actions = Vec::new();
        
        for action in actions_to_process {
            match action {
                PlayerAction::Move(direction) => {
                    if let Some(player_entity) = find_player_entity(world) {
                        // Get player's current position
                        let current_pos = match world.get::<&Position>(player_entity) {
                            Ok(pos) => pos.clone(),
                            Err(_) => {
                                // Player has no position, add action back to queue and continue
                                new_actions.push(action);
                                continue;
                            }
                        };

                        // Calculate new position based on direction
                        let new_pos = match direction {
                            Direction::North => Position::new(current_pos.x, current_pos.y - 1, current_pos.z),
                            Direction::South => Position::new(current_pos.x, current_pos.y + 1, current_pos.z),
                            Direction::East => Position::new(current_pos.x + 1, current_pos.y, current_pos.z),
                            Direction::West => Position::new(current_pos.x - 1, current_pos.y, current_pos.z),
                            Direction::NorthEast => Position::new(current_pos.x + 1, current_pos.y - 1, current_pos.z),
                            Direction::NorthWest => Position::new(current_pos.x - 1, current_pos.y - 1, current_pos.z),
                            Direction::SouthEast => Position::new(current_pos.x + 1, current_pos.y + 1, current_pos.z),
                            Direction::SouthWest => Position::new(current_pos.x - 1, current_pos.y + 1, current_pos.z),
                        };

                        // Check if the new position is passable (tile allows movement)
                        let can_move = Self::can_move_to(world, &new_pos);
                        
                        if can_move {
                            // Update player's position
                            if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
                                *pos = new_pos;
                            }
                            
                            // Mark that player has taken an action that costs energy
                            if let Ok(mut energy) = world.get::<&mut Energy>(player_entity) {
                                energy.current = energy.current.saturating_sub(100); // Cost for movement
                            }
                        } else {
                            // If can't move, add action back for later processing
                            new_actions.push(action);
                        }
                    } else {
                        // No player found, add action back
                        new_actions.push(action);
                    }
                }
                // For non-movement actions, add back to queue for other systems to handle
                _ => {
                    new_actions.push(action);
                }
            }
        }
        
        // Put unprocessed actions back in the buffer
        resources.input_buffer.pending_actions = new_actions;
        
        SystemResult::Continue
    }
}

impl MovementSystem {
    /// Check if an entity can move to the target position
    fn can_move_to(world: &World, target_pos: &Position) -> bool {
        // Check if there's a tile at the target position and if it's passable
        let mut passable = false;
        for (_, (pos, tile)) in world.query::<(&Position, &Tile)>().iter() {
            if pos.x == target_pos.x && pos.y == target_pos.y && pos.z == target_pos.z {
                if tile.is_passable {
                    passable = true;
                } else {
                    // Found a tile but it's not passable
                    return false;
                }
                break; // Found the tile, exit the loop
            }
        }
        
        // If no tile is found at the position, we assume it's not passable
        passable
    }
}

/// Helper function to find the player entity
fn find_player_entity(world: &World) -> Option<Entity> {
    for (entity, _) in world.query::<&Player>().iter() {
        return Some(entity);
    }
    None
}

pub struct AISystem;

impl System for AISystem {
    fn name(&self) -> &str {
        "AISystem"
    }

    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        let mut entities = Vec::new();
        for (entity, (ai, pos)) in world.query::<(&AI, &Position)>().iter() {
            entities.push((entity, ai.clone(), pos.clone()));
        }

        for (entity, ai, pos) in entities {
            if world.get::<&Player>(entity).is_ok() {
                continue;
            }

            if let crate::ecs::AIType::Aggressive = ai.ai_type {
                let mut closest_player = None;
                for (player_entity, (player_pos, _)) in world.query::<(&Position, &Player)>().iter() {
                    let distance = pos.distance_to(player_pos);
                    if distance <= ai.range() as f32 {
                        let update = closest_player.map_or(true, |(_, d)| distance < d);
                        if update {
                            closest_player = Some((player_entity, distance));
                        }
                    }
                }

                if let Some((player_entity, _)) = closest_player {
                    let entity_pos = match world.get::<&Position>(entity) {
                        Ok(pos) => Position::new(pos.x, pos.y, pos.z),
                        Err(_) => continue,
                    };
                    let player_pos = match world.get::<&Position>(player_entity) {
                        Ok(pos) => Position::new(pos.x, pos.y, pos.z),
                        Err(_) => continue,
                    };

                    let dx = (player_pos.x - entity_pos.x).signum();
                    let dy = (player_pos.y - entity_pos.y).signum();
                    let _ = Self::attempt_move_to(world, entity, entity_pos.x + dx, entity_pos.y + dy);
                }
            }
        }

        SystemResult::Continue
    }
}

impl AISystem {
    fn attempt_move_to(world: &mut World, entity: Entity, new_x: i32, new_y: i32) -> Result<(), Box<dyn Error>> {
        if let Ok(mut pos) = world.get::<&mut Position>(entity) {
            pos.x = new_x;
            pos.y = new_y;
            Ok(())
        } else {
            Err("missing position".into())
        }
    }
}

pub struct CombatSystem;

impl System for CombatSystem {
    fn name(&self) -> &str {
        "CombatSystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Process pending player actions for combat
        let mut actions_to_process = std::mem::take(&mut resources.input_buffer.pending_actions);
        let mut new_actions = Vec::new();
        
        for action in actions_to_process {
            match action {
                PlayerAction::Attack(ref target_pos) => {
                    if let Some(player_entity) = find_player_entity(world) {
                        let player_pos = match world.get::<&Position>(player_entity) {
                            Ok(pos) => Position::new(pos.x, pos.y, pos.z),
                            Err(_) => {
                                new_actions.push(action);
                                continue;
                            }
                        };
                        
                        // Calculate the position of the attack target
                        let attack_pos = Position::new(
                            player_pos.x + target_pos.x,
                            player_pos.y + target_pos.y,
                            player_pos.z
                        );
                        
                        // Find an enemy at the target position
                        let mut target_entity = None;
                        for (entity, (pos, actor)) in world.query::<(&Position, &Actor)>().iter() {
                            if actor.faction == Faction::Enemy && 
                               pos.x == attack_pos.x && 
                               pos.y == attack_pos.y && 
                               pos.z == attack_pos.z {
                                target_entity = Some(entity);
                                break;
                            }
                        }
                        
                        if let Some(target) = target_entity {
                            // Perform combat between player and target
                            // Collect entities to despawn after combat is resolved
                            let mut entities_to_despawn = Vec::new();
                            
                            // We need to temporarily update stats during combat
                            if world.get::<&Stats>(player_entity).is_ok() && world.get::<&Stats>(target).is_ok() {
                                // Extract the damage to be applied outside the borrow scope
                                let damage_result = {
                                    // Get mutable references to stats for combat calculation
                                    let (mut player_stats, mut target_stats) = 
                                        (world.get::<&mut Stats>(player_entity), world.get::<&mut Stats>(target));
                                    
                                    if let (Ok(mut p_stats), Ok(mut t_stats)) = (player_stats, target_stats) {
                                        // Create temporary stats for combat simulation
                                        let mut temp_player_stats = p_stats.clone();
                                        let mut temp_target_stats = t_stats.clone();
                                        
                                        let mut attacker = SimpleCombatant::new(&mut temp_player_stats);
                                        let mut defender = SimpleCombatant::new(&mut temp_target_stats);
                                        
                                        // Perform the attack using the combat module
                                        let combat_result = ::combat::Combat::engage(&mut attacker, &mut defender, false);
                                        Some((temp_player_stats.hp, temp_target_stats.hp, combat_result))
                                    } else {
                                        None
                                    }
                                };
                                
                                if let Some((new_player_hp, new_target_hp, combat_result)) = damage_result {
                                    // Now apply the actual damage
                                    if let Ok(mut actual_player_stats) = world.get::<&mut Stats>(player_entity) {
                                        actual_player_stats.hp = new_player_hp;
                                    }
                                    if let Ok(mut actual_target_stats) = world.get::<&mut Stats>(target) {
                                        actual_target_stats.hp = new_target_hp;
                                    }
                                    
                                    // Log combat results
                                    for log in &combat_result.logs {
                                        resources.game_state.message_log.push(log.clone());
                                        if resources.game_state.message_log.len() > 10 {
                                            resources.game_state.message_log.remove(0);
                                        }
                                    }
                                    
                                    // Check if defender was defeated
                                    if combat_result.defeated {
                                        // Mark entity for despawn later
                                        entities_to_despawn.push(target);
                                        resources.game_state.message_log.push("Enemy was defeated!".to_string());
                                    }
                                }
                                
                                // Spend energy for attack
                                if let Ok(mut energy) = world.get::<&mut Energy>(player_entity) {
                                    energy.current = energy.current.saturating_sub(100); // Cost for attack
                                }
                            }
                            
                            // Now despawn defeated entities outside the combat borrow scope
                            for entity in entities_to_despawn {
                                let _ = world.despawn(entity);
                            }
                        } else {
                            // No target at this position, add action back
                            resources.game_state.message_log.push("No enemy at this position!".to_string());
                            new_actions.push(action);
                        }
                    } else {
                        // No player found, add action back
                        new_actions.push(action);
                    }
                }
                // For non-combat actions, add back to queue for other systems to handle
                _ => {
                    new_actions.push(action);
                }
            }
        }
        
        // Put unprocessed actions back in the buffer
        resources.input_buffer.pending_actions = new_actions;
        
        SystemResult::Continue
    }
}

// Helper struct to implement the Combatant trait for ECS entities
struct SimpleCombatant<'a> {
    stats: &'a mut Stats,
    name: String,
    weapon: Option<crate::ecs::ECSItem>,
}

impl<'a> SimpleCombatant<'a> {
    fn new(stats: &'a mut Stats) -> Self {
        Self {
            stats,
            name: "Entity".to_string(),
            weapon: None,
        }
    }
}

impl<'a> ::combat::Combatant for SimpleCombatant<'a> {
    fn hp(&self) -> u32 {
        self.stats.hp
    }
    
    fn max_hp(&self) -> u32 {
        self.stats.max_hp
    }
    
    fn attack_power(&self) -> u32 {
        self.stats.attack
    }
    
    fn defense(&self) -> u32 {
        self.stats.defense
    }
    
    fn accuracy(&self) -> u32 {
        self.stats.accuracy
    }
    
    fn evasion(&self) -> u32 {
        self.stats.evasion
    }
    
    fn crit_bonus(&self) -> f32 {
        0.0
    }
    
    fn weapon(&self) -> Option<&::items::Weapon> {
        None
    }
    
    fn is_alive(&self) -> bool {
        self.stats.hp > 0
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn attack_distance(&self) -> u32 {
        1
    }
    
    fn take_damage(&mut self, amount: u32) -> bool {
        self.stats.hp = self.stats.hp.saturating_sub(amount);
        self.stats.hp > 0
    }
    
    fn heal(&mut self, amount: u32) {
        self.stats.hp = (self.stats.hp + amount).min(self.stats.max_hp);
    }
    
    fn exp_value(&self) -> u32 {
        10
    }
}

pub struct FOVSystem;

impl System for FOVSystem {
    fn name(&self) -> &str {
        "FOVSystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Check for game over conditions (player death)
        for (entity, (actor, stats)) in world.query::<(&Actor, &Stats)>().iter() {
            if actor.faction == Faction::Player && stats.hp == 0 {
                resources.game_state.game_state = GameStatus::GameOver;
                resources.game_state.message_log.push("You have died... Game Over!".to_string());
                return SystemResult::Stop; // End the game
            }
        }
        
        // Check for victory conditions (e.g., reaching max depth)
        if resources.game_state.depth >= resources.config.max_depth {
            // Check if player is on the final level and in a winning condition
            // For now, if the player reaches the max depth, they win
            for (entity, (actor, pos)) in world.query::<(&Actor, &Position)>().iter() {
                if actor.faction == Faction::Player && pos.z as usize == resources.config.max_depth {
                    resources.game_state.game_state = GameStatus::Victory;
                    resources.game_state.message_log.push("Congratulations! You won the game!".to_string());
                    return SystemResult::Stop; // End the game
                }
            }
        }

        // Update FOV for entities
        let entities: Vec<Entity> = world.query::<&Viewshed>().iter().map(|(entity, _)| entity).collect();
        for entity in entities {
            Self::update_fov(world, entity);
        }
        SystemResult::Continue
    }
}

impl FOVSystem {
    pub fn update_fov(world: &mut World, entity: Entity) {
        if let (Ok(pos), Ok(mut viewshed)) = (world.get::<&Position>(entity), world.get::<&mut Viewshed>(entity)) {
            viewshed.visible_tiles.clear();
            for dx in -(viewshed.range as i32)..=(viewshed.range as i32) {
                for dy in -(viewshed.range as i32)..=(viewshed.range as i32) {
                    let distance = ((dx * dx + dy * dy) as f32).sqrt();
                    if distance <= viewshed.range as f32 {
                        viewshed.visible_tiles.push(Position::new(pos.x + dx, pos.y + dy, pos.z));
                    }
                }
            }
            viewshed.dirty = false;
        }
    }
}

pub struct EffectSystem;

impl System for EffectSystem {
    fn name(&self) -> &str {
        "EffectSystem"
    }

    fn run(&mut self, _world: &mut World, _resources: &mut Resources) -> SystemResult {
        SystemResult::Continue
    }
}

pub struct EnergySystem;

impl System for EnergySystem {
    fn name(&self) -> &str {
        "EnergySystem"
    }

    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        for (_, energy) in world.query::<&mut Energy>().iter() {
            energy.current = energy.max;
        }
        SystemResult::Continue
    }

    fn is_energy_system(&self) -> bool {
        true
    }
}

pub struct InventorySystem;

impl System for InventorySystem {
    fn name(&self) -> &str {
        "InventorySystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Process pending player actions for inventory management
        let mut actions_to_process = std::mem::take(&mut resources.input_buffer.pending_actions);
        let mut new_actions = Vec::new();
        
        for action in actions_to_process {
            match action {
                PlayerAction::UseItem(slot_index) => {
                    if let Some(player_entity) = find_player_entity(world) {
                        // Get player's inventory
                        if let Ok(mut inventory) = world.get::<&mut Inventory>(player_entity) {
                            if slot_index < inventory.items.len() {
                                if let Some(ref item) = inventory.items[slot_index].item {
                                    // Check if this is a consumable item
                                    match &item.item_type {
                                        ItemType::Consumable { effect } => {
                                            match effect {
                                                ConsumableEffect::Healing { amount } => {
                                                    // Apply healing to player
                                                    if let Ok(mut stats) = world.get::<&mut Stats>(player_entity) {
                                                        stats.hp = (stats.hp + amount).min(stats.max_hp);
                                                        resources.game_state.message_log.push(
                                                            format!("You drink a {}, healing {} HP.", item.name, amount)
                                                        );
                                                        if resources.game_state.message_log.len() > 10 {
                                                            resources.game_state.message_log.remove(0);
                                                        }
                                                    }
                                                }
                                                ConsumableEffect::Damage { amount } => {
                                                    // Apply damage to player (negative effect)
                                                    if let Ok(mut stats) = world.get::<&mut Stats>(player_entity) {
                                                        stats.hp = stats.hp.saturating_sub(*amount);
                                                        resources.game_state.message_log.push(
                                                            format!("You drink a {}, taking {} damage!", item.name, amount)
                                                        );
                                                        if resources.game_state.message_log.len() > 10 {
                                                            resources.game_state.message_log.remove(0);
                                                        }
                                                    }
                                                }
                                                ConsumableEffect::Buff { stat, value, duration: _ } => {
                                                    // Apply stat buff to player
                                                    if let Ok(mut stats) = world.get::<&mut Stats>(player_entity) {
                                                        match stat {
                                                            StatType::Hp => stats.max_hp = (stats.max_hp as i32 + value) as u32,
                                                            StatType::Attack => stats.attack = (stats.attack as i32 + value) as u32,
                                                            StatType::Defense => stats.defense = (stats.defense as i32 + value) as u32,
                                                            StatType::Accuracy => stats.accuracy = (stats.accuracy as i32 + value) as u32,
                                                            StatType::Evasion => stats.evasion = (stats.evasion as i32 + value) as u32,
                                                        }
                                                        resources.game_state.message_log.push(
                                                            format!("You feel {}!", match stat {
                                                                StatType::Hp => format!("healthier ({})", value),
                                                                StatType::Attack => format!("stronger ({})", value),
                                                                StatType::Defense => format!("tougher ({})", value),
                                                                StatType::Accuracy => format!("more accurate ({})", value),
                                                                StatType::Evasion => format!("more evasive ({})", value),
                                                            })
                                                        );
                                                        if resources.game_state.message_log.len() > 10 {
                                                            resources.game_state.message_log.remove(0);
                                                        }
                                                    }
                                                }
                                                ConsumableEffect::Teleport => {
                                                    // Teleport player to random location in level
                                                    if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
                                                        pos.x = 5 + (resources.rng % 15) as i32; // Random position between 5-19
                                                        pos.y = 5 + ((resources.rng / 100) % 15) as i32; // Random position between 5-19
                                                        resources.rng = resources.rng.wrapping_add(12345); // Update RNG
                                                        resources.game_state.message_log.push("You teleport randomly!".to_string());
                                                        if resources.game_state.message_log.len() > 10 {
                                                            resources.game_state.message_log.remove(0);
                                                        }
                                                    }
                                                }
                                                ConsumableEffect::Identify => {
                                                    // For now, just add a message
                                                    resources.game_state.message_log.push("You feel more perceptive.".to_string());
                                                    if resources.game_state.message_log.len() > 10 {
                                                        resources.game_state.message_log.remove(0);
                                                    }
                                                }
                                            }
                                            
                                            // Remove the consumed item from inventory
                                            inventory.items.remove(slot_index);
                                        }
                                        _ => {
                                            resources.game_state.message_log.push("Cannot use this item.".to_string());
                                            if resources.game_state.message_log.len() > 10 {
                                                resources.game_state.message_log.remove(0);
                                            }
                                        }
                                    }
                                } else {
                                    resources.game_state.message_log.push("No item in this slot.".to_string());
                                    if resources.game_state.message_log.len() > 10 {
                                        resources.game_state.message_log.remove(0);
                                    }
                                }
                            } else {
                                resources.game_state.message_log.push("Invalid inventory slot.".to_string());
                                if resources.game_state.message_log.len() > 10 {
                                    resources.game_state.message_log.remove(0);
                                }
                                new_actions.push(action);
                            }
                        } else {
                            new_actions.push(action);
                        }
                    } else {
                        new_actions.push(action);
                    }
                }
                PlayerAction::DropItem(slot_index) => {
                    // Extract item data first to avoid borrow conflicts
                    let drop_result: Option<(Position, ECSItem)> = if let Some(player_entity) = find_player_entity(world) {
                        // Get the player's position and item to drop (in separate operations)
                        let player_pos = match world.get::<&Position>(player_entity) {
                            Ok(pos) => Position::new(pos.x, pos.y, pos.z),
                            Err(_) => {
                                new_actions.push(action);
                                continue;
                            }
                        };
                        
                        // Get and remove the item
                        if let Ok(mut inventory) = world.get::<&mut Inventory>(player_entity) {
                            if slot_index < inventory.items.len() {
                                if let Some(item_to_drop) = inventory.items.remove(slot_index).item {
                                    Some((player_pos.clone(), item_to_drop))  // Clone the position to get owned value
                                } else {
                                    resources.game_state.message_log.push("No item in this slot to drop.".to_string());
                                    if resources.game_state.message_log.len() > 10 {
                                        resources.game_state.message_log.remove(0);
                                    }
                                    new_actions.push(action);
                                    None
                                }
                            } else {
                                resources.game_state.message_log.push("Invalid inventory slot.".to_string());
                                if resources.game_state.message_log.len() > 10 {
                                    resources.game_state.message_log.remove(0);
                                }
                                new_actions.push(action);
                                None
                            }
                        } else {
                            new_actions.push(action);
                            None
                        }
                    } else {
                        new_actions.push(action);
                        None
                    };
                    
                    // Now spawn the item if we have the data
                    if let Some((player_pos, item_to_drop)) = drop_result {
                        world.spawn((
                            Position::new(player_pos.x, player_pos.y, player_pos.z),
                            Renderable {
                                symbol: item_to_drop.name.chars().next().unwrap_or('?'),
                                fg_color: Color::Yellow,
                                bg_color: Some(Color::Black),
                                order: 1,
                            },
                            ECSItem {
                                name: item_to_drop.name.clone(),
                                item_type: item_to_drop.item_type.clone(),
                                value: item_to_drop.value,
                                identified: item_to_drop.identified,
                            },
                            Tile {
                                terrain_type: TerrainType::Empty,
                                is_passable: true,
                                blocks_sight: false,
                                has_items: true,
                                has_monster: false,
                            },
                        ));
                        
                        resources.game_state.message_log.push(format!("You dropped {}.", item_to_drop.name));
                        if resources.game_state.message_log.len() > 10 {
                            resources.game_state.message_log.remove(0);
                        }
                    }
                }
                // For non-inventory actions, add back to queue for other systems to handle
                _ => {
                    new_actions.push(action);
                }
            }
        }
        
        // Process item pickup
        {
            // Collect players and items first to resolve borrowing conflicts
            let pickup_actions: Vec<_> = {
                let mut actions = Vec::new();
                for (player_entity, (player_pos, _actor)) in 
                    world.query::<(&Position, &Actor)>().iter() {
                    
                    if world.get::<&Player>(player_entity).is_err() {
                        continue;
                    }
                    
                    let mut items_for_player = Vec::new();
                    for (item_entity, (pos, item)) in world.query::<(&Position, &ECSItem)>().iter() {
                        if pos.x == player_pos.x && pos.y == player_pos.y && pos.z == player_pos.z {
                            items_for_player.push((item_entity, item.clone(), item.name.clone()));
                        }
                    }
                    
                    if let Ok(inventory) = world.get::<&Inventory>(player_entity) {
                        for (item_entity, item_clone, item_name) in items_for_player {
                            if inventory.items.len() < inventory.max_slots {
                                actions.push((player_entity, item_entity, item_clone, item_name));
                            } else {
                                resources.game_state.message_log.push("Your inventory is full!".to_string());
                                if resources.game_state.message_log.len() > 10 {
                                    resources.game_state.message_log.remove(0);
                                }
                                break;
                            }
                        }
                    }
                }
                actions
            };
            
            for (player_entity, item_entity, item, item_name) in pickup_actions {
                if let Ok(mut inventory) = world.get::<&mut Inventory>(player_entity) {
                    if inventory.items.len() < inventory.max_slots {
                        inventory.items.push(ItemSlot {
                            item: Some(item),
                            quantity: 1,
                        });
                        let _ = world.despawn(item_entity);
                        resources.game_state.message_log.push(format!("You picked up {}.", item_name));
                        if resources.game_state.message_log.len() > 10 {
                            resources.game_state.message_log.remove(0);
                        }
                    } else {
                        resources.game_state.message_log.push("Your inventory is full!".to_string());
                        if resources.game_state.message_log.len() > 10 {
                            resources.game_state.message_log.remove(0);
                        }
                    }
                }
            }
        }
        
        // Put unprocessed actions back in the buffer
        resources.input_buffer.pending_actions = new_actions;
        
        SystemResult::Continue
    }
}



pub struct DungeonSystem;

impl System for DungeonSystem {
    fn name(&self) -> &str {
        "DungeonSystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Process pending player actions for dungeon navigation
        let mut actions_to_process = std::mem::take(&mut resources.input_buffer.pending_actions);
        let mut new_actions = Vec::new();
        
        for action in actions_to_process {
            match action {
                PlayerAction::Descend => {
                    if let Some(player_entity) = find_player_entity(world) {
                        // Check if player is on stairs - get the position first
                        let player_pos_opt = match world.get::<&Position>(player_entity) {
                            Ok(pos) => Some(pos.clone()),
                            Err(_) => None,
                        };
                        
                        if let Some(player_pos) = player_pos_opt {
                            // Check if there's a stairs down tile at player's position
                            let mut on_stairs_down = false;
                            for (_, (pos, tile)) in world.query::<(&Position, &Tile)>().iter() {
                                if pos.x == player_pos.x && pos.y == player_pos.y && pos.z == player_pos.z {
                                    if matches!(tile.terrain_type, TerrainType::StairsDown) {
                                        on_stairs_down = true;
                                        break;
                                    }
                                }
                            }
                            
                            if on_stairs_down {
                                // Queue up level generation and player movement
                                resources.game_state.message_log.push("You descend to the next level...".to_string());
                                if resources.game_state.message_log.len() > 10 {
                                    resources.game_state.message_log.remove(0);
                                }
                                
                                resources.game_state.depth = (player_pos.z + 1) as usize;
                                
                                // Move player to new level
                                if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
                                    pos.z += 1;
                                    // Place player at stairs up position
                                    // For now, we'll place them at a default position (10, 10) on the new level
                                    pos.x = 10;
                                    pos.y = 10;
                                }
                                
                                // Add generation of new level after all actions are processed
                                // We'll generate it in a separate pass
                            } else {
                                resources.game_state.message_log.push("You need to stand on stairs to descend.".to_string());
                                if resources.game_state.message_log.len() > 10 {
                                    resources.game_state.message_log.remove(0);
                                }
                                new_actions.push(action);
                            }
                        } else {
                            new_actions.push(action);
                        }
                    } else {
                        new_actions.push(action);
                    }
                }
                PlayerAction::Ascend => {
                    if let Some(player_entity) = find_player_entity(world) {
                        // Check if player is on stairs - get the position first
                        let player_pos_opt = match world.get::<&Position>(player_entity) {
                            Ok(pos) => Some(pos.clone()),
                            Err(_) => None,
                        };
                        
                        if let Some(player_pos) = player_pos_opt {
                            // Check if there's a stairs up tile at player's position
                            let mut on_stairs_up = false;
                            for (_, (pos, tile)) in world.query::<(&Position, &Tile)>().iter() {
                                if pos.x == player_pos.x && pos.y == player_pos.y && pos.z == player_pos.z {
                                    if matches!(tile.terrain_type, TerrainType::StairsUp) {
                                        on_stairs_up = true;
                                        break;
                                    }
                                }
                            }
                            
                            if on_stairs_up {
                                if player_pos.z > 0 {
                                    // Move player to new level
                                    if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
                                        pos.z -= 1;
                                        // Place player at stairs down position at previous level
                                        pos.x = 10;
                                        pos.y = 10;
                                    }
                                    
                                    resources.game_state.message_log.push("You ascend to the previous level...".to_string());
                                    if resources.game_state.message_log.len() > 10 {
                                        resources.game_state.message_log.remove(0);
                                    }
                                    
                                    resources.game_state.depth = (player_pos.z - 1) as usize;
                                    
                                    // Generate level for the new depth after actions are processed
                                } else {
                                    // Player is at dungeon level 0, can't go higher
                                    resources.game_state.message_log.push("You can't go up from here.".to_string());
                                    if resources.game_state.message_log.len() > 10 {
                                        resources.game_state.message_log.remove(0);
                                    }
                                }
                            } else {
                                resources.game_state.message_log.push("You need to stand on stairs to ascend.".to_string());
                                if resources.game_state.message_log.len() > 10 {
                                    resources.game_state.message_log.remove(0);
                                }
                                new_actions.push(action);
                            }
                        } else {
                            new_actions.push(action);
                        }
                    } else {
                        new_actions.push(action);
                    }
                }
                // For non-dungeon actions, add back to queue for other systems to handle
                _ => {
                    new_actions.push(action);
                }
            }
        }
        
        // Put unprocessed actions back in the buffer
        resources.input_buffer.pending_actions = new_actions;
        
        SystemResult::Continue
    }
}

impl DungeonSystem {
    /// Generate a basic dungeon level
    fn generate_level(&mut self, world: &mut World, resources: &mut Resources, level: i32) {
        // Remove all tiles for the level being generated
        let tiles_to_remove: Vec<_> = world
            .query::<(&Position, &Tile)>()
            .iter()
            .filter(|(_, (pos, _))| pos.z == level)
            .map(|(e, _)| e)
            .collect();
        
        for entity in tiles_to_remove {
            let _ = world.despawn(entity);
        }
        
        // Generate a basic 20x20 room layout for the level
        for x in 5..25 {
            for y in 5..25 {
                let terrain_type = if x == 5 || x == 24 || y == 5 || y == 24 {
                    TerrainType::Wall
                } else {
                    TerrainType::Floor
                };
                
                let renderable = Renderable {
                    symbol: if x == 5 || x == 24 || y == 5 || y == 24 { '#' } else { '.' },
                    fg_color: if x == 5 || x == 24 || y == 5 || y == 24 { Color::Gray } else { Color::White },
                    bg_color: Some(Color::Black),
                    order: 0,
                };
                
                world.spawn((
                    Position::new(x, y, level),
                    Tile {
                        terrain_type,
                        is_passable: x != 5 && x != 24 && y != 5 && y != 24,
                        blocks_sight: x == 5 || x == 24 || y == 5 || y == 24,
                        has_items: false,
                        has_monster: false,
                    },
                    renderable,
                ));
            }
        }
        
        // Place stairs based on current level for connections
        if level > 0 {
            // Place stairs up (going down to the previous level)
            world.spawn((
                Position::new(9, 9, level),
                Tile {
                    terrain_type: TerrainType::StairsUp,
                    is_passable: true,
                    blocks_sight: false,
                    has_items: false,
                    has_monster: false,
                },
                Renderable {
                    symbol: '<',
                    fg_color: Color::Cyan,
                    bg_color: Some(Color::Black),
                    order: 1,
                },
            ));
        }
        
        // Place stairs down if not the deepest level
        if level < (resources.config.max_depth as i32 - 1) {
            world.spawn((
                Position::new(15, 15, level),
                Tile {
                    terrain_type: TerrainType::StairsDown,
                    is_passable: true,
                    blocks_sight: false,
                    has_items: false,
                    has_monster: false,
                },
                Renderable {
                    symbol: '>',
                    fg_color: Color::Cyan,
                    bg_color: Some(Color::Black),
                    order: 1,
                },
            ));
        }
        
        // Add some simple monsters and items to the level
        if level > 0 {  // Add content to levels other than 0
            // Add a simple enemy
            let enemy_pos = Position::new(12, 12, level);
            world.spawn((
                enemy_pos,
                Actor {
                    name: format!("Goblin {}", level),
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
                    attack: 5 + (level as u32 * 2),
                    defense: 2 + (level as u32),
                    accuracy: 70,
                    evasion: 10,
                    level: level as u32,
                    experience: 10 + (level as u32 * 5),
                },
                Energy {
                    current: 100,
                    max: 100,
                    regeneration_rate: 1,
                },
            ));
            
            // Add a healing potion
            world.spawn((
                Position::new(14, 10, level),
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
        }
    }
}

pub struct RenderingSystem;

impl System for RenderingSystem {
    fn name(&self) -> &str {
        "RenderingSystem"
    }

    fn run(&mut self, _world: &mut World, _resources: &mut Resources) -> SystemResult {
        SystemResult::Continue
    }
}

pub struct InteractionSystem;

impl InteractionSystem {
    pub fn handle_interactions(world: &mut World) {
        for (_, (pos, actor)) in world.query::<(&Position, &Actor)>().iter() {
            if actor.faction == crate::ecs::Faction::Player {
                let _ = (pos.x, pos.y);
            }
        }
    }
}

impl System for InteractionSystem {
    fn name(&self) -> &str {
        "InteractionSystem"
    }

    fn run(&mut self, world: &mut World, _resources: &mut Resources) -> SystemResult {
        Self::handle_interactions(world);
        SystemResult::Continue
    }
}
