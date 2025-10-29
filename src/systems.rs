use crate::ecs::{
    AI, AIState, AIType, Actor, Color, ConsumableEffect, Direction, ECSItem, ECSWorld, EffectType,
    Energy, Faction, GameOverReason, GameStatus, Hunger, Inventory, ItemSlot, ItemType,
    NavigateDirection, Player, PlayerAction, PlayerProgress, Position, Renderable, Resources,
    StatType, Stats, StatusEffects, TerrainType, Tile, Viewshed, Wealth,
};
use crate::event_bus::LogLevel;
use hecs::{Entity, World};
use std::error::Error;

use rand;

pub enum SystemResult {
    Continue,
    Stop,
    Error(String),
}

/// A deterministic phase that participates in the turn pipeline.
///
/// Systems are executed in the order defined by `game_loop::GameLoop::systems`;
/// earlier systems have the opportunity to consume actions before later ones
/// inspect the buffers. When adding new systems, keep the documentation in
/// `docs/turn_system.md` up to date.
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

/// Resolves `PlayerAction::Move` entries and flags successful steps so the
/// turn scheduler can deduct energy exactly once.
pub struct MovementSystem;

impl System for MovementSystem {
    fn name(&self) -> &str {
        "MovementSystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // Process pending player actions for movement
        let actions_to_process = std::mem::take(&mut resources.input_buffer.pending_actions);
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
                            Direction::North => {
                                Position::new(current_pos.x, current_pos.y - 1, current_pos.z)
                            }
                            Direction::South => {
                                Position::new(current_pos.x, current_pos.y + 1, current_pos.z)
                            }
                            Direction::East => {
                                Position::new(current_pos.x + 1, current_pos.y, current_pos.z)
                            }
                            Direction::West => {
                                Position::new(current_pos.x - 1, current_pos.y, current_pos.z)
                            }
                            Direction::NorthEast => {
                                Position::new(current_pos.x + 1, current_pos.y - 1, current_pos.z)
                            }
                            Direction::NorthWest => {
                                Position::new(current_pos.x - 1, current_pos.y - 1, current_pos.z)
                            }
                            Direction::SouthEast => {
                                Position::new(current_pos.x + 1, current_pos.y + 1, current_pos.z)
                            }
                            Direction::SouthWest => {
                                Position::new(current_pos.x - 1, current_pos.y + 1, current_pos.z)
                            }
                        };

                        // Check if the new position is passable (tile allows movement)
                        let can_move = Self::can_move_to(world, &new_pos);

                        if can_move {
                            // Update player's position
                            if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
                                *pos = new_pos;
                            }
                            // Mark action as completed for energy deduction
                            resources
                                .input_buffer
                                .completed_actions
                                .push(PlayerAction::Move(direction));
                        } else {
                            // If can't move, add action back for later processing
                            new_actions.push(PlayerAction::Move(direction));
                        }
                    } else {
                        // No player found, add action back
                        new_actions.push(PlayerAction::Move(direction));
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
    world
        .query::<&Player>()
        .iter()
        .next()
        .map(|(entity, _)| entity)
}

/// AI System with split intent generation and execution phases.
///
/// This system integrates with the energy-driven turn system to:
/// 1. Generate intents when AI actors have sufficient energy
/// 2. Execute actions through a shared action queue
/// 3. Implement energy-aware waiting (patrol, flee, etc.)
/// 4. Read fresh world state snapshots for decision making
/// 5. Emit decision and target-change events via event bus
pub struct AISystem;

impl System for AISystem {
    fn name(&self) -> &str {
        "AISystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // This system is now a placeholder - actual AI processing is done
        // through run_with_events during AI turn phase
        SystemResult::Continue
    }
}

/// AI intent with detailed context for execution
#[derive(Debug, Clone)]
pub struct AIActionIntent {
    pub entity: Entity,
    pub intent_type: AIIntentType,
    pub target_entity: Option<Entity>,
    pub target_position: Option<Position>,
    pub reason: String, // For debugging and logging
}

#[derive(Debug, Clone, PartialEq)]
pub enum AIIntentType {
    Move(Direction),
    Attack(Entity),
    Wait,
    Flee(Direction),
    Patrol(Position),
    UseSkill,
}

impl AISystem {
    /// Run AI system with event bus integration.
    /// This is called during the AI turn phase to generate and execute intents.
    pub fn run_with_events(world: &mut ECSWorld) -> SystemResult {
        use crate::event_bus::{GameEvent, LogLevel};
        use crate::turn_system::energy_costs;

        // Collect AI entities with sufficient energy
        let ai_entities: Vec<(Entity, AIType, AIState, Position, u32)> = world
            .world
            .query::<(&AI, &Position, &Energy)>()
            .iter()
            .filter(|(entity, _)| {
                // Filter out player entities
                world.world.get::<&Player>(*entity).is_err()
            })
            .filter(|(_, (_, _, energy))| {
                // Only process AI with enough energy for an action
                energy.current >= energy_costs::FULL_ACTION
            })
            .map(|(entity, (ai, pos, energy))| {
                (
                    entity,
                    ai.ai_type.clone(),
                    ai.state.clone(),
                    pos.clone(),
                    energy.current,
                )
            })
            .collect();

        // Get fresh world state snapshot
        let player_positions: Vec<(Entity, Position, Option<u32>)> = world
            .world
            .query::<(&Position, &Player)>()
            .iter()
            .map(|(entity, (pos, _))| {
                let hp = world
                    .world
                    .get::<&Stats>(entity)
                    .map(|s| s.hp)
                    .ok();
                (entity, pos.clone(), hp)
            })
            .collect();

        // Generate intents for each AI entity
        for (ai_entity, ai_type, ai_state, ai_pos, _energy) in ai_entities {
            // Generate intent based on AI type and current state
            let intent = Self::generate_intent(
                &world.world,
                ai_entity,
                &ai_type,
                &ai_state,
                &ai_pos,
                &player_positions,
            );

            if let Some(action_intent) = intent {
                // Emit decision event
                world.publish_event(GameEvent::AIDecisionMade {
                    entity: ai_entity.id(),
                    decision: action_intent.reason.clone(),
                });

                // Update AI state and target if changed
                Self::update_ai_state(
                    &mut world.world,
                    ai_entity,
                    &action_intent,
                    &mut world.event_bus,
                );

                // Execute the intent
                Self::execute_intent(&mut world.world, &action_intent, &mut world.event_bus);
            }
        }

        SystemResult::Continue
    }

    /// Generate an intent for an AI entity based on current world state
    fn generate_intent(
        world: &World,
        ai_entity: Entity,
        ai_type: &AIType,
        ai_state: &AIState,
        ai_pos: &Position,
        player_positions: &[(Entity, Position, Option<u32>)],
    ) -> Option<AIActionIntent> {
        match ai_type {
            AIType::Aggressive => {
                Self::generate_aggressive_intent(world, ai_entity, ai_pos, player_positions)
            }
            AIType::Passive => {
                // Passive AI just waits
                Some(AIActionIntent {
                    entity: ai_entity,
                    intent_type: AIIntentType::Wait,
                    target_entity: None,
                    target_position: None,
                    reason: "Passive AI waiting".to_string(),
                })
            }
            AIType::Neutral => {
                Self::generate_neutral_intent(world, ai_entity, ai_pos, player_positions)
            }
            AIType::Patrol { path } => {
                Self::generate_patrol_intent(world, ai_entity, ai_state, ai_pos, path)
            }
        }
    }

    /// Generate intent for aggressive AI
    fn generate_aggressive_intent(
        world: &World,
        ai_entity: Entity,
        ai_pos: &Position,
        player_positions: &[(Entity, Position, Option<u32>)],
    ) -> Option<AIActionIntent> {
        let ai_range = world
            .get::<&AI>(ai_entity)
            .map(|ai| ai.range() as f32)
            .unwrap_or(10.0);

        // Check for status effects that impair AI
        let is_impaired = world
            .get::<&StatusEffects>(ai_entity)
            .map(|effects| {
                effects.has_effect(EffectType::Paralysis)
                    || effects.has_effect(EffectType::Frost)
                    || effects.has_effect(EffectType::Rooted)
            })
            .unwrap_or(false);

        if is_impaired {
            return Some(AIActionIntent {
                entity: ai_entity,
                intent_type: AIIntentType::Wait,
                target_entity: None,
                target_position: None,
                reason: "Impaired by status effect".to_string(),
            });
        }

        // Find closest player
        let mut closest_player: Option<(Entity, Position, f32)> = None;
        for (player_entity, player_pos, player_hp) in player_positions {
            // Skip dead players
            if player_hp.map_or(true, |hp| hp == 0) {
                continue;
            }

            let distance = ai_pos.distance_to(player_pos);
            if distance <= ai_range {
                let should_update = closest_player.as_ref().map_or(true, |(_, _, d)| distance < *d);
                if should_update {
                    closest_player = Some((*player_entity, player_pos.clone(), distance));
                }
            }
        }

        if let Some((target_entity, target_pos, distance)) = closest_player {
            // Check if adjacent (can attack)
            if distance <= 1.5 {
                // sqrt(2) ≈ 1.414 for diagonal
                return Some(AIActionIntent {
                    entity: ai_entity,
                    intent_type: AIIntentType::Attack(target_entity),
                    target_entity: Some(target_entity),
                    target_position: Some(target_pos),
                    reason: format!("Attacking target at distance {:.1}", distance),
                });
            }

            // Otherwise, move towards target
            let dx = (target_pos.x - ai_pos.x).signum();
            let dy = (target_pos.y - ai_pos.y).signum();
            let direction = Self::signum_to_direction(dx, dy);

            Some(AIActionIntent {
                entity: ai_entity,
                intent_type: AIIntentType::Move(direction),
                target_entity: Some(target_entity),
                target_position: Some(target_pos),
                reason: format!("Chasing target at distance {:.1}", distance),
            })
        } else {
            // No target in range, wait
            Some(AIActionIntent {
                entity: ai_entity,
                intent_type: AIIntentType::Wait,
                target_entity: None,
                target_position: None,
                reason: "No target in range".to_string(),
            })
        }
    }

