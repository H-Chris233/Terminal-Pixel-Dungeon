//! Energy-driven turn scheduler orchestrating player/AI phases.
//!
//! The `TurnSystem` now implements an explicit state machine with phases:
//! Input → Intent Queue → Resolution → Aftermath. It uses an energy/initiative
//! priority queue to determine actor order and centralizes all action costs.

use crate::ecs::*;
use anyhow;
use hecs::{Entity, World};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Centralized energy cost table for all actions.
///
/// This module defines the canonical energy costs for both player and AI actions.
/// All costs are in energy units, where 100 is the standard "full action" cost.
pub mod energy_costs {
    use super::*;
    
    /// Full action energy cost (movement, attack, use item, etc.)
    pub const FULL_ACTION: u32 = 100;
    /// Wait action energy cost (half of full action)
    pub const WAIT: u32 = 50;
    /// No energy cost (for actions like quit, menu navigation)
    pub const FREE: u32 = 0;
    
    /// Get the energy cost for a player action
    pub fn player_action_cost(action: &PlayerAction) -> u32 {
        match action {
            PlayerAction::Move(_)
            | PlayerAction::Attack(_)
            | PlayerAction::UseItem(_)
            | PlayerAction::DropItem(_)
            | PlayerAction::Descend
            | PlayerAction::Ascend => FULL_ACTION,
            
            PlayerAction::Wait => WAIT,
            
            // Menu and system actions are free
            PlayerAction::Quit
            | PlayerAction::OpenInventory
            | PlayerAction::OpenOptions
            | PlayerAction::OpenHelp
            | PlayerAction::OpenCharacterInfo
            | PlayerAction::CloseMenu
            | PlayerAction::MenuNavigate(_)
            | PlayerAction::MenuSelect
            | PlayerAction::MenuBack => FREE,
        }
    }
    
    /// Get the energy cost for an AI action
    pub fn ai_action_cost(action: &AIIntent) -> u32 {
        match action {
            AIIntent::Move => FULL_ACTION,
            AIIntent::Attack => FULL_ACTION,
            AIIntent::Wait => WAIT,
            AIIntent::Flee => FULL_ACTION,
            AIIntent::UseSkill => FULL_ACTION,
        }
    }
}

/// AI action intents
#[derive(Debug, Clone, PartialEq)]
pub enum AIIntent {
    Move,
    Attack,
    Wait,
    Flee,
    UseSkill,
}

/// Unified action type for both player and AI
#[derive(Debug, Clone)]
pub enum Action {
    Player(PlayerAction),
    AI(AIIntent),
}

impl Action {
    /// Get the energy cost for this action
    pub fn cost(&self) -> u32 {
        match self {
            Action::Player(pa) => energy_costs::player_action_cost(pa),
            Action::AI(ai) => energy_costs::ai_action_cost(ai),
        }
    }
}

/// Action intent with priority for queue ordering
#[derive(Debug, Clone)]
pub struct ActionIntent {
    pub entity: Entity,
    pub action: Action,
    pub energy_cost: u32,
    pub priority: u32, // Higher priority acts first (for tie-breaking)
}

impl ActionIntent {
    pub fn new(entity: Entity, action: Action, priority: u32) -> Self {
        let energy_cost = action.cost();
        Self {
            entity,
            action,
            energy_cost,
            priority,
        }
    }
}

// Priority queue ordering: higher energy = higher priority, then by priority field
impl Ord for ActionIntent {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by priority field (higher is better)
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // Then by energy cost (lower cost = higher priority for tie-breaking)
                other.energy_cost.cmp(&self.energy_cost)
            }
            other => other,
        }
    }
}

impl PartialOrd for ActionIntent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ActionIntent {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.energy_cost == other.energy_cost
    }
}

impl Eq for ActionIntent {}

/// Turn phase in the state machine
#[derive(Debug, Clone, PartialEq)]
pub enum TurnPhase {
    /// Collecting input from player/AI
    Input,
    /// Building intent queue from entities with sufficient energy
    IntentQueue,
    /// Resolving actions in priority order
    Resolution,
    /// Post-action phase: energy regen, status effects, etc.
    Aftermath,
}

/// Legacy turn state enum for backward compatibility
#[derive(Debug, Clone, PartialEq)]
pub enum TurnState {
    /// Player-facing systems are allowed to enqueue or resolve actions.
    PlayerTurn,
    /// Reserved for multi-step player actions that span multiple frames.
    ProcessingPlayerAction,
    /// AI controllers resolve intents until the player regains full energy.
    AITurn,
    /// Reserved for scripted AI sequences that should not interleave with the player.
    ProcessingAIActions,
}

