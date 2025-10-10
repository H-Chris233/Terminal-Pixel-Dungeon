//! Turn-based system implementation for the game.

use crate::ecs::*;
use anyhow;
use hecs::{Entity, World};

/// Trait for entities that can take turns
pub trait TurnTaker {
    /// Returns the energy cost to perform an action
    fn action_cost(&self) -> u32;
    /// Returns the energy gained per turn when not taking actions
    fn passive_regen(&self) -> u32;
    /// Returns the energy gained per turn when taking actions
    fn active_regen(&self) -> u32;
}

#[derive(Debug, Clone, PartialEq)]
pub enum TurnState {
    PlayerTurn,
    ProcessingPlayerAction,
    AITurn,
    ProcessingAIActions,
}

/// System to manage turn-based game flow
pub struct TurnSystem {
    /// Current state of the turn system
    pub state: TurnState,
    /// Whether the player has taken an action this turn
    player_action_taken: bool,
}

impl TurnSystem {
    pub fn new() -> Self {
        Self {
            state: TurnState::PlayerTurn,
            player_action_taken: false,
        }
    }

    /// Process all pending player actions and advance the turn
    pub fn process_player_actions(&mut self, world: &mut World, resources: &mut Resources) -> Result<(), anyhow::Error> {
        // Process all pending player actions
        let mut actions_to_process = std::mem::take(&mut resources.input_buffer.pending_actions);
        
        for action in actions_to_process.drain(..) {
            match action {
                PlayerAction::Move(_) | 
                PlayerAction::Attack(_) | 
                PlayerAction::UseItem(_) | 
                PlayerAction::DropItem(_) => {
                    // Any action that moves or interacts costs energy
                    if let Some(player_entity) = find_player(world) {
                        if let Ok(mut energy) = world.get::<&mut Energy>(player_entity) {
                            energy.current = energy.current.saturating_sub(100); // Action cost
                        }
                    }
                }
                PlayerAction::Wait => {
                    // Wait action still costs some energy but less than a full action
                    if let Some(player_entity) = find_player(world) {
                        if let Ok(mut energy) = world.get::<&mut Energy>(player_entity) {
                            energy.current = energy.current.saturating_sub(50); // Wait cost
                        }
                    }
                }
                PlayerAction::Descend | 
                PlayerAction::Ascend => {
                    // Stairs actions cost energy too
                    if let Some(player_entity) = find_player(world) {
                        if let Ok(mut energy) = world.get::<&mut Energy>(player_entity) {
                            energy.current = energy.current.saturating_sub(100); // Action cost
                        }
                    }
                }
                PlayerAction::Quit => {
                    resources.game_state.game_state = GameStatus::GameOver;
                    return Ok(());
                }
            }
        }

        // Put back any unprocessed actions
        resources.input_buffer.pending_actions = actions_to_process;
        
        // Check if player has enough energy for another action
        if let Some(player_entity) = find_player(world) {
            if let Ok(energy) = world.get::<&Energy>(player_entity) {
                if energy.current < 100 {
                    // Player has used their action, switch to AI turns
                    // The state is already managed in process_player_actions
                }
            }
        }

        Ok(())
    }

    /// Process AI turns until the player's energy is full again
    pub fn process_ai_turns(&mut self, world: &mut World, _resources: &mut Resources) -> Result<(), anyhow::Error> {
        // Process AI actions for all entities that have enough energy
        let ai_entities_with_energy: Vec<_> = world
            .query::<(&AI, &Energy, &Actor)>()
            .iter()
            .filter(|(_, (_, energy, _))| energy.current >= 100)
            .map(|(entity, (_, _, actor))| (entity, actor.name.clone()))
            .collect();
        
        let mut ai_actions_taken = false;
        
        for (ai_entity, actor_name) in ai_entities_with_energy {
            // Perform AI action (this would typically include moving, attacking, etc.)
            // For now, we'll just reduce their energy to show they took an action
            if let Ok(mut energy) = world.get::<&mut Energy>(ai_entity) {
                energy.current = energy.current.saturating_sub(100); // Cost per action
                ai_actions_taken = true;
            }
        }

        // If no AI actions were taken, immediately switch back to player
        if !ai_actions_taken {
            self.state = TurnState::PlayerTurn;
        }

        Ok(())
    }

    /// Regenerate energy for all entities after a complete turn
    fn regenerate_energy(&self, world: &mut World) {
        for (_, mut energy) in world.query_mut::<&mut Energy>() {
            // In a turn-based system, entities typically regain full energy after each turn
            // except the player who is only full when they have completed their actions
            energy.current = energy.max;
        }
    }

    /// Process a complete turn cycle (player action + AI actions until player energy is full)
    pub fn process_turn_cycle(&mut self, world: &mut World, resources: &mut Resources) -> Result<(), anyhow::Error> {
        match self.state {
            TurnState::PlayerTurn => {
                // Check for pending actions
                if !resources.input_buffer.pending_actions.is_empty() {
                    self.process_player_actions(world, resources)?;
                    if self.player_action_taken {
                        // After player action, go to AI turn
                        self.state = TurnState::AITurn;
                        self.player_action_taken = false; // Reset for next turn
                    }
                }
            }
            TurnState::AITurn => {
                // Process AI turns
                self.process_ai_turns(world, resources)?;
                
                // After AI turn, regenerate energy for all entities
                self.regenerate_energy(world);
                
                // Switch back to player turn
                self.state = TurnState::PlayerTurn;
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