    /// Generate intent for neutral AI (reacts when attacked)
    fn generate_neutral_intent(
        world: &World,
        ai_entity: Entity,
        ai_pos: &Position,
        player_positions: &[(Entity, Position, Option<u32>)],
    ) -> Option<AIActionIntent> {
        // Check if AI has a target (has been attacked)
        let has_target = world
            .get::<&AI>(ai_entity)
            .ok()
            .and_then(|ai| ai.target)
            .is_some();

        if has_target {
            // If provoked, act like aggressive
            Self::generate_aggressive_intent(world, ai_entity, ai_pos, player_positions)
        } else {
            // Otherwise wait
            Some(AIActionIntent {
                entity: ai_entity,
                intent_type: AIIntentType::Wait,
                target_entity: None,
                target_position: None,
                reason: "Neutral AI waiting".to_string(),
            })
        }
    }

    /// Generate intent for patrol AI
    fn generate_patrol_intent(
        world: &World,
        ai_entity: Entity,
        ai_state: &AIState,
        ai_pos: &Position,
        patrol_path: &[Position],
    ) -> Option<AIActionIntent> {
        if patrol_path.is_empty() {
            return Some(AIActionIntent {
                entity: ai_entity,
                intent_type: AIIntentType::Wait,
                target_entity: None,
                target_position: None,
                reason: "Empty patrol path".to_string(),
            });
        }

        // Find next patrol point
        let next_point = patrol_path
            .iter()
            .min_by_key(|p| {
                let dx = (p.x - ai_pos.x).abs();
                let dy = (p.y - ai_pos.y).abs();
                dx + dy
            })
            .cloned();

        if let Some(target_pos) = next_point {
            if ai_pos.x == target_pos.x && ai_pos.y == target_pos.y {
                // At patrol point, wait
                return Some(AIActionIntent {
                    entity: ai_entity,
                    intent_type: AIIntentType::Wait,
                    target_entity: None,
                    target_position: Some(target_pos),
                    reason: "At patrol point".to_string(),
                });
            }

            // Move towards patrol point
            let dx = (target_pos.x - ai_pos.x).signum();
            let dy = (target_pos.y - ai_pos.y).signum();
            let direction = Self::signum_to_direction(dx, dy);

            Some(AIActionIntent {
                entity: ai_entity,
                intent_type: AIIntentType::Patrol(target_pos.clone()),
                target_entity: None,
                target_position: Some(target_pos.clone()),
                reason: format!("Patrolling to ({}, {})", target_pos.x, target_pos.y),
            })
        } else {
            Some(AIActionIntent {
                entity: ai_entity,
                intent_type: AIIntentType::Wait,
                target_entity: None,
                target_position: None,
                reason: "No patrol point found".to_string(),
            })
        }
    }

    /// Update AI state and target based on intent
    fn update_ai_state(
        world: &mut World,
        ai_entity: Entity,
        intent: &AIActionIntent,
        event_bus: &mut crate::event_bus::EventBus,
    ) {
        if let Ok(mut ai) = world.get::<&mut AI>(ai_entity) {
            let old_state = ai.state.clone();
            let old_target = ai.target;

            // Update state based on intent
            match &intent.intent_type {
                AIIntentType::Attack(_) => ai.state = AIState::Attacking,
                AIIntentType::Move(_) => {
                    if matches!(ai.ai_type, AIType::Aggressive) {
                        ai.state = AIState::Chasing;
                    }
                }
                AIIntentType::Flee(_) => ai.state = AIState::Fleeing,
                AIIntentType::Patrol(_) => ai.state = AIState::Patrolling,
                AIIntentType::Wait => {
                    if !matches!(ai.state, AIState::Patrolling) {
                        ai.state = AIState::Idle;
                    }
                }
                _ => {}
            }

            // Update target
            let new_target = intent.target_entity;
            if old_target != new_target {
                ai.target = new_target;
                event_bus.publish(crate::event_bus::GameEvent::AITargetChanged {
                    entity: ai_entity.id(),
                    old_target: old_target.map(|e| e.id()),
                    new_target: new_target.map(|e| e.id()),
                });
            }
        }
    }

    /// Execute an AI intent
    fn execute_intent(
        world: &mut World,
        intent: &AIActionIntent,
        event_bus: &mut crate::event_bus::EventBus,
    ) {
        use crate::turn_system::energy_costs;

        match &intent.intent_type {
            AIIntentType::Move(_) | AIIntentType::Patrol(_) => {
                let direction = match &intent.intent_type {
                    AIIntentType::Move(d) => *d,
                    AIIntentType::Patrol(target_pos) => {
                        // Calculate direction to patrol point
                        if let Ok(current_pos) = world.get::<&Position>(intent.entity) {
                            let dx = (target_pos.x - current_pos.x).signum();
                            let dy = (target_pos.y - current_pos.y).signum();
                            Self::signum_to_direction(dx, dy)
                        } else {
                            Direction::North // fallback
                        }
                    }
                    _ => Direction::North,
                };

                // Get current position and calculate new position
                let (old_pos, new_pos) = {
                    if let Ok(current_pos) = world.get::<&Position>(intent.entity) {
                        let (dx, dy) = Self::direction_to_offset(direction);
                        let old = Position::new(current_pos.x, current_pos.y, current_pos.z);
                        let new = Position::new(
                            current_pos.x + dx,
                            current_pos.y + dy,
                            current_pos.z,
                        );
                        (Some(old), Some(new))
                    } else {
                        (None, None)
                    }
                };

                if let (Some(old_pos), Some(new_pos)) = (old_pos, new_pos) {
                    // Check if move is valid
                    if Self::can_move_to(world, &new_pos) {
                        if let Ok(mut pos) = world.get::<&mut Position>(intent.entity) {
                            *pos = new_pos.clone();

                            // Emit movement event
                            event_bus.publish(crate::event_bus::GameEvent::EntityMoved {
                                entity: intent.entity.id(),
                                from_x: old_pos.x,
                                from_y: old_pos.y,
                                to_x: new_pos.x,
                                to_y: new_pos.y,
                            });
                        }
                    }

                    // Consume energy
                    if let Ok(mut energy) = world.get::<&mut Energy>(intent.entity) {
                        energy.current = energy.current.saturating_sub(energy_costs::FULL_ACTION);
                    }
                }
            }
            AIIntentType::Attack(target_entity) => {
                // Attack is handled by CombatSystem, but we mark intent
                // For now, just consume energy and log
                if let Ok(mut energy) = world.get::<&mut Energy>(intent.entity) {
                    energy.current = energy.current.saturating_sub(energy_costs::FULL_ACTION);
                }

                event_bus.publish(crate::event_bus::GameEvent::LogMessage {
                    message: format!("AI entity {} attacks", intent.entity.id()),
                    level: crate::event_bus::LogLevel::Info,
                });
            }
            AIIntentType::Wait => {
                // Wait action consumes less energy
                if let Ok(mut energy) = world.get::<&mut Energy>(intent.entity) {
                    energy.current = energy.current.saturating_sub(energy_costs::WAIT);
                }
            }
            AIIntentType::Flee(direction) => {
                // Similar to move but in opposite direction
                let new_pos = {
                    if let Ok(current_pos) = world.get::<&Position>(intent.entity) {
                        let (dx, dy) = Self::direction_to_offset(*direction);
                        Some(Position::new(
                            current_pos.x + dx,
                            current_pos.y + dy,
                            current_pos.z,
                        ))
                    } else {
                        None
                    }
                };

                if let Some(new_pos) = new_pos {
                    if Self::can_move_to(world, &new_pos) {
                        if let Ok(mut pos) = world.get::<&mut Position>(intent.entity) {
                            *pos = new_pos;
                        }
                    }

                    // Consume energy
                    if let Ok(mut energy) = world.get::<&mut Energy>(intent.entity) {
                        energy.current = energy.current.saturating_sub(energy_costs::FULL_ACTION);
                    }
                }
            }
            AIIntentType::UseSkill => {
                // Placeholder for future skill implementation
                if let Ok(mut energy) = world.get::<&mut Energy>(intent.entity) {
                    energy.current = energy.current.saturating_sub(energy_costs::FULL_ACTION);
                }
            }
        }
    }

    /// Check if an entity can move to the target position
    fn can_move_to(world: &World, target_pos: &Position) -> bool {
        let mut passable = false;
        for (_, (pos, tile)) in world.query::<(&Position, &Tile)>().iter() {
            if pos.x == target_pos.x && pos.y == target_pos.y && pos.z == target_pos.z {
                if tile.is_passable {
                    passable = true;
                } else {
                    return false;
                }
                break;
            }
        }
        passable
    }

    /// Convert direction signum to Direction enum
    fn signum_to_direction(dx: i32, dy: i32) -> Direction {
        match (dx, dy) {
            (0, -1) => Direction::North,
            (0, 1) => Direction::South,
            (1, 0) => Direction::East,
            (-1, 0) => Direction::West,
            (1, -1) => Direction::NorthEast,
            (-1, -1) => Direction::NorthWest,
            (1, 1) => Direction::SouthEast,
            (-1, 1) => Direction::SouthWest,
            _ => Direction::North, // Default fallback
        }
    }

    /// Convert Direction to offset
    fn direction_to_offset(direction: Direction) -> (i32, i32) {
        match direction {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, -1),
            Direction::NorthWest => (-1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
        }
    }
}

/// Bridges `PlayerAction::Attack` into combat resolution and emits events via
/// `CombatSystem::run_with_events` so UI/logging layers stay in sync.
pub struct CombatSystem;

impl System for CombatSystem {
    fn name(&self) -> &str {
        "CombatSystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // 注意：这个系统现在需要通过 ECSWorld 来运行，以便访问事件总线
        // 暂时保留原有逻辑，实际应该通过 run_with_events 方法调用
        SystemResult::Continue
    }
}