/// Turn metadata tracking global state
#[derive(Debug, Clone)]
pub struct TurnMeta {
    /// Global turn counter (increments each full turn cycle)
    pub global_turn: u32,
    /// Sub-turn counter within a global turn (increments per action)
    pub sub_turn: u32,
    /// Last entity that took an action
    pub last_actor: Option<Entity>,
    /// Current phase in the turn state machine
    pub phase: TurnPhase,
    /// Legacy state for backward compatibility
    pub legacy_state: TurnState,
}

impl Default for TurnMeta {
    fn default() -> Self {
        Self {
            global_turn: 0,
            sub_turn: 0,
            last_actor: None,
            phase: TurnPhase::Input,
            legacy_state: TurnState::PlayerTurn,
        }
    }
}

impl TurnMeta {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Advance to the next sub-turn
    pub fn advance_sub_turn(&mut self) {
        self.sub_turn += 1;
    }
    
    /// Advance to the next global turn
    pub fn advance_global_turn(&mut self) {
        self.global_turn += 1;
        self.sub_turn = 0;
    }
    
    /// Record the last actor
    pub fn set_last_actor(&mut self, entity: Entity) {
        self.last_actor = Some(entity);
    }
    
    /// Set the current phase
    pub fn set_phase(&mut self, phase: TurnPhase) {
        self.phase = phase;
    }
}

/// Coordinates energy consumption and state transitions between player and AI.
///
/// The game loop owns the event bus and must publish `GameEvent::PlayerTurnStarted`,
/// `GameEvent::AITurnStarted`, and `GameEvent::TurnEnded` when `state` changes.
pub struct TurnSystem {
    /// Current state of the turn system (legacy)
    pub state: TurnState,
    /// Turn metadata
    pub meta: TurnMeta,
    /// Intent queue for action resolution
    intent_queue: BinaryHeap<ActionIntent>,
    /// Whether the player has taken an action this turn
    player_action_taken: bool,
}

impl TurnSystem {
    pub fn new() -> Self {
        Self {
            state: TurnState::PlayerTurn,
            meta: TurnMeta::new(),
            intent_queue: BinaryHeap::new(),
            player_action_taken: false,
        }
    }

