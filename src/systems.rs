use crate::ecs::{AI, Actor, Energy, Player, Position, Resources, Viewshed,
                PlayerAction, Direction, Tile, TerrainType, Renderable,
                Faction, Stats, Inventory, ItemSlot, ECSItem, ItemType,
                ConsumableEffect, StatType, GameStatus, Color, ECSWorld,
                AIType};
use crate::event_bus::LogLevel;
use hecs::{Entity, World};
use std::error::Error;

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
                            // Mark action as completed for energy deduction
                            resources.input_buffer.completed_actions.push(PlayerAction::Move(direction));
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
        // 收集需要处理的 AI 实体信息（只克隆最小必要的数据）
        let ai_actions: Vec<(Entity, AIType, Position)> = world
            .query::<(&AI, &Position)>()
            .iter()
            .filter(|(entity, _)| {
                // 过滤掉玩家实体
                world.get::<&Player>(*entity).is_err()
            })
            .map(|(entity, (ai, pos))| {
                // 只克隆 AI 类型和位置，不克隆整个 AI 结构
                (entity, ai.ai_type.clone(), pos.clone())
            })
            .collect();

        // 找到所有玩家的位置（缓存查询结果）
        let player_positions: Vec<(Entity, Position)> = world
            .query::<(&Position, &Player)>()
            .iter()
            .map(|(entity, (pos, _))| (entity, pos.clone()))
            .collect();

        // 处理每个 AI 实体的行为
        for (entity, ai_type, pos) in ai_actions {
            match ai_type {
                AIType::Aggressive => {
                    // 获取 AI 范围（不需要克隆整个 AI 对象）
                    let ai_range = match world.get::<&AI>(entity) {
                        Ok(ai) => ai.range() as f32,
                        Err(_) => continue,
                    };

                    // 查找最近的玩家
                    let mut closest_player = None;
                    for (player_entity, player_pos) in &player_positions {
                        let distance = pos.distance_to(player_pos);
                        if distance <= ai_range {
                            let update = closest_player.map_or(true, |(_, d)| distance < d);
                            if update {
                                closest_player = Some((*player_entity, distance));
                            }
                        }
                    }

                    // 如果找到玩家，向其移动
                    if let Some((player_entity, _)) = closest_player {
                        if let Some(player_pos) = player_positions.iter()
                            .find(|(e, _)| *e == player_entity)
                            .map(|(_, p)| p) {
                            let dx = (player_pos.x - pos.x).signum();
                            let dy = (player_pos.y - pos.y).signum();
                            let _ = Self::attempt_move_to(world, entity, pos.x + dx, pos.y + dy);
                        }
                    }
                }
                AIType::Passive | AIType::Neutral => {
                    // 被动或中立 AI 暂时不做任何事
                }
                AIType::Patrol { .. } => {
                    // 巡逻 AI 的实现（暂时留空）
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
                        let player_name = world.world.get::<&Actor>(player_entity)
                            .map(|a| a.name.clone())
                            .unwrap_or_else(|_| "Player".to_string());

                        // 计算攻击目标位置
                        let attack_pos = Position::new(
                            player_pos.x + target_pos.x,
                            player_pos.y + target_pos.y,
                            player_pos.z
                        );

                        // 查找目标位置的敌人
                        let mut target_info: Option<(Entity, String, u32)> = None;
                        for (entity, (pos, actor, _stats)) in world.world.query::<(&Position, &Actor, &Stats)>().iter() {
                            if actor.faction == Faction::Enemy &&
                               pos.x == attack_pos.x &&
                               pos.y == attack_pos.y &&
                               pos.z == attack_pos.z {
                                // 将 entity 转换为 u32 (使用内部表示)
                                let entity_id = entity.id();
                                target_info = Some((entity, actor.name.clone(), entity_id));
                                break;
                            }
                        }

                        if let Some((target, target_name, target_id)) = target_info {
                            let player_id = player_entity.id();

                            // 发布战斗开始事件
                            world.publish_event(GameEvent::CombatStarted {
                                attacker: player_id,
                                defender: target_id,
                            });

                            // 执行战斗计算 - 克隆 Stats 以避免借用问题
                            let combat_result = {
                                let player_stats = world.world.get::<&Stats>(player_entity).ok().map(|s| (*s).clone());
                                let target_stats = world.world.get::<&Stats>(target).ok().map(|s| (*s).clone());

                                if let (Some(mut temp_player), Some(mut temp_target)) = (player_stats, target_stats) {
                                    let mut attacker = SimpleCombatant::new(&mut temp_player);
                                    let mut defender = SimpleCombatant::new(&mut temp_target);

                                    // 执行战斗
                                    let result = ::combat::Combat::engage(&mut attacker, &mut defender, false);
                                    Some((temp_player.hp, temp_target.hp, result))
                                } else {
                                    None
                                }
                            };

                            if let Some((new_player_hp, new_target_hp, combat_result)) = combat_result {
                                // 计算伤害值
                                let player_old_hp = world.world.get::<&Stats>(player_entity)
                                    .map(|s| s.hp)
                                    .unwrap_or(0);
                                let target_old_hp = world.world.get::<&Stats>(target)
                                    .map(|s| s.hp)
                                    .unwrap_or(0);

                                let damage_to_target = target_old_hp.saturating_sub(new_target_hp);
                                let damage_to_player = player_old_hp.saturating_sub(new_player_hp);

                                // 发布伤害事件
                                if damage_to_target > 0 {
                                    world.publish_event(GameEvent::DamageDealt {
                                        attacker: player_id,
                                        victim: target_id,
                                        damage: damage_to_target,
                                        is_critical: combat_result.logs.iter().any(|log| log.contains("Critical")),
                                    });
                                }

                                if damage_to_player > 0 {
                                    world.publish_event(GameEvent::DamageDealt {
                                        attacker: target_id,
                                        victim: player_id,
                                        damage: damage_to_player,
                                        is_critical: false,
                                    });
                                }

                                // 应用实际伤害
                                if let Ok(mut stats) = world.world.get::<&mut Stats>(player_entity) {
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

                                // 检查目标是否死亡
                                if new_target_hp == 0 {
                                    world.publish_event(GameEvent::EntityDied {
                                        entity: target_id,
                                        entity_name: target_name.clone(),
                                    });
                                    // 移除死亡的实体
                                    let _ = world.world.despawn(target);
                                }

                                // 检查玩家是否死亡
                                if new_player_hp == 0 {
                                    world.publish_event(GameEvent::EntityDied {
                                        entity: player_id,
                                        entity_name: player_name.clone(),
                                    });
                                    world.publish_event(GameEvent::GameOver {
                                        reason: "你被击败了！".to_string(),
                                    });
                                }

                                // 消耗能量
                                if let Ok(mut energy) = world.world.get::<&mut Energy>(player_entity) {
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
                                                    if let Ok(mut stats) = world.get::<&mut Stats>(player_entity) {
                                                        stats.hp = (stats.hp + amount).min(stats.max_hp);
                                                        let message = format!("You drink a {}, healing {} HP.", item.name, amount);
                                                        
                                                        // Add message to game state log (original behavior)
                                                        resources.game_state.message_log.push(message);
                                                        if resources.game_state.message_log.len() > 10 {
                                                            resources.game_state.message_log.remove(0);
                                                        }
                                                    }
                                                }
                                                ConsumableEffect::Damage { amount } => {
                                                    // Apply damage to player (negative effect)
                                                    if let Ok(mut stats) = world.get::<&mut Stats>(player_entity) {
                                                        stats.hp = stats.hp.saturating_sub(*amount);
                                                        let message = format!("You drink a {}, taking {} damage!", item.name, amount);
                                                        
                                                        // Add message to game state log (original behavior)
                                                        resources.game_state.message_log.push(message);
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
                                                        let message = format!("You feel {}!", match stat {
                                                            StatType::Hp => format!("healthier ({})", value),
                                                            StatType::Attack => format!("stronger ({})", value),
                                                            StatType::Defense => format!("tougher ({})", value),
                                                            StatType::Accuracy => format!("more accurate ({})", value),
                                                            StatType::Evasion => format!("more evasive ({})", value),
                                                        });
                                                        
                                                        // Add message to game state log (original behavior)
                                                        resources.game_state.message_log.push(message);
                                                        if resources.game_state.message_log.len() > 10 {
                                                            resources.game_state.message_log.remove(0);
                                                        }
                                                    }
                                                }
                                                ConsumableEffect::Teleport => {
                                                    // Teleport player to random location in level
                                                    if let Ok(mut pos) = world.get::<&mut Position>(player_entity) {
                                                        use rand::Rng;
                                                        // Use proper RNG for random position
                                                        pos.x = 5 + resources.rng.gen_range(0..15); // Random position between 5-19
                                                        pos.y = 5 + resources.rng.gen_range(0..15); // Random position between 5-19
                                                        let message = "You teleport randomly!".to_string();
                                                        
                                                        // Add message to game state log (original behavior)
                                                        resources.game_state.message_log.push(message);
                                                        if resources.game_state.message_log.len() > 10 {
                                                            resources.game_state.message_log.remove(0);
                                                        }
                                                    }
                                                }
                                                ConsumableEffect::Identify => {
                                                    // For now, just add a message
                                                    let message = "You feel more perceptive.".to_string();
                                                    
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
                    let drop_result: Option<(Position, ECSItem, u32)> = if let Some(player_entity) = find_player_entity(world) {
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
                                if let Some(item_to_drop) = inventory.items.remove(slot_index).item {
                                    Some((player_pos.clone(), item_to_drop, player_id))  // Clone the position to get owned value
                                } else {
                                    // Add message to game state log (original behavior)
                                    resources.game_state.message_log.push("No item in this slot to drop.".to_string());
                                    if resources.game_state.message_log.len() > 10 {
                                        resources.game_state.message_log.remove(0);
                                    }
                                    new_actions.push(action);
                                    None
                                }
                            } else {
                                // Add message to game state log (original behavior)
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
                    
                    let mut available_slots = world
                        .get::<&Inventory>(player_entity)
                        .ok()
                        .map(|inventory| inventory.max_slots.saturating_sub(inventory.items.len()))
                        .unwrap_or(0);
                    
                    if available_slots == 0 {
                        resources.game_state.message_log.push("Your inventory is full!".to_string());
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
                        resources.game_state.message_log.push("Your inventory is full!".to_string());
                        if resources.game_state.message_log.len() > 10 {
                            resources.game_state.message_log.remove(0);
                        }
                    }
                }
                if picked_up {
                    let _ = world.despawn(item_entity);
                    resources.game_state.message_log.push(format!("You picked up {}.", item_name));
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
                                if pos.x == player_pos.x && pos.y == player_pos.y && pos.z == player_pos.z {
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
                                resources.game_state.message_log.push("You descend to the next level...".to_string());
                                if resources.game_state.message_log.len() > 10 {
                                    resources.game_state.message_log.remove(0);
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
                                    
                                    let message = "You ascend to the previous level...".to_string();
                                    
                                    // Add message to game state log (original behavior)
                                    resources.game_state.message_log.push(message);
                                    if resources.game_state.message_log.len() > 10 {
                                        resources.game_state.message_log.remove(0);
                                    }
                                    
                                    resources.game_state.depth = (player_pos.z - 1) as usize;
                                    
                                    // Add message to game state log (original behavior)
                                    resources.game_state.message_log.push("You ascend to the previous level...".to_string());
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
                    Actor { name: enemy.name().to_string(), faction: Faction::Enemy },
                    Renderable { symbol: enemy.symbol, fg_color: Color::Green, bg_color: Some(Color::Black), order: 5 },
                    Stats { hp: enemy.hp, max_hp: enemy.max_hp, attack: enemy.attack, defense: enemy.defense, accuracy: 70, evasion: 10, level: enemy.attack_range as u32, experience: enemy.exp_value },
                    Energy { current: 100, max: 100, regeneration_rate: 1 },
                ));
            }

            for item in &lvl.items {
                world.spawn((
                    Position::new(item.x, item.y, level),
                    Renderable { symbol: '!', fg_color: Color::Red, bg_color: Some(Color::Black), order: 1 },
                    ECSItem { name: item.name.clone(), item_type: ItemType::Consumable { effect: ConsumableEffect::Healing { amount: 10 } }, value: 5, identified: true },
                    Tile { terrain_type: TerrainType::Empty, is_passable: true, blocks_sight: false, has_items: true, has_monster: false },
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