impl CombatSystem {
    /// 使用事件总线的战斗系统运行方法
    pub fn run_with_events(world: &mut ECSWorld) -> SystemResult {
        use crate::event_bus::GameEvent;

        // 处理待处理的玩家战斗动作
        let actions_to_process = std::mem::take(&mut world.resources.input_buffer.pending_actions);
        let mut new_actions = Vec::new();

        for action in actions_to_process {
            match action {
                PlayerAction::Attack(ref target_pos) => {
                    if let Some(player_entity) = find_player_entity(&world.world) {
                        // 获取玩家位置
                        let player_pos = match world.world.get::<&Position>(player_entity) {
                            Ok(pos) => Position::new(pos.x, pos.y, pos.z),
                            Err(_) => {
                                new_actions.push(action);
                                continue;
                            }
                        };

                        // 获取玩家名称
                        let player_name = world
                            .world
                            .get::<&Actor>(player_entity)
                            .map(|a| a.name.clone())
                            .unwrap_or_else(|_| "Player".to_string());

                        // 计算攻击目标位置
                        let attack_pos = Position::new(
                            player_pos.x + target_pos.x,
                            player_pos.y + target_pos.y,
                            player_pos.z,
                        );

                        // 查找目标位置的敌人
                        let mut target_info: Option<(Entity, String, u32)> = None;
                        for (entity, (pos, actor, _stats)) in
                            world.world.query::<(&Position, &Actor, &Stats)>().iter()
                        {
                            if actor.faction == Faction::Enemy
                                && pos.x == attack_pos.x
                                && pos.y == attack_pos.y
                                && pos.z == attack_pos.z
                            {
                                // 将 entity 转换为 u32 (使用内部表示)
                                let entity_id = entity.id();
                                target_info = Some((entity, actor.name.clone(), entity_id));
                                break;
                            }
                        }

                        if let Some((target, target_name, target_id)) = target_info {
                            let player_id = player_entity.id();

                            // 执行战斗计算 - 克隆 Stats 以避免借用问题，并使用潜行判定
                            let combat_result = {
                                let player_stats = world
                                    .world
                                    .get::<&Stats>(player_entity)
                                    .ok()
                                    .map(|s| (*s).clone());
                                let target_stats =
                                    world.world.get::<&Stats>(target).ok().map(|s| (*s).clone());

                                if let (Some(mut temp_player), Some(mut temp_target)) =
                                    (player_stats, target_stats)
                                {
                                    let mut attacker = SimpleCombatant::new(&mut temp_player);
                                    let mut defender = SimpleCombatant::new(&mut temp_target);

                                    // 构建阻挡检测闭包（基于地图Tile）
                                    let z = player_pos.z;
                                    let mut blocked_set = std::collections::HashSet::new();
                                    for (_, (pos, tile)) in
                                        world.world.query::<(&Position, &Tile)>().iter()
                                    {
                                        if pos.z == z && tile.blocks_sight {
                                            blocked_set.insert((pos.x, pos.y));
                                        }
                                    }
                                    let is_blocked =
                                        |x: i32, y: i32| -> bool { blocked_set.contains(&(x, y)) };

                                    // 获取攻击者FOV范围
                                    let attacker_fov_range = world
                                        .world
                                        .get::<&Viewshed>(player_entity)
                                        .map(|v| v.range as u32)
                                        .unwrap_or(8);

                                    // 攻击参数（带潜行）
                                    let mut params = ::combat::AttackParams {
                                        attacker: &mut attacker,
                                        attacker_id: player_id,
                                        attacker_x: player_pos.x,
                                        attacker_y: player_pos.y,
                                        defender: &mut defender,
                                        defender_id: target_id,
                                        defender_x: attack_pos.x,
                                        defender_y: attack_pos.y,
                                        is_blocked: &is_blocked,
                                        attacker_fov_range,
                                    };

                                    // 执行战斗（包含潜行与反击）
                                    let result =
                                        ::combat::Combat::perform_attack_with_ambush(&mut params);
                                    Some((temp_player.hp, temp_target.hp, result))
                                } else {
                                    None
                                }
                            };

                            if let Some((new_player_hp, new_target_hp, combat_result)) =
                                combat_result
                            {
                                // 将 CombatResult 事件映射到游戏事件总线
                                for ev in &combat_result.events {
                                    match ev {
                                        ::combat::CombatEvent::CombatStarted {
                                            attacker,
                                            defender,
                                        } => {
                                            world.publish_event(GameEvent::CombatStarted {
                                                attacker: *attacker,
                                                defender: *defender,
                                            });
                                        }
                                        ::combat::CombatEvent::DamageDealt {
                                            attacker,
                                            victim,
                                            damage,
                                            is_critical,
                                        } => {
                                            world.publish_event(GameEvent::DamageDealt {
                                                attacker: *attacker,
                                                victim: *victim,
                                                damage: *damage,
                                                is_critical: *is_critical,
                                            });
                                        }
                                        ::combat::CombatEvent::EntityDied {
                                            entity,
                                            entity_name,
                                        } => {
                                            world.publish_event(GameEvent::EntityDied {
                                                entity: *entity,
                                                entity_name: entity_name.clone(),
                                            });
                                        }
                                        ::combat::CombatEvent::Ambush {
                                            attacker: _,
                                            defender: _,
                                        } => {
                                            world.publish_event(GameEvent::LogMessage {
                                                message: format!(
                                                    "伏击！{} -> {}",
                                                    player_name, target_name
                                                ),
                                                level: LogLevel::Info,
                                            });
                                        }
                                    }
                                }

                                // 应用实际伤害（更新HP）
                                if let Ok(mut stats) = world.world.get::<&mut Stats>(player_entity)
                                {
                                    stats.hp = new_player_hp;
                                }
                                if let Ok(mut stats) = world.world.get::<&mut Stats>(target) {
                                    stats.hp = new_target_hp;
                                }

                                // 发布日志消息
                                for log in &combat_result.logs {
                                    world.publish_event(GameEvent::LogMessage {
                                        message: log.clone(),
                                        level: LogLevel::Info,
                                    });
                                }

                                // 检查目标是否死亡（移除实体）
                                if new_target_hp == 0 {
                                    let _ = world.world.despawn(target);
                                }

                                // 检查玩家是否死亡
                                if new_player_hp == 0 {
                                    world.publish_event(GameEvent::GameOver {
                                        reason: "你被击败了！".to_string(),
                                    });
                                }

                                // 消耗能量
                                if let Ok(mut energy) =
                                    world.world.get::<&mut Energy>(player_entity)
                                {
                                    energy.current = energy.current.saturating_sub(100);
                                }
                            }
                        } else {
                            // 没有找到目标
                            world.publish_event(GameEvent::LogMessage {
                                message: "该位置没有敌人！".to_string(),
                                level: LogLevel::Info,
                            });
                            new_actions.push(action);
                        }
                    } else {
                        new_actions.push(action);
                    }
                }
                // 其他动作返回队列
                _ => {
                    new_actions.push(action);
                }
            }
        }

        // 恢复未处理的动作
        world.resources.input_buffer.pending_actions = new_actions;

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
    fn id(&self) -> u32 {
        0 // 在ECS上下文中，这将由ECS Entity ID替换
    }

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
                resources.game_state.game_state = GameStatus::GameOver {
                    reason: GameOverReason::Died("死亡"),
                };
                resources
                    .game_state
                    .message_log
                    .push("You have died... Game Over!".to_string());
                return SystemResult::Stop; // End the game
            }
        }

        // Check for victory conditions (e.g., reaching max depth)
        if resources.game_state.depth >= resources.config.max_depth {
            // Check if player is on the final level and in a winning condition
            // For now, if the player reaches the max depth, they win
            for (entity, (actor, pos)) in world.query::<(&Actor, &Position)>().iter() {
                if actor.faction == Faction::Player && pos.z as usize == resources.config.max_depth
                {
                    resources.game_state.game_state = GameStatus::Victory;
                    resources
                        .game_state
                        .message_log
                        .push("Congratulations! You won the game!".to_string());
                    return SystemResult::Stop; // End the game
                }
            }
        }

        // Update FOV for entities
        let entities: Vec<Entity> = world
            .query::<&Viewshed>()
            .iter()
            .map(|(entity, _)| entity)
            .collect();
        for entity in entities {
            Self::update_fov(world, entity);
        }
        SystemResult::Continue
    }
}

impl FOVSystem {
    /// 更新实体的视野
    ///
    /// 根据 Viewshed 组件中配置的算法类型，计算实体可见的格子。
    /// 考虑地形阻挡（墙壁、障碍物等）。
    pub fn update_fov(world: &mut World, entity: Entity) {
        // 获取实体位置和视野配置
        let (pos, range, algorithm) = match (
            world.get::<&Position>(entity),
            world.get::<&Viewshed>(entity),
        ) {
            (Ok(p), Ok(v)) => (p.clone(), v.range, v.algorithm),
            _ => return, // 没有必要组件，跳过
        };

        // 计算可见格子
        let visible_positions = match algorithm {
            crate::ecs::FovAlgorithm::ShadowCasting => Self::shadow_casting_fov(&pos, range, world),
            crate::ecs::FovAlgorithm::DiamondWalls => Self::diamond_walls_fov(&pos, range, world),
            crate::ecs::FovAlgorithm::RayCasting => Self::ray_casting_fov(&pos, range, world),
        };

        // 更新 Viewshed 组件
        if let Ok(mut viewshed) = world.get::<&mut Viewshed>(entity) {
            // 将新可见的格子添加到记忆中
            for visible_pos in &visible_positions {
                if !viewshed.memory.contains(visible_pos) {
                    viewshed.memory.push(visible_pos.clone());
                }
            }

            // 更新当前可见格子
            viewshed.visible_tiles = visible_positions;
            viewshed.dirty = false;
        }
    }

    /// 阴影投射算法
    ///
    /// 最真实的 FOV 算法，适合大多数 Roguelike 游戏。
    /// 时间复杂度：O(n²) 其中 n 是视野范围
    fn shadow_casting_fov(pos: &Position, range: u8, world: &World) -> Vec<Position> {
        let mut visible = vec![pos.clone()]; // 当前位置总是可见
        let range_sq = (range as i32 * range as i32) as f32;

        for dx in -(range as i32)..=(range as i32) {
            for dy in -(range as i32)..=(range as i32) {
                // 跳过超出圆形范围的格子
                let distance_sq = (dx * dx + dy * dy) as f32;
                if distance_sq > range_sq {
                    continue;
                }

                let target_pos = Position::new(pos.x + dx, pos.y + dy, pos.z);

                // 使用光线追踪检查视线
                if Self::has_line_of_sight(pos, &target_pos, world) {
                    visible.push(target_pos);
                }
            }
        }

        visible
    }

