//! Energy-driven turn scheduler orchestrating player/AI phases.
//!
//! The `TurnSystem` consumes actions that upstream systems mark as
//! `completed`, applies the canonical energy costs defined here, and flips
//! between `PlayerTurn` and `AITurn` states until the player regains full
//! energy. See `docs/turn_system.md` for a full architecture guide.

use crate::ecs::*;
use anyhow;
use hecs::{Entity, World};

/// Energy cost constants consumed by the scheduler.
///
/// Keep this list in sync with `docs/turn_system.md` so that gameplay,
/// documentation, and tooling share the same source of truth.
pub mod energy_costs {
    /// Full action energy cost (movement, attack, use item, etc.)
    pub const FULL_ACTION: u32 = 100;
    /// Wait action energy cost (half of full action)
    pub const WAIT: u32 = 50;
    /// No energy cost (for actions like quit)
    pub const FREE: u32 = 0;
}

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
/// High-level phases recognised by the turn scheduler.
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

/// Coordinates energy consumption and state transitions between player and AI.
///
/// The game loop owns the event bus and must publish `GameEvent::PlayerTurnStarted`,
/// `GameEvent::AITurnStarted`, and `GameEvent::TurnEnded` when `state` changes.
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
        let energy_cost = match action {
            PlayerAction::Move(_)
            | PlayerAction::Attack(_)
            | PlayerAction::UseItem(_)
            | PlayerAction::DropItem(_)
            | PlayerAction::Descend
            | PlayerAction::Ascend => energy_costs::FULL_ACTION, // Full action cost
            PlayerAction::Wait => energy_costs::WAIT, // Wait costs half
            PlayerAction::Quit => energy_costs::FREE, // No energy cost for quitting

            // 菜单相关动作 - 免消耗（仅状态切换）
            PlayerAction::OpenInventory
            | PlayerAction::OpenOptions
            | PlayerAction::OpenHelp
            | PlayerAction::OpenCharacterInfo
            | PlayerAction::CloseMenu
            | PlayerAction::MenuNavigate(_)
            | PlayerAction::MenuSelect
            | PlayerAction::MenuBack => energy_costs::FREE,
        };

        if energy_cost > 0 {
            if let Some(player_entity) = find_player(world) {
                if let Ok(mut energy) = world.get::<&mut Energy>(player_entity) {
                    let before = energy.current;
                    energy.current = energy.current.saturating_sub(energy_cost);
                    if energy.current < before {
                        self.player_action_taken = true;
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
            let ai_entities_with_energy: Vec<_> = world
                .query::<(&AI, &Energy, &Actor)>()
                .iter()
                .filter(|(_, (_, energy, _))| energy.current >= 100)
                .map(|(entity, (_, _, actor))| entity)
                .collect();

            if ai_entities_with_energy.is_empty() {
                break;
            }

            // Each AI takes one action
            for ai_entity in ai_entities_with_energy {
                if let Ok(mut energy) = world.get::<&mut Energy>(ai_entity) {
                    energy.current = energy.current.saturating_sub(100);
                }
            }
        }

        // After AI finishes, switch back to player turn
        self.state = TurnState::PlayerTurn;
        Ok(())
    }

    /// Regenerate energy for all entities after a complete turn
    fn regenerate_energy(&self, world: &mut World) {
        for (_, energy) in world.query_mut::<&mut Energy>() {
            energy.current = (energy.current + energy.regeneration_rate).min(energy.max);
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
                            selected_option: 1, // 默认选中“否”
                        };
                        continue; // 不结束循环，允许后续处理
                    }

                    // Consume energy for the completed action
                    self.consume_player_energy(world, &action)?;
                }

                // If player has taken an action, switch to AI turn
                if self.player_action_taken {
                    self.state = TurnState::AITurn;
                    self.player_action_taken = false; // Reset for next turn
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