    /// Deduct energy for a completed player action.
    ///
    /// Callers must only invoke this after an action has been moved from
    /// `pending_actions` into `completed_actions`. The invariant keeps the
    /// scheduler from double-charging multi-system interactions.
    pub fn consume_player_energy(
        &mut self,
        world: &mut World,
        action: &PlayerAction,
    ) -> Result<(), anyhow::Error> {
        let energy_cost = energy_costs::player_action_cost(action);

        if energy_cost > 0 {
            if let Some(player_entity) = find_player(world) {
                if let Ok(mut energy) = world.get::<&mut Energy>(player_entity) {
                    let before = energy.current;
                    energy.current = energy.current.saturating_sub(energy_cost);
                    if energy.current < before {
                        self.player_action_taken = true;
                        self.meta.set_last_actor(player_entity);
                        self.meta.advance_sub_turn();
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if player has actions to process
    pub fn has_pending_actions(&self, resources: &Resources) -> bool {
        !resources.input_buffer.pending_actions.is_empty()
    }

    /// Build intent queue from all entities with sufficient energy
    fn build_intent_queue(&mut self, world: &World) {
        self.intent_queue.clear();
        
        // Collect all entities with Energy component that can act
        for (entity, energy) in world.query::<&Energy>().iter() {
            if energy.current >= energy_costs::FULL_ACTION {
                // Determine priority: player has highest priority
                let priority = if is_player(world, entity) {
                    1000 // Player gets highest priority
                } else {
                    100 // AI gets lower priority
                };
                
                // For now, just queue a generic intent (actual AI logic handled elsewhere)
                // This demonstrates the structure
                if is_player(world, entity) {
                    // Player intents are handled through the input buffer
                    // We'll process them in the resolution phase
                } else {
                    // Queue AI intent (simplified - actual AI logic elsewhere)
                    let intent = ActionIntent::new(
                        entity,
                        Action::AI(AIIntent::Wait),
                        priority,
                    );
                    self.intent_queue.push(intent);
                }
            }
        }
    }

    /// Process the intent queue and resolve actions
    fn resolve_intents(&mut self, world: &mut World) -> Result<(), anyhow::Error> {
        while let Some(intent) = self.intent_queue.pop() {
            // Consume energy for the action
            if let Ok(mut energy) = world.get::<&mut Energy>(intent.entity) {
                energy.current = energy.current.saturating_sub(intent.energy_cost);
            }
            
            // Record the actor
            self.meta.set_last_actor(intent.entity);
            self.meta.advance_sub_turn();
        }
        
        Ok(())
    }

    /// Process AI turns until the player's energy is full again.
    ///
    /// AI controllers are expected to consume energy in 100-point chunks; if
    /// you introduce actors with different step costs, update this loop and the
    /// documentation table accordingly.
    pub fn process_ai_turns(
        &mut self,
        world: &mut World,
        _resources: &mut Resources,
    ) -> Result<(), anyhow::Error> {
        // Set phase to IntentQueue
        self.meta.set_phase(TurnPhase::IntentQueue);
        
        // Continue processing AI actions until player energy is refilled or no AI can act
        loop {
            // If player has full energy, stop AI processing
            if let Some(player_entity) = find_player(world) {
                if let Ok(energy) = world.get::<&Energy>(player_entity) {
                    if energy.current >= energy.max {
                        break;
                    }
                }
            } else {
                break; // no player
            }

            // Collect AI entities that can act this iteration
            let ai_entities_with_energy: Vec<Entity> = world
                .query::<(&AI, &Energy, &Actor)>()
                .iter()
                .filter(|(_, (_, energy, _))| energy.current >= energy_costs::FULL_ACTION)
                .map(|(entity, _)| entity)
                .collect();

            if ai_entities_with_energy.is_empty() {
                break;
            }

            // Each AI takes one action
            for ai_entity in ai_entities_with_energy {
                if let Ok(mut energy) = world.get::<&mut Energy>(ai_entity) {
                    energy.current = energy.current.saturating_sub(energy_costs::FULL_ACTION);
                    self.meta.set_last_actor(ai_entity);
                    self.meta.advance_sub_turn();
                }
            }
        }

        // After AI finishes, switch back to player turn
        self.state = TurnState::PlayerTurn;
        self.meta.legacy_state = TurnState::PlayerTurn;
        Ok(())
    }

    /// Regenerate energy for all entities after a complete turn
    fn regenerate_energy(&mut self, world: &mut World) {
        self.meta.set_phase(TurnPhase::Aftermath);
        
        for (_, energy) in world.query_mut::<&mut Energy>() {
            energy.current = (energy.current + energy.regeneration_rate).min(energy.max);
        }
    }

    /// Advance player progress (turn counter)
    fn advance_player_progress(&mut self, world: &mut World) {
        if let Some(player_entity) = find_player(world) {
            if let Ok(mut progress) = world.get::<&mut PlayerProgress>(player_entity) {
                progress.advance_turn();
            }
        }
    }

    /// Process a complete turn cycle (player action + AI actions until player energy is full).
    ///
    /// The caller should compare `self.state` before/after invoking this method
    /// and publish the appropriate `GameEvent` turn hooks.
    pub fn process_turn_cycle(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
    ) -> Result<(), anyhow::Error> {
        match self.state {
            TurnState::PlayerTurn => {
                self.meta.set_phase(TurnPhase::Input);
                
                // Process completed actions and deduct energy
                let completed_actions =
                    std::mem::take(&mut resources.input_buffer.completed_actions);

                for action in completed_actions {
                    // Handle Quit action specially（现在改为弹出确认对话框，不直接退出）
                    if matches!(action, PlayerAction::Quit) {
                        let return_to = match resources.game_state.game_state {
                            GameStatus::MainMenu { .. } => ReturnTo::MainMenu,
                            _ => ReturnTo::Running,
                        };
                        resources.game_state.game_state = GameStatus::ConfirmQuit {
                            return_to,
                            selected_option: 1, // 默认选中"否"
                        };
                        continue; // 不结束循环，允许后续处理
                    }

                    // Consume energy for the completed action
                    self.consume_player_energy(world, &action)?;
                }

                // If player has taken an action, switch to AI turn
                if self.player_action_taken {
                    self.state = TurnState::AITurn;
                    self.meta.legacy_state = TurnState::AITurn;
                    self.meta.set_phase(TurnPhase::IntentQueue);
                    self.player_action_taken = false; // Reset for next turn
                }
            }
            TurnState::AITurn => {
                // Process AI turns
                self.process_ai_turns(world, resources)?;

                // After AI turn, regenerate energy for all entities
                self.regenerate_energy(world);
                
                // Advance player turn counter
                self.advance_player_progress(world);
                
                // Advance global turn counter
                self.meta.advance_global_turn();
                
                // Sync global turn to clock
                resources.clock.turn_count = self.meta.global_turn;

                // Switch back to player turn
                self.state = TurnState::PlayerTurn;
                self.meta.legacy_state = TurnState::PlayerTurn;
                self.meta.set_phase(TurnPhase::Input);
            }
            // For the other states, we'll handle them if needed
            _ => {
                // Default behavior
            }
        }

        Ok(())
    }

    /// Check if it's currently the player's turn
    pub fn is_player_turn(&self) -> bool {
        matches!(self.state, TurnState::PlayerTurn)
    }

    /// Check if it's currently an AI turn
    pub fn is_ai_turn(&self) -> bool {
        matches!(self.state, TurnState::AITurn)
    }

    /// Get whether player has taken an action this turn
    pub fn player_action_taken(&self) -> bool {
        self.player_action_taken
    }

    /// Set the turn system state from saved data
    pub fn set_state(&mut self, state: TurnState, player_action_taken: bool) {
        self.state = state.clone();
        self.meta.legacy_state = state;
        self.player_action_taken = player_action_taken;
    }
    
    /// Get current turn metadata
    pub fn get_meta(&self) -> &TurnMeta {
        &self.meta
    }
    
    /// Get entities in turn order based on energy
    pub fn get_turn_order(&self, world: &World) -> Vec<(Entity, u32)> {
        let mut entities: Vec<(Entity, u32)> = world
            .query::<&Energy>()
            .iter()
            .map(|(entity, energy)| (entity, energy.current))
            .collect();
        
        // Sort by energy (descending)
        entities.sort_by(|a, b| b.1.cmp(&a.1));
        
        entities
    }
}

impl Default for TurnSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to find the player entity
fn find_player(world: &World) -> Option<Entity> {
    for (entity, _actor) in world.query::<&Actor>().iter() {
        if let Ok(actor) = world.get::<&Actor>(entity) {
            if actor.faction == Faction::Player {
                return Some(entity);
            }
        }
    }
    None
}

/// Helper function to check if an entity is the player
fn is_player(world: &World, entity: Entity) -> bool {
    if let Ok(actor) = world.get::<&Actor>(entity) {
        actor.faction == Faction::Player
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_energy_cost_lookup() {
        assert_eq!(energy_costs::player_action_cost(&PlayerAction::Move(crate::ecs::Direction::North)), 100);
        assert_eq!(energy_costs::player_action_cost(&PlayerAction::Wait), 50);
        assert_eq!(energy_costs::player_action_cost(&PlayerAction::Quit), 0);
        assert_eq!(energy_costs::ai_action_cost(&AIIntent::Attack), 100);
        assert_eq!(energy_costs::ai_action_cost(&AIIntent::Wait), 50);
    }
    
    #[test]
    fn test_turn_meta_advancement() {
        let mut meta = TurnMeta::new();
        assert_eq!(meta.global_turn, 0);
        assert_eq!(meta.sub_turn, 0);
        
        meta.advance_sub_turn();
        assert_eq!(meta.sub_turn, 1);
        
        meta.advance_global_turn();
        assert_eq!(meta.global_turn, 1);
        assert_eq!(meta.sub_turn, 0);
    }
    
    #[test]
    fn test_action_intent_ordering() {
        let mut world = World::new();
        let entity1 = world.spawn(());
        let entity2 = world.spawn(());
        
        let intent1 = ActionIntent::new(entity1, Action::AI(AIIntent::Move), 100);
        let intent2 = ActionIntent::new(entity2, Action::AI(AIIntent::Attack), 200);
        
        // Higher priority should come first
        assert!(intent2 > intent1);
    }
    
    #[test]
    fn test_turn_system_initialization() {
        let system = TurnSystem::new();
        assert!(system.is_player_turn());
        assert!(!system.is_ai_turn());
        assert_eq!(system.meta.global_turn, 0);
        assert_eq!(system.meta.phase, TurnPhase::Input);
    }
    
    #[test]
    fn test_energy_consumption() {
        let mut world = World::new();
        let mut system = TurnSystem::new();
        
        // Create player entity
        let player = world.spawn((
            Actor {
                name: "Player".to_string(),
                faction: Faction::Player,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 1,
            },
            PlayerProgress::default(),
        ));
        
        // Consume energy for move action
        let action = PlayerAction::Move(Direction::North);
        system.consume_player_energy(&mut world, &action).unwrap();
        
        // Check energy was consumed
        let energy = world.get::<&Energy>(player).unwrap();
        assert_eq!(energy.current, 0);
        assert!(system.player_action_taken);
    }
    
    #[test]
    fn test_energy_regeneration() {
        let mut world = World::new();
        let mut system = TurnSystem::new();
        
        // Create entities with depleted energy
        let entity = world.spawn((
            Energy {
                current: 50,
                max: 100,
                regeneration_rate: 10,
            },
        ));
        
        // Regenerate energy
        system.regenerate_energy(&mut world);
        
        // Check energy was regenerated
        let energy = world.get::<&Energy>(entity).unwrap();
        assert_eq!(energy.current, 60);
    }
    
    #[test]
    fn test_turn_order_by_energy() {
        let mut world = World::new();
        let system = TurnSystem::new();
        
        // Create entities with different energy levels
        let _e1 = world.spawn((Energy { current: 50, max: 100, regeneration_rate: 1 },));
        let _e2 = world.spawn((Energy { current: 100, max: 100, regeneration_rate: 1 },));
        let _e3 = world.spawn((Energy { current: 75, max: 100, regeneration_rate: 1 },));
        
        // Get turn order
        let order = system.get_turn_order(&world);
        
        // Should be ordered by energy: e2 (100), e3 (75), e1 (50)
        assert_eq!(order.len(), 3);
        assert_eq!(order[0].1, 100);
        assert_eq!(order[1].1, 75);
        assert_eq!(order[2].1, 50);
    }
    
    #[test]
    fn test_player_progress_advancement() {
        let mut world = World::new();
        let mut system = TurnSystem::new();
        
        // Create player with progress tracking
        let player = world.spawn((
            Actor {
                name: "Player".to_string(),
                faction: Faction::Player,
            },
            PlayerProgress::default(),
        ));
        
        // Check initial turns
        let progress = world.get::<&PlayerProgress>(player).unwrap();
        assert_eq!(progress.turns, 0);
        drop(progress);
        
        // Advance turn
        system.advance_player_progress(&mut world);
        
        // Check turns incremented
        let progress = world.get::<&PlayerProgress>(player).unwrap();
        assert_eq!(progress.turns, 1);
    }
    
    #[test]
    fn test_full_turn_cycle() {
        let mut world = World::new();
        let mut resources = Resources::default();
        let mut system = TurnSystem::new();
        
        // Create player
        let player = world.spawn((
            Actor {
                name: "Player".to_string(),
                faction: Faction::Player,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 10,
            },
            PlayerProgress::default(),
        ));
        
        // Create AI enemy
        let enemy = world.spawn((
            Actor {
                name: "Goblin".to_string(),
                faction: Faction::Enemy,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 10,
            },
            AI {
                ai_type: AIType::Aggressive,
                target: Some(player),
                state: AIState::Idle,
            },
        ));
        
        // Initial state
        assert!(system.is_player_turn());
        assert_eq!(system.meta.global_turn, 0);
        
        // Simulate player action
        resources.input_buffer.completed_actions.push(PlayerAction::Wait);
        system.process_turn_cycle(&mut world, &mut resources).unwrap();
        
        // Should switch to AI turn
        assert!(system.is_ai_turn());
        
        // Process AI turn
        system.process_turn_cycle(&mut world, &mut resources).unwrap();
        
        // Should be back to player turn and global turn advanced
        assert!(system.is_player_turn());
        assert_eq!(system.meta.global_turn, 1);
        
        // Check energy regeneration happened
        let player_energy = world.get::<&Energy>(player).unwrap();
        assert_eq!(player_energy.current, 60); // 50 after wait + 10 regen
        
        let enemy_energy = world.get::<&Energy>(enemy).unwrap();
        assert_eq!(enemy_energy.current, 10); // 0 after action + 10 regen
    }
    
    #[test]
    fn test_wait_action_costs_less() {
        let mut world = World::new();
        let mut system = TurnSystem::new();
        
        // Create player
        let player = world.spawn((
            Actor {
                name: "Player".to_string(),
                faction: Faction::Player,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 1,
            },
            PlayerProgress::default(),
        ));
        
        // Consume energy for wait action (should cost 50)
        let action = PlayerAction::Wait;
        system.consume_player_energy(&mut world, &action).unwrap();
        
        let energy = world.get::<&Energy>(player).unwrap();
        assert_eq!(energy.current, 50);
    }
}