    /// 菱形墙算法
    ///
    /// 适合正交移动的地图，视野呈菱形。
    /// 特点：相邻的墙壁总是可见
    fn diamond_walls_fov(pos: &Position, range: u8, world: &World) -> Vec<Position> {
        let mut visible = vec![pos.clone()];
        let range_i32 = range as i32;

        for dx in -range_i32..=range_i32 {
            for dy in -range_i32..=range_i32 {
                // 菱形范围：曼哈顿距离
                let distance = dx.abs() + dy.abs();
                if distance > range_i32 * 2 {
                    continue;
                }

                let target_pos = Position::new(pos.x + dx, pos.y + dy, pos.z);

                // 检查视线，但相邻墙壁总是可见
                let is_adjacent_wall = distance <= 1 && Self::is_blocked(&target_pos, world);
                if is_adjacent_wall || Self::has_line_of_sight(pos, &target_pos, world) {
                    visible.push(target_pos);
                }
            }
        }

        visible
    }

    /// 光线投射/Bresenham 算法
    ///
    /// 性能最优的 FOV 算法，使用 Bresenham 直线算法。
    /// 时间复杂度：O(n²) 但常数因子最小
    fn ray_casting_fov(pos: &Position, range: u8, world: &World) -> Vec<Position> {
        let mut visible = vec![pos.clone()];
        let range_sq = (range as i32 * range as i32) as f32;

        for dx in -(range as i32)..=(range as i32) {
            for dy in -(range as i32)..=(range as i32) {
                let distance_sq = (dx * dx + dy * dy) as f32;
                if distance_sq > range_sq {
                    continue;
                }

                let target_pos = Position::new(pos.x + dx, pos.y + dy, pos.z);

                // 使用 Bresenham 算法追踪光线
                if Self::bresenham_line_of_sight(pos, &target_pos, world) {
                    visible.push(target_pos);
                }
            }
        }

        visible
    }

    /// 检查两点间是否有视线（递归光线追踪）
    fn has_line_of_sight(from: &Position, to: &Position, world: &World) -> bool {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let steps = dx.abs().max(dy.abs());

        if steps == 0 {
            return true;
        }

        let x_inc = dx as f32 / steps as f32;
        let y_inc = dy as f32 / steps as f32;

        let mut x = from.x as f32;
        let mut y = from.y as f32;

        for _ in 0..steps {
            x += x_inc;
            y += y_inc;

            let check_pos = Position::new(x.round() as i32, y.round() as i32, from.z);

            // 如果到达目标位置，视线畅通
            if check_pos.x == to.x && check_pos.y == to.y {
                return true;
            }

            // 如果遇到阻挡，视线被阻断
            if Self::is_blocked(&check_pos, world) {
                return false;
            }
        }

        true
    }

    /// Bresenham 直线算法检查视线
    fn bresenham_line_of_sight(from: &Position, to: &Position, world: &World) -> bool {
        let mut x = from.x;
        let mut y = from.y;
        let dx = (to.x - from.x).abs();
        let dy = (to.y - from.y).abs();
        let sx = if from.x < to.x { 1 } else { -1 };
        let sy = if from.y < to.y { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            // 到达目标
            if x == to.x && y == to.y {
                return true;
            }

            // 检查当前位置是否阻挡视线
            let check_pos = Position::new(x, y, from.z);
            if x != from.x || y != from.y {
                // 不检查起点
                if Self::is_blocked(&check_pos, world) {
                    return false;
                }
            }

            // Bresenham 算法步进
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// 检查某个位置是否阻挡视线
    fn is_blocked(pos: &Position, world: &World) -> bool {
        // 查找该位置的 Tile 组件
        for (_, (tile_pos, tile)) in world.query::<(&Position, &Tile)>().iter() {
            if tile_pos.x == pos.x && tile_pos.y == pos.y && tile_pos.z == pos.z {
                return tile.blocks_sight;
            }
        }

        // 如果没有 Tile 信息，默认不阻挡（假设是空地）
        false
    }
}

/// Effect processing phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectPhase {
    StartOfTurn,
    EndOfTurn,
}

pub struct EffectSystem {
    pub phase: EffectPhase,
}

impl EffectSystem {
    pub fn new() -> Self {
        Self {
            phase: EffectPhase::EndOfTurn,
        }
    }

    /// Run the effect system with event bus support
    pub fn run_with_events(ecs_world: &mut ECSWorld, phase: EffectPhase) -> SystemResult {
        use crate::ecs::{StatusEffects, Stats, Player};
        use crate::event_bus::GameEvent;
        
        let current_turn = ecs_world.resources.clock.turn_count;
        
        // Collect entities with status effects
        let mut entities_to_process: Vec<(Entity, StatusEffects, Stats, bool, String)> = Vec::new();
        
        for (entity, (effects, stats)) in ecs_world.world.query::<(&StatusEffects, &Stats)>().iter() {
            let is_player = ecs_world.world.get::<&Player>(entity).is_ok();
            let entity_name = ecs_world.world.get::<&Actor>(entity)
                .map(|a| a.name.clone())
                .unwrap_or_else(|_| "Unknown".to_string());
            entities_to_process.push((entity, effects.clone(), stats.clone(), is_player, entity_name));
        }
        
        // Process each entity's effects
        for (entity, mut status_effects, mut stats, is_player, entity_name) in entities_to_process {
            // Skip if already processed this turn
            if status_effects.last_tick_turn >= current_turn {
                continue;
            }
            
            status_effects.last_tick_turn = current_turn;
            
            let mut total_damage = 0u32;
            let mut total_healing = 0u32;
            let mut effects_to_remove = Vec::new();
            
            // Process each effect
            for (idx, effect) in status_effects.effects.iter_mut().enumerate() {
                let effect_type = effect.effect_type();
                let effect_name = format!("{:?}", effect_type);
                
                // Apply effect based on phase
                match phase {
                    EffectPhase::StartOfTurn => {
                        // Some effects trigger at start of turn (e.g., paralysis check)
                        match effect_type {
                            crate::ecs::EffectType::Paralysis | 
                            crate::ecs::EffectType::Rooted => {
                                // These effects prevent action but don't deal damage
                            }
                            _ => {}
                        }
                    }
                    EffectPhase::EndOfTurn => {
                        // Most DoT effects trigger at end of turn
                        let damage = effect.damage();
                        if damage > 0 {
                            total_damage += damage;
                            
                            ecs_world.publish_event(GameEvent::StatusEffectTicked {
                                entity: entity.id(),
                                status: effect_name.clone(),
                                damage,
                                remaining_turns: effect.turns(),
                            });
                        }
                    }
                }
                
                // Decrement duration (only at end of turn)
                if phase == EffectPhase::EndOfTurn {
                    let still_active = effect.update();
                    if !still_active {
                        effects_to_remove.push(idx);
                        ecs_world.publish_event(GameEvent::StatusRemoved {
                            entity: entity.id(),
                            status: effect_name,
                            reason: "expired".to_string(),
                        });
                    }
                }
            }
            
            // Remove expired effects (in reverse order to maintain indices)
            for &idx in effects_to_remove.iter().rev() {
                status_effects.effects.remove(idx);
            }
            
            // Apply accumulated damage/healing
            if total_damage > 0 {
                stats.hp = stats.hp.saturating_sub(total_damage);
                
                // Check for death
                if stats.hp == 0 {
                    ecs_world.publish_event(GameEvent::EntityDied {
                        entity: entity.id(),
                        entity_name: entity_name.clone(),
                    });
                    
                    // Remove all effects on death
                    ecs_world.publish_event(GameEvent::StatusRemoved {
                        entity: entity.id(),
                        status: "所有效果".to_string(),
                        reason: "death".to_string(),
                    });
                    
                    // Despawn entity
                    let _ = ecs_world.world.despawn(entity);
                    continue;
                }
            }
            
            if total_healing > 0 {
                stats.hp = (stats.hp + total_healing).min(stats.max_hp);
            }
            
            // Update components
            let _ = ecs_world.world.insert(entity, (status_effects, stats));
        }
        
        SystemResult::Continue
    }
}

impl System for EffectSystem {
    fn name(&self) -> &str {
        "EffectSystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        use crate::ecs::{StatusEffects, Stats, Player};
        
        let current_turn = resources.clock.turn_count;
        
        // Collect entities with status effects
        let mut entities_to_process: Vec<(Entity, StatusEffects, Stats, bool, String)> = Vec::new();
        
        for (entity, (effects, stats)) in world.query::<(&StatusEffects, &Stats)>().iter() {
            let is_player = world.get::<&Player>(entity).is_ok();
            let entity_name = world.get::<&Actor>(entity)
                .map(|a| a.name.clone())
                .unwrap_or_else(|_| "Unknown".to_string());
            entities_to_process.push((entity, effects.clone(), stats.clone(), is_player, entity_name));
        }
        
        // Process each entity's effects
        for (entity, mut status_effects, mut stats, _is_player, entity_name) in entities_to_process {
            // Skip if already processed this turn
            if status_effects.last_tick_turn >= current_turn {
                continue;
            }
            
            status_effects.last_tick_turn = current_turn;
            
            let mut total_damage = 0u32;
            let mut effects_to_remove = Vec::new();
            
            // Process each effect
            for (idx, effect) in status_effects.effects.iter_mut().enumerate() {
                // Apply damage/healing from effect (at end of turn)
                if self.phase == EffectPhase::EndOfTurn {
                    let damage = effect.damage();
                    if damage > 0 {
                        total_damage += damage;
                    }
                    
                    // Decrement duration
                    let still_active = effect.update();
                    if !still_active {
                        effects_to_remove.push(idx);
                        resources.game_state.message_log.push(format!(
                            "{}的{:?}效果已消失",
                            entity_name,
                            effect.effect_type()
                        ));
                    }
                }
            }
            
            // Remove expired effects
            for &idx in effects_to_remove.iter().rev() {
                status_effects.effects.remove(idx);
            }
            
            // Apply accumulated damage
            if total_damage > 0 {
                stats.hp = stats.hp.saturating_sub(total_damage);
                
                // Check for death
                if stats.hp == 0 {
                    resources.game_state.message_log.push(format!("{} 死亡", entity_name));
                    let _ = world.despawn(entity);
                    continue;
                }
            }
            
            // Update components
            let _ = world.insert(entity, (status_effects, stats));
        }
        
        SystemResult::Continue
    }
}

/// Legacy energy refill system kept for backwards compatibility with older
/// pipelines. The interactive loop skips it because `TurnSystem` now manages
/// regeneration explicitly.
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
        let actions_to_process = std::mem::take(&mut resources.input_buffer.pending_actions);
        let mut new_actions = Vec::new();

        for action in actions_to_process {
            match action {
                PlayerAction::UseItem(slot_index) => {
                    if let Some(player_entity) = find_player_entity(world) {
                        let player_id = player_entity.id();

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
                                                    if let Ok(mut stats) =
                                                        world.get::<&mut Stats>(player_entity)
                                                    {
                                                        stats.hp =
                                                            (stats.hp + amount).min(stats.max_hp);
                                                        let message = format!(
                                                            "You drink a {}, healing {} HP.",
                                                            item.name, amount
                                                        );

                                                        // Add message to game state log (original behavior)
                                                        resources
                                                            .game_state
                                                            .message_log
                                                            .push(message);
                                                        if resources.game_state.message_log.len()
                                                            > 10
                                                        {
                                                            resources
                                                                .game_state
                                                                .message_log
                                                                .remove(0);
                                                        }
                                                    }
                                                }
                                                ConsumableEffect::Damage { amount } => {
                                                    // Apply damage to player (negative effect)
                                                    if let Ok(mut stats) =
                                                        world.get::<&mut Stats>(player_entity)
                                                    {
                                                        stats.hp = stats.hp.saturating_sub(*amount);
                                                        let message = format!(
                                                            "You drink a {}, taking {} damage!",
                                                            item.name, amount
                                                        );

                                                        // Add message to game state log (original behavior)
                                                        resources
                                                            .game_state
                                                            .message_log
                                                            .push(message);
                                                        if resources.game_state.message_log.len()
                                                            > 10
                                                        {
                                                            resources
                                                                .game_state
                                                                .message_log
                                                                .remove(0);
                                                        }
                                                    }
                                                }
                                                ConsumableEffect::Buff {
                                                    stat,
                                                    value,
                                                    duration: _,
                                                } => {
                                                    // Apply stat buff to player
                                                    if let Ok(mut stats) =
                                                        world.get::<&mut Stats>(player_entity)
                                                    {
                                                        match stat {
                                                            StatType::Hp => {
                                                                stats.max_hp = (stats.max_hp as i32
                                                                    + value)
                                                                    as u32
                                                            }
                                                            StatType::Attack => {
                                                                stats.attack = (stats.attack as i32
                                                                    + value)
                                                                    as u32
                                                            }
                                                            StatType::Defense => {
                                                                stats.defense =
                                                                    (stats.defense as i32 + value)
                                                                        as u32
                                                            }
                                                            StatType::Accuracy => {
                                                                stats.accuracy =
                                                                    (stats.accuracy as i32 + value)
                                                                        as u32
                                                            }
                                                            StatType::Evasion => {
                                                                stats.evasion =
                                                                    (stats.evasion as i32 + value)
                                                                        as u32
                                                            }
                                                        }
                                                        let message = format!(
                                                            "You feel {}!",
                                                            match stat {
                                                                StatType::Hp =>
                                                                    format!("healthier ({})", value),
                                                                StatType::Attack =>
                                                                    format!("stronger ({})", value),
                                                                StatType::Defense =>
                                                                    format!("tougher ({})", value),
                                                                StatType::Accuracy => format!(
                                                                    "more accurate ({})",
                                                                    value
                                                                ),
                                                                StatType::Evasion => format!(
                                                                    "more evasive ({})",
                                                                    value
                                                                ),
                                                            }
                                                        );

                                                        // Add message to game state log (original behavior)
                                                        resources
                                                            .game_state
                                                            .message_log
                                                            .push(message);
                                                        if resources.game_state.message_log.len()
                                                            > 10
                                                        {
                                                            resources
                                                                .game_state
                                                                .message_log
                                                                .remove(0);
                                                        }
                                                    }
                                                }
                                                ConsumableEffect::Teleport => {
                                                    // Teleport player to random location in level
                                                    if let Ok(mut pos) =
                                                        world.get::<&mut Position>(player_entity)
                                                    {
                                                        use rand::Rng;
                                                        // Use proper RNG for random position
                                                        pos.x = 5 + resources.rng.gen_range(0..15); // Random position between 5-19
                                                        pos.y = 5 + resources.rng.gen_range(0..15); // Random position between 5-19
                                                        let message =
                                                            "You teleport randomly!".to_string();

                                                        // Add message to game state log (original behavior)
                                                        resources
                                                            .game_state
                                                            .message_log
                                                            .push(message);
                                                        if resources.game_state.message_log.len()
                                                            > 10
                                                        {
                                                            resources
                                                                .game_state
                                                                .message_log
                                                                .remove(0);
                                                        }
                                                    }
                                                }
                                                ConsumableEffect::Identify => {
                                                    // For now, just add a message
                                                    let message =
                                                        "You feel more perceptive.".to_string();

                                                    // Add message to game state log (original behavior)
                                                    resources.game_state.message_log.push(message);
                                                    if resources.game_state.message_log.len() > 10 {
                                                        resources.game_state.message_log.remove(0);
                                                    }
                                                }
                                            }

                                            // Remove the consumed item from inventory
                                            inventory.items.remove(slot_index);
                                        }
                                        _ => {
                                            let message = "Cannot use this item.".to_string();

                                            // Add message to game state log (original behavior)
                                            resources.game_state.message_log.push(message);
                                            if resources.game_state.message_log.len() > 10 {
                                                resources.game_state.message_log.remove(0);
                                            }
                                        }
                                    }
                                } else {
                                    let message = "No item in this slot.".to_string();

                                    // Add message to game state log (original behavior)
                                    resources.game_state.message_log.push(message);
                                    if resources.game_state.message_log.len() > 10 {
                                        resources.game_state.message_log.remove(0);
                                    }
                                }
                            } else {
                                let message = "Invalid inventory slot.".to_string();
                                new_actions.push(action);

                                // Add message to game state log (original behavior)
                                resources.game_state.message_log.push(message);
                                if resources.game_state.message_log.len() > 10 {
                                    resources.game_state.message_log.remove(0);
                                }
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
                    let drop_result: Option<(Position, ECSItem, u32)> = if let Some(player_entity) =
                        find_player_entity(world)
                    {
                        let player_id = player_entity.id();

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
                                if let Some(item_to_drop) = inventory.items.remove(slot_index).item
                                {
                                    Some((player_pos.clone(), item_to_drop, player_id)) // Clone the position to get owned value
                                } else {
                                    // Add message to game state log (original behavior)
                                    resources
                                        .game_state
                                        .message_log
                                        .push("No item in this slot to drop.".to_string());
                                    if resources.game_state.message_log.len() > 10 {
                                        resources.game_state.message_log.remove(0);
                                    }
                                    new_actions.push(action);
                                    None
                                }
                            } else {
                                // Add message to game state log (original behavior)
                                resources
                                    .game_state
                                    .message_log
                                    .push("Invalid inventory slot.".to_string());
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
                    if let Some((player_pos, item_to_drop, player_id)) = drop_result {
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
                                quantity: item_to_drop.quantity,
                                level: item_to_drop.level,
                                cursed: item_to_drop.cursed,
                                charges: item_to_drop.charges,
                                detailed_data: item_to_drop.detailed_data.clone(),
                            },
                            Tile {
                                terrain_type: TerrainType::Empty,
                                is_passable: true,
                                blocks_sight: false,
                                has_items: true,
                                has_monster: false,
                            },
                        ));

                        // Add message to game state log (original behavior)
                        resources
                            .game_state
                            .message_log
                            .push(format!("You dropped {}.", item_to_drop.name));
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
                    world.query::<(&Position, &Actor)>().iter()
                {
                    if world.get::<&Player>(player_entity).is_err() {
                        continue;
                    }

                    let mut items_for_player = Vec::new();
                    for (item_entity, (pos, item)) in world.query::<(&Position, &ECSItem)>().iter()
                    {
                        if pos.x == player_pos.x && pos.y == player_pos.y && pos.z == player_pos.z {
                            items_for_player.push((item_entity, item.clone(), item.name.clone()));
                        }
                    }

                    let mut available_slots = world
                        .get::<&Inventory>(player_entity)
                        .ok()
                        .map(|inventory| inventory.max_slots.saturating_sub(inventory.items.len()))
                        .unwrap_or(0);

                    if available_slots == 0 {
                        resources
                            .game_state
                            .message_log
                            .push("Your inventory is full!".to_string());
                        if resources.game_state.message_log.len() > 10 {
                            resources.game_state.message_log.remove(0);
                        }
                        continue;
                    }

                    for (item_entity, item_clone, item_name) in items_for_player {
                        if available_slots == 0 {
                            break;
                        }
                        actions.push((player_entity, item_entity, item_clone, item_name));
                        available_slots -= 1;
                    }
                }
                actions
            };

            for (player_entity, item_entity, item, item_name) in pickup_actions {
                let mut picked_up = false;
                if let Ok(mut inventory) = world.get::<&mut Inventory>(player_entity) {
                    if inventory.items.len() < inventory.max_slots {
                        inventory.items.push(ItemSlot {
                            item: Some(item),
                            quantity: 1,
                        });
                        picked_up = true;
                    } else {
                        resources
                            .game_state
                            .message_log
                            .push("Your inventory is full!".to_string());
                        if resources.game_state.message_log.len() > 10 {
                            resources.game_state.message_log.remove(0);
                        }
                    }
                }
                if picked_up {
                    let _ = world.despawn(item_entity);
                    resources
                        .game_state
                        .message_log
                        .push(format!("You picked up {}.", item_name));
                    if resources.game_state.message_log.len() > 10 {
                        resources.game_state.message_log.remove(0);
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
        let actions_to_process = std::mem::take(&mut resources.input_buffer.pending_actions);
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
                                if pos.x == player_pos.x
                                    && pos.y == player_pos.y
                                    && pos.z == player_pos.z
                                {
                                    if matches!(tile.terrain_type, TerrainType::StairsDown) {
                                        on_stairs_down = true;
                                        break;
                                    }
                                }
                            }

                            if on_stairs_down {
                                // Queue up level generation and player movement
                                let message = "You descend to the next level...".to_string();

                                // Message already added above in the game state log
                                // resources.game_state.message_log.push(message);
                                // if resources.game_state.message_log.len() > 10 {
                                //     resources.game_state.message_log.remove(0);
                                // }

                                resources.game_state.depth = (player_pos.z + 1) as usize;

                                // Move player to new level
                                if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
                                    pos.z += 1;
                                    // Place player at stairs up position
                                    // For now, we'll place them at a default position (10, 10) on the new level
                                    pos.x = 10;
                                    pos.y = 10;
                                }

                                // Add message to game state log (original behavior)
                                resources
                                    .game_state
                                    .message_log
                                    .push("You descend to the next level...".to_string());
                                if resources.game_state.message_log.len() > 10 {
                                    resources.game_state.message_log.remove(0);
                                }

                                // Add generation of new level after all actions are processed
                                // We'll generate it in a separate pass
                            } else {
                                resources
                                    .game_state
                                    .message_log
                                    .push("You need to stand on stairs to descend.".to_string());
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
                                if pos.x == player_pos.x
                                    && pos.y == player_pos.y
                                    && pos.z == player_pos.z
                                {
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

                                    let message = "You ascend to the previous level...".to_string();

                                    // Add message to game state log (original behavior)
                                    resources.game_state.message_log.push(message);
                                    if resources.game_state.message_log.len() > 10 {
                                        resources.game_state.message_log.remove(0);
                                    }

                                    resources.game_state.depth = (player_pos.z - 1) as usize;

                                    // Add message to game state log (original behavior)
                                    resources
                                        .game_state
                                        .message_log
                                        .push("You ascend to the previous level...".to_string());
                                    if resources.game_state.message_log.len() > 10 {
                                        resources.game_state.message_log.remove(0);
                                    }

                                    // Generate level for the new depth after actions are processed
                                } else {
                                    // Player is at dungeon level 0, can't go higher
                                    let message = "You can't go up from here.".to_string();

                                    // Add message to game state log (original behavior)
                                    resources.game_state.message_log.push(message);
                                    if resources.game_state.message_log.len() > 10 {
                                        resources.game_state.message_log.remove(0);
                                    }
                                }
                            } else {
                                let message = "You need to stand on stairs to ascend.".to_string();
                                new_actions.push(action);

                                // Add message to game state log (original behavior)
                                resources.game_state.message_log.push(message);
                                if resources.game_state.message_log.len() > 10 {
                                    resources.game_state.message_log.remove(0);
                                }
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
    /// Run dungeon system with event bus access for LevelChanged events
    pub fn run_with_events(ecs_world: &mut ECSWorld) -> SystemResult {
        use crate::event_bus::GameEvent;

        // Process pending player actions for dungeon navigation
        let actions_to_process =
            std::mem::take(&mut ecs_world.resources.input_buffer.pending_actions);
        let mut new_actions = Vec::new();

        for action in actions_to_process {
            match action {
                PlayerAction::Descend => {
                    if let Some(player_entity) = find_player_entity(&ecs_world.world) {
                        // Get player position as owned values
                        let player_pos_x;
                        let player_pos_y;
                        let player_pos_z;

                        match ecs_world.world.get::<&Position>(player_entity) {
                            Ok(pos) => {
                                player_pos_x = pos.x;
                                player_pos_y = pos.y;
                                player_pos_z = pos.z;
                            }
                            Err(_) => {
                                new_actions.push(action);
                                continue;
                            }
                        }

                        // Check if there's a stairs down tile at player's position
                        let mut on_stairs_down = false;
                        for (_, (pos, tile)) in ecs_world.world.query::<(&Position, &Tile)>().iter()
                        {
                            if pos.x == player_pos_x
                                && pos.y == player_pos_y
                                && pos.z == player_pos_z
                            {
                                if matches!(tile.terrain_type, TerrainType::StairsDown) {
                                    on_stairs_down = true;
                                    break;
                                }
                            }
                        }

                        if on_stairs_down {
                            let old_level = player_pos_z as usize;
                            let new_level = (player_pos_z + 1) as usize;

                            ecs_world.resources.game_state.depth = new_level;

                            // Move player to new level
                            if let Ok(mut pos) = ecs_world.world.get::<&mut Position>(player_entity)
                            {
                                pos.z += 1;
                                pos.x = 10;
                                pos.y = 10;
                            }

                            // Publish LevelChanged event
                            ecs_world.publish_event(GameEvent::LevelChanged {
                                old_level,
                                new_level,
                            });

                            // Add message to game state log
                            ecs_world
                                .resources
                                .game_state
                                .message_log
                                .push("You descend to the next level...".to_string());
                            if ecs_world.resources.game_state.message_log.len() > 10 {
                                ecs_world.resources.game_state.message_log.remove(0);
                            }
                        } else {
                            ecs_world
                                .resources
                                .game_state
                                .message_log
                                .push("You need to stand on stairs to descend.".to_string());
                            if ecs_world.resources.game_state.message_log.len() > 10 {
                                ecs_world.resources.game_state.message_log.remove(0);
                            }
                            new_actions.push(action);
                        }
                    } else {
                        new_actions.push(action);
                    }
                }
                PlayerAction::Ascend => {
                    if let Some(player_entity) = find_player_entity(&ecs_world.world) {
                        // Get player position as owned value
                        let player_pos_x;
                        let player_pos_y;
                        let player_pos_z;

                        match ecs_world.world.get::<&Position>(player_entity) {
                            Ok(pos) => {
                                player_pos_x = pos.x;
                                player_pos_y = pos.y;
                                player_pos_z = pos.z;
                            }
                            Err(_) => {
                                new_actions.push(action);
                                continue;
                            }
                        }

                        // Check if there's a stairs up tile at player's position
                        let mut on_stairs_up = false;
                        for (_, (pos, tile)) in ecs_world.world.query::<(&Position, &Tile)>().iter()
                        {
                            if pos.x == player_pos_x
                                && pos.y == player_pos_y
                                && pos.z == player_pos_z
                            {
                                if matches!(tile.terrain_type, TerrainType::StairsUp) {
                                    on_stairs_up = true;
                                    break;
                                }
                            }
                        }

                        if on_stairs_up {
                            if player_pos_z > 0 {
                                let old_level = player_pos_z as usize;
                                let new_level = (player_pos_z - 1) as usize;

                                // Move player to new level
                                if let Ok(mut pos) =
                                    ecs_world.world.get::<&mut Position>(player_entity)
                                {
                                    pos.z -= 1;
                                    pos.x = 10;
                                    pos.y = 10;
                                }

                                ecs_world.resources.game_state.depth = new_level;

                                // Publish LevelChanged event
                                ecs_world.publish_event(GameEvent::LevelChanged {
                                    old_level,
                                    new_level,
                                });

                                // Add message to game state log
                                ecs_world
                                    .resources
                                    .game_state
                                    .message_log
                                    .push("You ascend to the previous level...".to_string());
                                if ecs_world.resources.game_state.message_log.len() > 10 {
                                    ecs_world.resources.game_state.message_log.remove(0);
                                }
                            } else {
                                // Player is at dungeon level 0, can't go higher
                                let message = "You can't go up from here.".to_string();

                                ecs_world.resources.game_state.message_log.push(message);
                                if ecs_world.resources.game_state.message_log.len() > 10 {
                                    ecs_world.resources.game_state.message_log.remove(0);
                                }
                            }
                        } else {
                            let message = "You need to stand on stairs to ascend.".to_string();
                            new_actions.push(action);

                            ecs_world.resources.game_state.message_log.push(message);
                            if ecs_world.resources.game_state.message_log.len() > 10 {
                                ecs_world.resources.game_state.message_log.remove(0);
                            }
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
        ecs_world.resources.input_buffer.pending_actions = new_actions;

        SystemResult::Continue
    }

    /// Generate a basic dungeon level
    fn generate_level(&mut self, world: &mut World, resources: &mut Resources, level: i32) {
        // Prefer using dungeon::Dungeon if present
        if let Some(dungeon) = crate::ecs::get_dungeon_clone(world) {
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

            // Populate tiles from dungeon level data
            let lvl = &dungeon.levels[dungeon.depth - 1];
            for tile in &lvl.tiles {
                let terrain = match &tile.info.terrain_type {
                    dungeon::level::tiles::TerrainType::Floor => TerrainType::Floor,
                    dungeon::level::tiles::TerrainType::Wall => TerrainType::Wall,
                    dungeon::level::tiles::TerrainType::Door(_) => TerrainType::Door,
                    dungeon::level::tiles::TerrainType::Stair(dir) => match dir {
                        dungeon::level::tiles::StairDirection::Up => TerrainType::StairsUp,
                        dungeon::level::tiles::StairDirection::Down => TerrainType::StairsDown,
                    },
                    dungeon::level::tiles::TerrainType::Water => TerrainType::Water,
                    dungeon::level::tiles::TerrainType::Trap(_) => TerrainType::Trap,
                    dungeon::level::tiles::TerrainType::Special => TerrainType::Empty,
                    dungeon::level::tiles::TerrainType::Grass => TerrainType::Floor,
                };

                world.spawn((
                    Position::new(tile.x, tile.y, level),
                    Tile {
                        terrain_type: terrain.clone(),
                        is_passable: tile.info.passable,
                        blocks_sight: tile.info.blocks_sight,
                        has_items: lvl.items.iter().any(|i| i.x == tile.x && i.y == tile.y),
                        has_monster: lvl.enemies.iter().any(|e| e.x == tile.x && e.y == tile.y),
                    },
                    Renderable {
                        symbol: match terrain {
                            TerrainType::Floor => '.',
                            TerrainType::Wall => '#',
                            TerrainType::Door => '+',
                            TerrainType::StairsDown => '>',
                            TerrainType::Water => '~',
                            TerrainType::Trap => '^',
                            _ => ' ',
                        },
                        fg_color: Color::White,
                        bg_color: Some(Color::Black),
                        order: 0,
                    },
                ));
            }

            // Spawn enemies and items from level
            for enemy in &lvl.enemies {
                world.spawn((
                    Position::new(enemy.x, enemy.y, level),
                    Actor {
                        name: enemy.name().to_string(),
                        faction: Faction::Enemy,
                    },
                    Renderable {
                        symbol: enemy.symbol,
                        fg_color: Color::Green,
                        bg_color: Some(Color::Black),
                        order: 5,
                    },
                    Stats {
                        hp: enemy.hp,
                        max_hp: enemy.max_hp,
                        attack: enemy.attack,
                        defense: enemy.defense,
                        accuracy: 70,
                        evasion: 10,
                        level: enemy.attack_range as u32,
                        experience: enemy.exp_value,
                        class: None,
                    },
                    Energy {
                        current: 100,
                        max: 100,
                        regeneration_rate: 1,
                    },
                ));
            }

            for item in &lvl.items {
                world.spawn((
                    Position::new(item.x, item.y, level),
                    Renderable {
                        symbol: '!',
                        fg_color: Color::Red,
                        bg_color: Some(Color::Black),
                        order: 1,
                    },
                    ECSItem {
                        name: item.name.clone(),
                        item_type: ItemType::Consumable {
                            effect: ConsumableEffect::Healing { amount: 10 },
                        },
                        value: 5,
                        identified: true,
                        quantity: 1,
                        level: 0,
                        cursed: false,
                        charges: None,
                        detailed_data: None,
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
            return;
        }

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
                    symbol: if x == 5 || x == 24 || y == 5 || y == 24 {
                        '#'
                    } else {
                        '.'
                    },
                    fg_color: if x == 5 || x == 24 || y == 5 || y == 24 {
                        Color::Gray
                    } else {
                        Color::White
                    },
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
        if level > 0 {
            // Add content to levels other than 0
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
                    class: None,
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
                    quantity: 1,
                    level: 0,
                    cursed: false,
                    charges: None,
                    detailed_data: None,
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

// ========== 新增：HungerSystem（饱食度系统）==========

/// 饥饿系统：处理玩家的饱食度变化
/// 遵循 SPD 标准：每20回合减少1点饱食度
/// 饱食度为0时每回合掉血
pub struct HungerSystem;

impl HungerSystem {
    /// 带事件总线的运行方法
    pub fn run_with_events(ecs_world: &mut ECSWorld) -> SystemResult {
        use crate::event_bus::GameEvent;

        // 获取当前总回合数
        let current_turn = ecs_world.resources.clock.turn_count;

        // 收集需要处理的实体信息（避免借用冲突）
        let mut entities_to_process = Vec::new();

        for (entity, (hunger, stats)) in ecs_world.world.query::<(&Hunger, &Stats)>().iter() {
            let is_player = ecs_world.world.get::<&Player>(entity).is_ok();
            entities_to_process.push((entity, hunger.clone(), stats.clone(), is_player));
        }

        // 处理每个实体
        for (entity, mut hunger, mut stats, is_player) in entities_to_process {
            // 每20回合减少1点饱食度
            if current_turn > 0 && (current_turn - hunger.last_hunger_turn) >= 20 {
                let old_satiety = hunger.satiety;
                hunger.satiety = hunger.satiety.saturating_sub(1);
                hunger.last_hunger_turn = current_turn;

                // 发布饥饿度变化事件
                ecs_world.publish_event(GameEvent::HungerChanged {
                    entity: entity.id() as u32,
                    old_satiety,
                    new_satiety: hunger.satiety,
                });

                // 饥饿状态处理
                if hunger.is_starving() {
                    // 发布挨饿事件
                    ecs_world.publish_event(GameEvent::PlayerStarving {
                        entity: entity.id() as u32,
                    });

                    // 饥饿致死：每回合掉1血
                    let damage = 1;
                    stats.hp = stats.hp.saturating_sub(damage);

                    // 发布饥饿伤害事件
                    ecs_world.publish_event(GameEvent::StarvationDamage {
                        entity: entity.id() as u32,
                        damage,
                    });

                    // 检查玩家是否死亡
                    if stats.hp == 0 && is_player {
                        ecs_world.publish_event(GameEvent::GameOver {
                            reason: "死于饥饿".to_string(),
                        });

                        // 更新组件
                        let _ = ecs_world.world.insert(entity, (hunger, stats));
                        return SystemResult::Stop;
                    }
                } else if hunger.is_hungry() {
                    // 发布饥饿警告事件（每40回合一次，避免刷屏）
                    if current_turn % 40 == 0 {
                        ecs_world.publish_event(GameEvent::PlayerHungry {
                            entity: entity.id() as u32,
                            satiety: hunger.satiety,
                        });
                    }
                }

                // 更新组件
                let _ = ecs_world.world.insert(entity, (hunger, stats));
            }
        }

        SystemResult::Continue
    }
}

impl System for HungerSystem {
    fn name(&self) -> &str {
        "HungerSystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // 获取当前总回合数
        let current_turn = resources.clock.turn_count;

        // 处理所有拥有 Hunger 组件的实体（主要是玩家）
        for (entity, (hunger, stats)) in world.query::<(&mut Hunger, &mut Stats)>().iter() {
            // 每20回合减少1点饱食度
            if current_turn > 0 && (current_turn - hunger.last_hunger_turn) >= 20 {
                hunger.satiety = hunger.satiety.saturating_sub(1);
                hunger.last_hunger_turn = current_turn;

                // 饥饿状态处理
                if hunger.is_starving() {
                    // 饥饿致死：每回合掉1血
                    stats.hp = stats.hp.saturating_sub(1);
                    resources
                        .game_state
                        .message_log
                        .push("你正在饿死！".to_string());

                    // 检查玩家是否死亡
                    if stats.hp == 0 && world.get::<&Player>(entity).is_ok() {
                        resources.game_state.game_state = GameStatus::GameOver {
                            reason: GameOverReason::Died("死亡"),
                        };
                        resources
                            .game_state
                            .message_log
                            .push("你死于饥饿...".to_string());
                        return SystemResult::Stop;
                    }
                } else if hunger.is_hungry() {
                    // 饥饿警告状态
                    if current_turn % 40 == 0 {
                        // 每40回合提示一次
                        resources
                            .game_state
                            .message_log
                            .push("你感到饥饿...".to_string());
                    }
                }
            }
        }

        SystemResult::Continue
    }
}

/// 渲染系统
///
/// 负责协调所有渲染组件，但由于实际渲染由RatatuiRenderer处理，
/// 这个系统主要用于标记渲染状态和清理渲染缓存。
pub struct RenderingSystem;

impl System for RenderingSystem {
    fn name(&self) -> &str {
        "RenderingSystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        // 标记视锥为dirty（如果需要重新计算FOV）
        for (_, viewshed) in world.query::<&mut Viewshed>().iter() {
            if viewshed.dirty {
                // FOVSystem会处理实际的视锥计算
                // 这里只是确保dirty状态被记录
            }
        }

        // 清理过期的渲染缓存（如果实现的话）
        // 这里可以添加渲染缓存清理逻辑

        // 更新渲染相关的资源状态
        resources.game_state.frame_count = resources.game_state.frame_count.wrapping_add(1);

        SystemResult::Continue
    }
}

/// 菜单系统
///
/// 处理所有菜单相关的动作，包括菜单导航、状态切换等。
pub struct MenuSystem;

impl System for MenuSystem {
    fn name(&self) -> &str {
        "MenuSystem"
    }

    fn run(&mut self, _world: &mut World, resources: &mut Resources) -> SystemResult {
        // 收集需要处理的菜单动作，避免借用冲突
        let menu_actions: Vec<PlayerAction> = resources
            .input_buffer
            .completed_actions
            .iter()
            .filter(|action| {
                matches!(
                    action,
                    PlayerAction::OpenInventory
                        | PlayerAction::OpenOptions
                        | PlayerAction::OpenHelp
                        | PlayerAction::OpenCharacterInfo
                        | PlayerAction::CloseMenu
                        | PlayerAction::MenuNavigate(_)
                        | PlayerAction::MenuSelect
                        | PlayerAction::MenuBack
                )
            })
            .cloned()
            .collect();

        // 处理收集到的菜单动作
        for action in menu_actions {
            match action {
                PlayerAction::OpenInventory => {
                    resources.game_state.game_state = GameStatus::Inventory { selected_item: 0 };
                }

                PlayerAction::OpenOptions => {
                    resources.game_state.game_state = GameStatus::Options { selected_option: 0 };
                }

                PlayerAction::OpenHelp => {
                    resources.game_state.game_state = GameStatus::Help;
                }

                PlayerAction::OpenCharacterInfo => {
                    resources.game_state.game_state = GameStatus::CharacterInfo;
                }

                PlayerAction::CloseMenu => {
                    match resources.game_state.game_state {
                        GameStatus::ConfirmQuit { return_to, .. } => {
                            // 在确认退出对话框中按 Esc/Backspace 返回到原状态
                            resources.game_state.game_state = match return_to {
                                crate::ecs::ReturnTo::Running => GameStatus::Running,
                                crate::ecs::ReturnTo::MainMenu => {
                                    GameStatus::MainMenu { selected_option: 0 }
                                }
                            };
                        }
                        GameStatus::MainMenu { .. } => {
                            // 在主菜单按下 Esc 不再退出，避免误触直接退出
                            // 保持在主菜单，等待明确的退出动作（如 'q'）
                        }
                        GameStatus::Paused { .. } => {
                            // 在暂停菜单按 Esc 返回游戏
                            resources.game_state.game_state = GameStatus::Running;
                        }
                        GameStatus::Running => {
                            // 游戏中按 Esc 打开暂停菜单
                            resources.game_state.game_state =
                                GameStatus::Paused { selected_option: 0 };
                        }
                        _ => {
                            // 在其他菜单状态，返回游戏或上一级菜单
                            resources.game_state.game_state = GameStatus::Running;
                        }
                    }
                }

                PlayerAction::Quit => {
                    // 触发确认退出对话框
                    let return_to = match resources.game_state.game_state {
                        GameStatus::MainMenu { .. } => crate::ecs::ReturnTo::MainMenu,
                        _ => crate::ecs::ReturnTo::Running,
                    };
                    resources.game_state.game_state = GameStatus::ConfirmQuit {
                        return_to,
                        selected_option: 1, // 默认选中“否”
                    };
                }

                PlayerAction::MenuNavigate(direction) => {
                    self.handle_menu_navigation(resources, &direction);
                }

                PlayerAction::MenuSelect => {
                    self.handle_menu_selection(resources);
                }

                PlayerAction::MenuBack => {
                    self.handle_menu_back(resources);
                }

                _ => {
                    // 其他动作不会被传递到这里
                }
            }
        }

        SystemResult::Continue
    }
}

impl MenuSystem {
    /// 开始新游戏
    pub fn start_new_game(resources: &mut Resources) {
        resources.game_state.game_state = GameStatus::Running;
        resources
            .game_state
            .message_log
            .push("开始新游戏！".to_string());

        // TODO: 这里应该调用游戏世界的初始化
        // 但由于架构限制，可能需要在游戏循环中处理
    }

    /// 处理菜单导航
    fn handle_menu_navigation(&self, resources: &mut Resources, direction: &NavigateDirection) {
        match resources.game_state.game_state {
            GameStatus::MainMenu {
                ref mut selected_option,
            } => {
                // 主菜单导航（5个选项：开始游戏、继续游戏、游戏设置、帮助说明、退出游戏）
                match direction {
                    NavigateDirection::Up => {
                        *selected_option = selected_option.saturating_sub(1);
                    }
                    NavigateDirection::Down => {
                        *selected_option = (*selected_option + 1).min(4);
                    }
                    _ => {}
                }
            }

            GameStatus::Paused {
                ref mut selected_option,
            } => {
                // 暂停菜单导航（6个选项）
                match direction {
                    NavigateDirection::Up => {
                        *selected_option = selected_option.saturating_sub(1);
                    }
                    NavigateDirection::Down => {
                        *selected_option = (*selected_option + 1).min(5);
                    }
                    _ => {}
                }
            }

            GameStatus::Options {
                ref mut selected_option,
            } => {
                // 选项菜单导航
                match direction {
                    NavigateDirection::Up => {
                        *selected_option = selected_option.saturating_sub(1);
                    }
                    NavigateDirection::Down => {
                        *selected_option = (*selected_option + 1).min(4); // 5个选项
                    }
                    _ => {}
                }
            }

            GameStatus::Inventory {
                ref mut selected_item,
            } => {
                // 物品栏导航
                match direction {
                    NavigateDirection::Up => {
                        *selected_item = selected_item.saturating_sub(1);
                    }
                    NavigateDirection::Down => {
                        *selected_item = (*selected_item + 1).min(9); // 最多10格物品栏
                    }
                    _ => {}
                }
            }

            GameStatus::ClassSelection { ref mut cursor } => {
                // 职业选择导航（4个职业：战士、法师、盗贼、女猎手）
                match direction {
                    NavigateDirection::Up => {
                        *cursor = cursor.saturating_sub(1);
                    }
                    NavigateDirection::Down => {
                        *cursor = (*cursor + 1).min(3);
                    }
                    _ => {}
                }
            }

            GameStatus::ConfirmQuit {
                ref mut selected_option,
                ..
            } => {
                // 确认退出对话框的导航：在 0(是)/1(否) 之间切换
                match direction {
                    NavigateDirection::Left | NavigateDirection::Up => {
                        *selected_option = 0;
                    }
                    NavigateDirection::Right | NavigateDirection::Down => {
                        *selected_option = 1;
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }

    /// 处理菜单选择
    fn handle_menu_selection(&self, resources: &mut Resources) {
        match resources.game_state.game_state {
            GameStatus::MainMenu { selected_option } => {
                // 主菜单选择逻辑
                match selected_option {
                    0 => {
                        // 开始新游戏 - 进入职业选择界面
                        resources.game_state.game_state = GameStatus::ClassSelection { cursor: 0 };
                    }
                    1 => {
                        // 继续游戏（TODO: 实现加载存档功能）
                        resources
                            .game_state
                            .message_log
                            .push("继续游戏功能暂未实现".to_string());
                        MenuSystem::start_new_game(resources); // 临时：直接开始新游戏
                    }
                    2 => {
                        // 游戏设置
                        resources.game_state.game_state =
                            GameStatus::Options { selected_option: 0 };
                    }
                    3 => {
                        // 帮助说明
                        resources.game_state.game_state = GameStatus::Help;
                    }
                    4 => {
                        // 退出游戏
                        resources.game_state.game_state = GameStatus::ConfirmQuit {
                            return_to: crate::ecs::ReturnTo::MainMenu,
                            selected_option: 1, // 默认选中"否"
                        };
                    }
                    _ => {}
                }
            }

            GameStatus::Paused { selected_option } => {
                // 暂停菜单选择逻辑
                match selected_option {
                    0 => {
                        // 继续游戏
                        resources.game_state.game_state = GameStatus::Running;
                    }
                    1 => {
                        // 物品栏
                        resources.game_state.game_state =
                            GameStatus::Inventory { selected_item: 0 };
                    }
                    2 => {
                        // 角色信息
                        resources.game_state.game_state = GameStatus::CharacterInfo;
                    }
                    3 => {
                        // 游戏设置
                        resources.game_state.game_state =
                            GameStatus::Options { selected_option: 0 };
                    }
                    4 => {
                        // 帮助说明
                        resources.game_state.game_state = GameStatus::Help;
                    }
                    5 => {
                        // 保存并退出
                        resources.game_state.game_state = GameStatus::ConfirmQuit {
                            return_to: crate::ecs::ReturnTo::MainMenu,
                            selected_option: 1,
                        };
                    }
                    _ => {}
                }
            }

            GameStatus::Options { selected_option } => {
                // 选项菜单选择逻辑
                match selected_option {
                    0 => {
                        // 切换音效
                        resources
                            .game_state
                            .message_log
                            .push("音效切换功能暂未实现".to_string());
                    }
                    1 => {
                        // 切换音乐
                        resources
                            .game_state
                            .message_log
                            .push("音乐切换功能暂未实现".to_string());
                    }
                    2 => {
                        // 按键绑定
                        resources
                            .game_state
                            .message_log
                            .push("按键绑定功能暂未实现".to_string());
                    }
                    3 => {
                        // 显示模式
                        resources
                            .game_state
                            .message_log
                            .push("显示模式切换功能暂未实现".to_string());
                    }
                    4 => {
                        // 语言
                        resources
                            .game_state
                            .message_log
                            .push("语言切换功能暂未实现".to_string());
                    }
                    _ => {}
                }
            }

            GameStatus::Inventory { selected_item } => {
                // 物品栏选择逻辑
                resources.game_state.message_log.push(format!(
                    "选择了物品 #{} (使用功能暂未实现)",
                    selected_item + 1
                ));
            }

            GameStatus::ClassSelection { cursor } => {
                // 职业选择确认
                let class = match cursor {
                    0 => hero::class::Class::Warrior,
                    1 => hero::class::Class::Mage,
                    2 => hero::class::Class::Rogue,
                    3 => hero::class::Class::Huntress,
                    _ => hero::class::Class::Warrior,
                };

                // 存储选中的职业，用于后续初始化
                resources.game_state.selected_class = Some(class.clone());
                resources
                    .game_state
                    .message_log
                    .push(format!("选择了职业：{}", class));

                // 进入游戏状态（实际的初始化将在 game_loop 中处理）
                MenuSystem::start_new_game(resources);
            }

            GameStatus::ConfirmQuit {
                return_to,
                selected_option,
            } => {
                // 确认退出：0=是，1=否
                if selected_option == 0 {
                    // 退出到 GameOver
                    resources.game_state.game_state = GameStatus::GameOver {
                        reason: GameOverReason::Quit,
                    };
                } else {
                    // 返回原状态
                    resources.game_state.game_state = match return_to {
                        crate::ecs::ReturnTo::Running => GameStatus::Running,
                        crate::ecs::ReturnTo::MainMenu => {
                            GameStatus::MainMenu { selected_option: 0 }
                        }
                    };
                }
            }

            _ => {}
        }
    }

    /// 处理菜单返回
    fn handle_menu_back(&self, resources: &mut Resources) {
        match resources.game_state.game_state {
            GameStatus::Help | GameStatus::CharacterInfo => {
                // 从帮助/角色信息返回游戏
                resources.game_state.game_state = GameStatus::Running;
            }

            GameStatus::Options { .. } | GameStatus::Inventory { .. } => {
                // 从选项/物品栏返回游戏
                resources.game_state.game_state = GameStatus::Running;
            }

            GameStatus::ClassSelection { .. } => {
                // 从职业选择返回主菜单
                resources.game_state.game_state = GameStatus::MainMenu { selected_option: 0 };
            }

            _ => {}
        }
    }
}

// ========== Boss 系统 ==========

pub struct BossSystem;

impl System for BossSystem {
    fn name(&self) -> &str {
        "BossSystem"
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) -> SystemResult {
        use crate::ecs::{BossComponent, BossSkillComponent};

        // 收集所有 Boss 实体及其信息
        let boss_data: Vec<(
            Entity,
            combat::boss::BossType,
            combat::boss::BossPhase,
            u32,
            u32,
        )> = world
            .query::<(&BossComponent, &Stats)>()
            .iter()
            .map(|(entity, (boss_comp, stats))| {
                (
                    entity,
                    boss_comp.boss_type.clone(),
                    boss_comp.current_phase.clone(),
                    stats.hp,
                    stats.max_hp,
                )
            })
            .collect();

        // 找到玩家位置
        let player_pos = if let Some(player_entity) = find_player_entity(world) {
            world
                .get::<&Position>(player_entity)
                .ok()
                .map(|p| p.clone())
        } else {
            None
        };

        // 处理每个 Boss
        for (boss_entity, boss_type, current_phase, hp, max_hp) in boss_data {
            // 检查阶段转换
            let hp_percent = hp as f32 / max_hp as f32;
            let new_phase = combat::boss::BossPhase::from_health_percent(hp_percent);

            if new_phase != current_phase {
                // 更新阶段
                if let Ok(mut boss_comp) = world.get::<&mut BossComponent>(boss_entity) {
                    boss_comp.current_phase = new_phase.clone();
                }

                resources.game_state.message_log.push(format!(
                    "{}进入了{:?}阶段！",
                    boss_type.name(),
                    new_phase
                ));
            }

            // Boss AI：选择并使用技能
            if let Some(player_pos) = &player_pos {
                if let Ok(boss_pos) = world.get::<&Position>(boss_entity) {
                    let distance = ((boss_pos.x - player_pos.x).pow(2) as f32
                        + (boss_pos.y - player_pos.y).pow(2) as f32)
                        .sqrt();

                    // 根据 Boss 逻辑决定是否使用技能
                    // 这里简化处理，实际应该检查冷却时间等
                    if distance <= 10.0 {
                        // 在攻击范围内，可能使用技能
                        // 技能逻辑将在 CombatSystem 或专门的 BossSkillSystem 中处理
                    }
                }
            }

            // 更新技能冷却
            if let Ok(mut skill_comp) = world.get::<&mut BossSkillComponent>(boss_entity) {
                skill_comp.cooldowns.tick();
            }
        }

        SystemResult::Continue
    }
}
