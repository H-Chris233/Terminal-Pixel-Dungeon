//! Game loop orchestrating the deterministic phase pipeline described in
//! `docs/turn_system.md`.
//!
//! Besides running systems, the loop bridges turn-state transitions to
//! `GameEvent` notifications and gates AI processing based on the energy
//! scheduler.

use crate::core::GameEngine;
use crate::ecs::*;
use crate::input::*;
use crate::renderer::*;
use crate::systems::*;
use crate::turn_system::{TurnState, TurnSystem};
use anyhow;
use save::{AutoSave, SaveSystem};
use std::time::Duration;

/// Main game loop that runs the ECS systems in order
pub struct GameLoop<R: Renderer, I: InputSource, C: Clock> {
    pub game_engine: GameEngine,
    pub ecs_world: ECSWorld,
    pub renderer: R,
    pub input_source: I,
    pub clock: C,
    pub systems: Vec<Box<dyn System>>,
    pub turn_system: TurnSystem,
    pub is_running: bool,
    pub save_system: Option<AutoSave>,
}

impl<R: Renderer, I: InputSource<Event = crate::input::InputEvent>, C: Clock> GameLoop<R, I, C> {
    pub fn new(renderer: R, input_source: I, clock: C) -> Self {
        // The order of this vector defines the phase pipeline described in the
        // turn-system documentation. Update `docs/turn_system.md` when adjusting it.
        let systems: Vec<Box<dyn System>> = vec![
            Box::new(InputSystem),
            Box::new(MenuSystem), // 菜单系统（需要优先处理菜单动作）
            Box::new(TimeSystem),
            Box::new(MovementSystem),
            Box::new(AISystem),
            Box::new(CombatSystem),
            Box::new(FOVSystem),
            Box::new(EffectSystem),
            Box::new(EnergySystem),
            Box::new(InventorySystem),
            Box::new(HungerSystem), // 新增：饥饿系统
            Box::new(DungeonSystem),
            Box::new(RenderingSystem),
        ];

        let ecs_world = ECSWorld::new();
        let game_engine = GameEngine::new();

        let save_system = match SaveSystem::new("saves", 10) {
            Ok(save_sys) => Some(AutoSave::new(save_sys, std::time::Duration::from_secs(300))),
            Err(e) => {
                eprintln!("Failed to initialize save system: {}", e);
                None
            }
        };

        Self {
            game_engine,
            ecs_world,
            renderer,
            input_source,
            clock,
            systems,
            turn_system: TurnSystem::new(),
            is_running: true,
            save_system,
        }
    }

    /// Initialize the game state
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        self.renderer.init()?;

        // 设置初始状态为主菜单，默认选中第一项
        self.ecs_world.resources.game_state.game_state = GameStatus::MainMenu {
            selected_option: 0,
        };

        // 初始化基础实体（确保世界非空，便于测试与渲染）
        self.initialize_entities();

        Ok(())
    }

    /// Initialize starting entities
    fn initialize_entities(&mut self) {
        // Determine player start position from dungeon if available
        let (start_x, start_y, start_z) =
            if let Some(dungeon) = crate::ecs::get_dungeon_clone(&self.ecs_world.world) {
                let lvl = dungeon.current_level();
                (lvl.stair_up.0, lvl.stair_up.1, dungeon.depth as i32 - 1)
            } else {
                (10, 10, 0)
            };

        // Add player entity
        let player_entity = self.ecs_world.world.spawn((
            Position::new(start_x, start_y, start_z),
            Actor {
                name: "Player".to_string(),
                faction: Faction::Player,
            },
            Renderable {
                symbol: '@',
                fg_color: Color::Yellow,
                bg_color: Some(Color::Black),
                order: 10,
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
                class: Some(hero::class::Class::Warrior),
            },
            Inventory {
                items: vec![],
                max_slots: 10,
            },
            // ========== 新增：玩家专属组件 ==========
            crate::ecs::Hunger::new(5), // 初始饱食度为5（半饱）
            crate::ecs::Wealth::new(0), // 初始金币为0
            crate::ecs::PlayerProgress::new(10, hero::class::Class::Warrior, hero::class::SkillState::default()), // 初始力量10，战士职业
            Viewshed {
                range: 8,
                visible_tiles: vec![],
                memory: vec![],
                dirty: true,
                algorithm: crate::ecs::FovAlgorithm::default(),
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 1,
            },
            crate::ecs::Player, // Player marker component
        ));

        // Add some test enemies
        self.ecs_world.world.spawn((
            Position::new(15, 10, 0),
            Actor {
                name: "Goblin".to_string(),
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
                attack: 5,
                defense: 2,
                accuracy: 70,
                evasion: 10,
                level: 1,
                experience: 10,
                class: None,
            },
            AI {
                ai_type: AIType::Aggressive,
                target: Some(player_entity),
                state: AIState::Idle,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 1,
            },
        ));

        // Add some items
        self.ecs_world.world.spawn((
            Position::new(12, 12, 0),
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

        // Add some basic dungeon tiles (simplified)
        for x in 5..25 {
            for y in 5..25 {
                self.ecs_world.world.spawn((
                    Position::new(x, y, 0),
                    Tile {
                        terrain_type: if x == 5 || x == 24 || y == 5 || y == 24 {
                            TerrainType::Wall
                        } else {
                            TerrainType::Floor
                        },
                        is_passable: x != 5 && x != 24 && y != 5 && y != 24,
                        blocks_sight: x == 5 || x == 24 || y == 5 || y == 24,
                        has_items: false,
                        has_monster: false,
                    },
                    Renderable {
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
                    },
                ));
            }
        }
    }

    /// Main game loop
    pub fn run(&mut self) -> anyhow::Result<()> {
        while self.is_running {
            // Check game state before processing
            match self.ecs_world.resources.game_state.game_state {
                crate::ecs::GameStatus::GameOver { reason: _ } => {
                    self.is_running = false;
                    break;
                }
                crate::ecs::GameStatus::Victory => {
                    self.is_running = false;
                    break;
                }
                _ => {} // Continue normal game processing
            }

            // Handle input
            self.handle_input()?;

            // Update game state based on turns
            self.update_turn()?;

            // Check game state again after update
            match self.ecs_world.resources.game_state.game_state {
                crate::ecs::GameStatus::GameOver { reason: _ } => {
                    self.is_running = false;
                    break;
                }
                crate::ecs::GameStatus::Victory => {
                    self.is_running = false;
                    break;
                }
                _ => {} // Continue normal game processing
            }

            // Render the game
            self.render()?;

            // Small delay to prevent busy looping
            self.clock.sleep(Duration::from_millis(1));
        }

        self.cleanup()?;
        Ok(())
    }

    /// Handle user input
    fn handle_input(&mut self) -> anyhow::Result<()> {
        // Poll for input with a small timeout
        if let Ok(Some(event)) = self.input_source.poll(Duration::from_millis(50)) {
            match event {
                InputEvent::Key(key_event) => {
                    // Convert key event to player action (state-aware, internal KeyEvent)
                    if let Some(action) = key_event_to_player_action_from_internal(
                        key_event,
                        &self.ecs_world.resources.game_state.game_state,
                    ) {
                        // 菜单相关动作直接标记为已完成，交由 MenuSystem 处理；其余进入待处理队列
                        match action {
                            PlayerAction::OpenInventory
                            | PlayerAction::OpenOptions
                            | PlayerAction::OpenHelp
                            | PlayerAction::OpenCharacterInfo
                            | PlayerAction::CloseMenu
                            | PlayerAction::MenuNavigate(_)
                            | PlayerAction::MenuSelect
                            | PlayerAction::MenuBack
                            | PlayerAction::Quit => {
                                self.ecs_world
                                    .resources
                                    .input_buffer
                                    .completed_actions
                                    .push(action);
                            }
                            _ => {
                                self.ecs_world
                                    .resources
                                    .input_buffer
                                    .pending_actions
                                    .push(action);
                            }
                        }
                    }
                }
                InputEvent::Resize(width, height) => {
                    // Handle terminal resize
                    self.renderer
                        .resize(&mut self.ecs_world.resources, width, height)?;
                }
                _ => {} // Other events currently ignored
            }
        }

        Ok(())
    }

    /// Update game state by running all systems for a turn
    fn update_turn(&mut self) -> anyhow::Result<()> {
        // 检查是否需要初始化新游戏（从职业选择进入游戏）
        if let GameStatus::Running = self.ecs_world.resources.game_state.game_state {
            if let Some(class) = self.ecs_world.resources.game_state.selected_class.take() {
                // 清理旧的游戏世界
                self.reinitialize_with_class(class)?;
            }
        }

        // 记录状态以便桥接事件（UIAction → GameEvent）
        let prev_status = self.ecs_world.resources.game_state.game_state;
        let prev_turn_state = self.turn_system.state.clone();

        // Run systems based on turn state
        if self.turn_system.is_player_turn() {
            // Run non-input systems
            for system in &mut self.systems {
                // Skip EnergySystem as we're managing energy through turn system now
                if system.is_energy_system() {
                    continue;
                }

                // 特殊处理 CombatSystem，使用事件版本
                if system.name() == "CombatSystem" {
                    match CombatSystem::run_with_events(&mut self.ecs_world) {
                        SystemResult::Continue => continue,
                        SystemResult::Stop => {
                            self.is_running = false;
                            return Ok(());
                        }
                        SystemResult::Error(msg) => {
                            eprintln!("System error: {}", msg);
                            return Err(anyhow::anyhow!(msg));
                        }
                    }
                }

                // 特殊处理 HungerSystem，使用事件版本
                if system.name() == "HungerSystem" {
                    match HungerSystem::run_with_events(&mut self.ecs_world) {
                        SystemResult::Continue => continue,
                        SystemResult::Stop => {
                            self.is_running = false;
                            return Ok(());
                        }
                        SystemResult::Error(msg) => {
                            eprintln!("System error: {}", msg);
                            return Err(anyhow::anyhow!(msg));
                        }
                    }
                }

                // 特殊处理 DungeonSystem，使用事件版本
                if system.name() == "DungeonSystem" {
                    match DungeonSystem::run_with_events(&mut self.ecs_world) {
                        SystemResult::Continue => continue,
                        SystemResult::Stop => {
                            self.is_running = false;
                            return Ok(());
                        }
                        SystemResult::Error(msg) => {
                            eprintln!("System error: {}", msg);
                            return Err(anyhow::anyhow!(msg));
                        }
                    }
                }

                match system.run(&mut self.ecs_world.world, &mut self.ecs_world.resources) {
                    SystemResult::Continue => continue,
                    SystemResult::Stop => {
                        self.is_running = false;
                        return Ok(());
                    }
                    SystemResult::Error(msg) => {
                        eprintln!("System error: {}", msg);
                        return Err(anyhow::anyhow!(msg));
                    }
                }
            }

            // 处理所有待处理的事件
            self.ecs_world.process_events();

            // Process the player's turn
            self.turn_system
                .process_turn_cycle(&mut self.ecs_world.world, &mut self.ecs_world.resources)?;
        } else {
            // Process AI turns without player input
            // Run non-input systems
            for system in &mut self.systems {
                // Skip EnergySystem as we're managing energy through turn system now
                if system.is_energy_system() {
                    continue;
                }

                // 特殊处理 CombatSystem，使用事件版本
                if system.name() == "CombatSystem" {
                    match CombatSystem::run_with_events(&mut self.ecs_world) {
                        SystemResult::Continue => continue,
                        SystemResult::Stop => {
                            self.is_running = false;
                            return Ok(());
                        }
                        SystemResult::Error(msg) => {
                            eprintln!("System error: {}", msg);
                            return Err(anyhow::anyhow!(msg));
                        }
                    }
                }

                // 特殊处理 HungerSystem，使用事件版本
                if system.name() == "HungerSystem" {
                    match HungerSystem::run_with_events(&mut self.ecs_world) {
                        SystemResult::Continue => continue,
                        SystemResult::Stop => {
                            self.is_running = false;
                            return Ok(());
                        }
                        SystemResult::Error(msg) => {
                            eprintln!("System error: {}", msg);
                            return Err(anyhow::anyhow!(msg));
                        }
                    }
                }

                // 特殊处理 DungeonSystem，使用事件版本
                if system.name() == "DungeonSystem" {
                    match DungeonSystem::run_with_events(&mut self.ecs_world) {
                        SystemResult::Continue => continue,
                        SystemResult::Stop => {
                            self.is_running = false;
                            return Ok(());
                        }
                        SystemResult::Error(msg) => {
                            eprintln!("System error: {}", msg);
                            return Err(anyhow::anyhow!(msg));
                        }
                    }
                }

                match system.run(&mut self.ecs_world.world, &mut self.ecs_world.resources) {
                    SystemResult::Continue => continue,
                    SystemResult::Stop => {
                        self.is_running = false;
                        return Ok(());
                    }
                    SystemResult::Error(msg) => {
                        eprintln!("System error: {}", msg);
                        return Err(anyhow::anyhow!(msg));
                    }
                }
            }

            // 处理所有待处理的事件
            self.ecs_world.process_events();

            // Process AI turns
            self.turn_system
                .process_turn_cycle(&mut self.ecs_world.world, &mut self.ecs_world.resources)?;
        }

        let current_turn_state = self.turn_system.state.clone();
        self.emit_turn_state_events(prev_turn_state, current_turn_state);

        // 桥接：根据 GameStatus 变化发布事件
        self.bridge_status_events(prev_status);

        // 事件入队后立刻处理当前帧事件（避免 UI 状态不同步）
        self.ecs_world.process_events();

        // 准备处理下一帧事件
        self.ecs_world.next_frame();

        // Check for auto-save
        if let Some(auto_save) = &mut self.save_system {
            if let Ok(save_data) = self.ecs_world.to_save_data(&self.turn_system) {
                if let Err(e) = auto_save.try_save(&save_data) {
                    eprintln!("Auto-save failed: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Publish turn-related events whenever the scheduler state changes.
    ///
    /// Systems interested in providing HUD cues or analytics can subscribe to
    /// these high-level hooks instead of polling action buffers directly.
    fn emit_turn_state_events(&mut self, previous: TurnState, current: TurnState) {
        use crate::event_bus::GameEvent;

        if previous == current {
            return;
        }

        match (previous, current) {
            (TurnState::PlayerTurn, TurnState::AITurn)
            | (TurnState::ProcessingPlayerAction, TurnState::AITurn) => {
                self.ecs_world.publish_event(GameEvent::AITurnStarted);
            }
            (TurnState::AITurn, TurnState::PlayerTurn)
            | (TurnState::ProcessingAIActions, TurnState::PlayerTurn) => {
                self.ecs_world.publish_event(GameEvent::TurnEnded {
                    turn: self.ecs_world.resources.clock.turn_count,
                });
                self.ecs_world.publish_event(GameEvent::PlayerTurnStarted);
            }
            (TurnState::AITurn, TurnState::ProcessingAIActions) => {
                self.ecs_world.publish_event(GameEvent::AITurnStarted);
            }
            _ => {}
        }
    }

    /// 将状态机变化桥接到事件总线
    fn bridge_status_events(&mut self, prev: GameStatus) {
        use crate::event_bus::GameEvent;
        let curr = self.ecs_world.resources.game_state.game_state;
        if prev == curr {
            return;
        }
        // Running ↔ 菜单/特殊状态
        let was_menu = matches!(
            prev,
            GameStatus::MainMenu { .. }
                | GameStatus::Paused { .. }
                | GameStatus::Options { .. }
                | GameStatus::Inventory { .. }
                | GameStatus::Help
                | GameStatus::CharacterInfo
                | GameStatus::ConfirmQuit { .. }
        );
        let is_menu = matches!(
            curr,
            GameStatus::MainMenu { .. }
                | GameStatus::Paused { .. }
                | GameStatus::Options { .. }
                | GameStatus::Inventory { .. }
                | GameStatus::Help
                | GameStatus::CharacterInfo
                | GameStatus::ConfirmQuit { .. }
        );

        if !was_menu && is_menu {
            self.ecs_world.publish_event(GameEvent::GamePaused);
        } else if was_menu && !is_menu {
            self.ecs_world.publish_event(GameEvent::GameResumed);
        }

        // 终局事件
        match curr {
            GameStatus::GameOver { reason } => {
                let msg = match reason {
                    GameOverReason::Died(s) => format!("死亡：{}", s),
                    GameOverReason::Defeated(s) => format!("被击败：{}", s),
                    GameOverReason::Starved => "死于饥饿".to_string(),
                    GameOverReason::Trapped(s) => format!("死于陷阱：{}", s),
                    GameOverReason::Quit => "玩家退出".to_string(),
                };
                self.ecs_world.publish_event(GameEvent::GameOver { reason: msg });
            }
            GameStatus::Victory => {
                self.ecs_world.publish_event(GameEvent::Victory);
            }
            _ => {}
        }
    }

    /// Update game state by running all systems
    fn update(&mut self) -> anyhow::Result<()> {
        for system in &mut self.systems {
            match system.run(&mut self.ecs_world.world, &mut self.ecs_world.resources) {
                SystemResult::Continue => continue,
                SystemResult::Stop => {
                    self.is_running = false;
                    break;
                }
                SystemResult::Error(msg) => {
                    eprintln!("System error: {}", msg);
                    return Err(anyhow::anyhow!(msg));
                }
            }
        }

        // Check for auto-save
        if let Some(auto_save) = &mut self.save_system {
            if let Ok(save_data) = self.ecs_world.to_save_data(&self.turn_system) {
                if let Err(e) = auto_save.try_save(&save_data) {
                    eprintln!("Auto-save failed: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Render the current game state
    fn render(&mut self) -> anyhow::Result<()> {
        self.renderer.draw(&mut self.ecs_world)?;
        Ok(())
    }

    /// Clean up resources
    fn cleanup(&mut self) -> anyhow::Result<()> {
        self.renderer.cleanup()?;
        Ok(())
    }

    /// Save the current game state
    pub fn save_game(&mut self, slot: usize) -> anyhow::Result<()> {
        if let Some(auto_save) = &mut self.save_system {
            let save_data = self.ecs_world.to_save_data(&self.turn_system)?;
            auto_save.save_system.save_game(slot, &save_data)?;
        }
        Ok(())
    }

    /// Load a saved game state
    pub fn load_game(&mut self, slot: usize) -> anyhow::Result<()> {
        if let Some(auto_save) = &mut self.save_system {
            let save_data = auto_save.save_system.load_game(slot)?;
            let (turn_state, player_action_taken) = self.ecs_world.from_save_data(save_data)?;
            self.turn_system.set_state(turn_state, player_action_taken);
        }
        Ok(())
    }

    /// 使用选定的职业重新初始化游戏世界
    fn reinitialize_with_class(&mut self, class: hero::class::Class) -> anyhow::Result<()> {
        // 清空当前世界
        self.ecs_world.world.clear();

        // 重新生成地牢
        self.ecs_world.generate_and_set_dungeon(10, 12345)?;

        // 获取起始位置
        let (start_x, start_y, _start_z) =
            if let Some(dungeon) = crate::ecs::get_dungeon_clone(&self.ecs_world.world) {
                let lvl = dungeon.current_level();
                (lvl.stair_up.0, lvl.stair_up.1, dungeon.depth as i32 - 1)
            } else {
                (10, 10, 0)
            };

        // 使用 EntityFactory 创建玩家实体
        let factory = crate::core::EntityFactory::new();
        let _player_entity = factory.create_player(
            &mut self.ecs_world.world,
            start_x,
            start_y,
            class,
        );

        // 添加一些测试敌人
        self.ecs_world.world.spawn((
            Position::new(15, 10, 0),
            Actor {
                name: "Goblin".to_string(),
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
                attack: 8,
                defense: 2,
                accuracy: 70,
                evasion: 10,
                level: 1,
                experience: 10,
                class: None,
            },
            Energy {
                current: 100,
                max: 100,
                regeneration_rate: 1,
            },
        ));
        
        // 添加基础地牢地砖
        for x in 5..25 {
            for y in 5..25 {
                self.ecs_world.world.spawn((
                    Position::new(x, y, 0),
                    Tile {
                        terrain_type: if x == 5 || x == 24 || y == 5 || y == 24 {
                            TerrainType::Wall
                        } else {
                            TerrainType::Floor
                        },
                        is_passable: x != 5 && x != 24 && y != 5 && y != 24,
                        blocks_sight: x == 5 || x == 24 || y == 5 || y == 24,
                        has_items: false,
                        has_monster: false,
                    },
                    Renderable {
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
                    },
                ));
            }
        }
        
        Ok(())
    }
}

/// Headless game loop for testing purposes
pub struct HeadlessGameLoop {
    pub game_engine: GameEngine,
    pub ecs_world: ECSWorld,
    pub systems: Vec<Box<dyn System>>,
    pub turn_system: TurnSystem,
    pub is_running: bool,
    pub save_system: Option<AutoSave>,
}

impl HeadlessGameLoop {
    pub fn new() -> Self {
        // Keep the phase order in sync with the documentation and the interactive loop.
        let systems: Vec<Box<dyn System>> = vec![
            Box::new(InputSystem),
            Box::new(MenuSystem), // 菜单系统（需要优先处理菜单动作）
            Box::new(TimeSystem),
            Box::new(MovementSystem),
            Box::new(AISystem),
            Box::new(CombatSystem),
            Box::new(FOVSystem),
            Box::new(EffectSystem),
            Box::new(EnergySystem),
            Box::new(InventorySystem),
            Box::new(HungerSystem), // 新增：饥饿系统
            Box::new(DungeonSystem),
            Box::new(RenderingSystem),
        ];

        let ecs_world = ECSWorld::new();
        let game_engine = GameEngine::new();

        let save_system = match SaveSystem::new("saves", 10) {
            Ok(save_sys) => Some(AutoSave::new(save_sys, std::time::Duration::from_secs(300))), // 5 min auto-save
            Err(e) => {
                eprintln!("Failed to initialize save system: {}", e);
                None
            }
        };

        Self {
            game_engine,
            ecs_world,
            systems,
            turn_system: TurnSystem::new(),
            is_running: true,
            save_system,
        }
    }

    /// Run the game loop without rendering (for testing)
    pub fn run_for_ticks(&mut self, ticks: u32) -> anyhow::Result<()> {
        for _ in 0..ticks {
            if !self.is_running {
                break;
            }

            self.update()?;
        }

        Ok(())
    }

    /// Save the current game state
    pub fn save_game(&mut self, slot: usize) -> anyhow::Result<()> {
        if let Some(auto_save) = &mut self.save_system {
            let save_data = self.ecs_world.to_save_data(&self.turn_system)?;
            auto_save.save_system.save_game(slot, &save_data)?;
        }
        Ok(())
    }

    /// Load a saved game state
    pub fn load_game(&mut self, slot: usize) -> anyhow::Result<()> {
        if let Some(auto_save) = &mut self.save_system {
            let save_data = auto_save.save_system.load_game(slot)?;
            let (turn_state, player_action_taken) = self.ecs_world.from_save_data(save_data)?;
            self.turn_system.set_state(turn_state, player_action_taken);
        }
        Ok(())
    }

    /// Update game state by running all systems
    fn update(&mut self) -> anyhow::Result<()> {
        for system in &mut self.systems {
            match system.run(&mut self.ecs_world.world, &mut self.ecs_world.resources) {
                SystemResult::Continue => continue,
                SystemResult::Stop => {
                    self.is_running = false;
                    break;
                }
                SystemResult::Error(msg) => {
                    eprintln!("System error: {}", msg);
                    return Err(anyhow::anyhow!(msg));
                }
            }
        }

        // Check for auto-save
        if let Some(auto_save) = &mut self.save_system {
            if let Ok(save_data) = self.ecs_world.to_save_data(&self.turn_system) {
                if let Err(e) = auto_save.try_save(&save_data) {
                    eprintln!("Auto-save failed: {}", e);
                }
            }
        }

        Ok(())
    }
}

/// Helper function to convert internal key event to player action
fn key_event_to_player_action_from_internal(
    key_event: KeyEvent,
    game_state: &GameStatus,
) -> Option<PlayerAction> {
    // 根据游戏状态决定如何解释按键（复用 input.rs 的语义）
    match game_state {
        GameStatus::MainMenu { .. }
        | GameStatus::Paused { .. }
        | GameStatus::Options { .. }
        | GameStatus::Inventory { .. }
        | GameStatus::Help
        | GameStatus::CharacterInfo
        | GameStatus::ClassSelection { .. }
        | GameStatus::ConfirmQuit { .. } => {
            // 菜单上下文：方向键/Enter/Esc 等（增加 WASD 支持）
            match (key_event.code, key_event.modifiers.shift) {
                (KeyCode::Up, _)
                | (KeyCode::Char('k'), _)
                | (KeyCode::Char('w'), _) => Some(PlayerAction::MenuNavigate(NavigateDirection::Up)),
                (KeyCode::Down, _)
                | (KeyCode::Char('j'), _)
                | (KeyCode::Char('s'), _) => Some(PlayerAction::MenuNavigate(NavigateDirection::Down)),
                (KeyCode::Left, _)
                | (KeyCode::Char('h'), _)
                | (KeyCode::Char('a'), _) => Some(PlayerAction::MenuNavigate(NavigateDirection::Left)),
                (KeyCode::Right, _)
                | (KeyCode::Char('l'), _)
                | (KeyCode::Char('d'), _) => Some(PlayerAction::MenuNavigate(NavigateDirection::Right)),
                (KeyCode::Enter, _) => Some(PlayerAction::MenuSelect),
                (KeyCode::Esc, _) | (KeyCode::Backspace, _) => Some(PlayerAction::CloseMenu),
                (KeyCode::Char('q'), _) => Some(PlayerAction::Quit),
                _ => None,
            }
        }
        _ => {
            match (key_event.code, key_event.modifiers.shift) {
                // Movement keys（仅支持方向键、vi-keys，避免与 'd' 丢弃冲突）
                (KeyCode::Char('k'), _) | (KeyCode::Up, _) => Some(PlayerAction::Move(Direction::North)),
                (KeyCode::Char('j'), _) | (KeyCode::Down, _) => Some(PlayerAction::Move(Direction::South)),
                (KeyCode::Char('h'), _) | (KeyCode::Left, _) => Some(PlayerAction::Move(Direction::West)),
                (KeyCode::Char('l'), _) | (KeyCode::Right, _) => Some(PlayerAction::Move(Direction::East)),
                (KeyCode::Char('y'), _) => Some(PlayerAction::Move(Direction::NorthWest)),
                (KeyCode::Char('u'), _) => Some(PlayerAction::Move(Direction::NorthEast)),
                (KeyCode::Char('b'), _) => Some(PlayerAction::Move(Direction::SouthWest)),
                (KeyCode::Char('n'), _) => Some(PlayerAction::Move(Direction::SouthEast)),

                // Wait/skip turn
                (KeyCode::Char('.'), _) => Some(PlayerAction::Wait),

                // Stairs
                (KeyCode::Char('>'), _) => Some(PlayerAction::Descend),
                (KeyCode::Char('<'), _) => Some(PlayerAction::Ascend),

                // Attack via direction（放宽 SHIFT 要求）
                (KeyCode::Char('K'), _) => Some(PlayerAction::Attack(Position { x: 0, y: -1, z: 0 })),
                (KeyCode::Char('J'), _) => Some(PlayerAction::Attack(Position { x: 0, y: 1, z: 0 })),
                (KeyCode::Char('H'), _) => Some(PlayerAction::Attack(Position { x: -1, y: 0, z: 0 })),
                (KeyCode::Char('L'), _) => Some(PlayerAction::Attack(Position { x: 1, y: 0, z: 0 })),
                (KeyCode::Char('Y'), _) => Some(PlayerAction::Attack(Position { x: -1, y: -1, z: 0 })),
                (KeyCode::Char('U'), _) => Some(PlayerAction::Attack(Position { x: 1, y: -1, z: 0 })),
                (KeyCode::Char('B'), _) => Some(PlayerAction::Attack(Position { x: -1, y: 1, z: 0 })),
                (KeyCode::Char('N'), _) => Some(PlayerAction::Attack(Position { x: 1, y: 1, z: 0 })),

                // Game control
                (KeyCode::Char('q'), _) => Some(PlayerAction::Quit),

                // Number keys for items/spells
                (KeyCode::Char('1'), _) => Some(PlayerAction::UseItem(0)),
                (KeyCode::Char('2'), _) => Some(PlayerAction::UseItem(1)),
                (KeyCode::Char('3'), _) => Some(PlayerAction::UseItem(2)),
                (KeyCode::Char('4'), _) => Some(PlayerAction::UseItem(3)),
                (KeyCode::Char('5'), _) => Some(PlayerAction::UseItem(4)),
                (KeyCode::Char('6'), _) => Some(PlayerAction::UseItem(5)),
                (KeyCode::Char('7'), _) => Some(PlayerAction::UseItem(6)),
                (KeyCode::Char('8'), _) => Some(PlayerAction::UseItem(7)),
                (KeyCode::Char('9'), _) => Some(PlayerAction::UseItem(8)),

                // Drop item
                (KeyCode::Char('d'), _) => Some(PlayerAction::DropItem(0)),

                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{InputEvent, InputSource};
    use crate::renderer::GameClock;
    use ratatui::{
        backend::TestBackend,
        layout::Rect,
        widgets::Block,
        Frame, Terminal,
    };
    use std::time::Duration;

    struct TestRenderer {
        terminal: Terminal<TestBackend>,
    }

    impl TestRenderer {
        fn new() -> anyhow::Result<Self> {
            let backend = TestBackend::new(80, 24);
            let terminal = Terminal::new(backend)?;
            Ok(Self { terminal })
        }
    }

    impl crate::renderer::Renderer for TestRenderer {
        type Backend = TestBackend;

        fn init(&mut self) -> anyhow::Result<()> {
            Ok(())
        }

        fn draw(&mut self, _ecs_world: &mut ECSWorld) -> anyhow::Result<()> {
            self.terminal.draw(|frame| {
                let area = frame.size();
                frame.render_widget(Block::default(), area);
            })?;
            Ok(())
        }

        fn draw_ui(&mut self, frame: &mut Frame<'_>, area: Rect) {
            frame.render_widget(Block::default(), area);
        }

        fn resize(
            &mut self,
            resources: &mut Resources,
            width: u16,
            height: u16,
        ) -> anyhow::Result<()> {
            resources.game_state.terminal_width = width;
            resources.game_state.terminal_height = height;
            Ok(())
        }

        fn cleanup(&mut self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct NoopInput;

    impl InputSource for NoopInput {
        type Event = InputEvent;

        fn poll(&mut self, _timeout: Duration) -> anyhow::Result<Option<Self::Event>> {
            Ok(None)
        }

        fn is_input_available(&self) -> anyhow::Result<bool> {
            Ok(false)
        }
    }

    #[test]
    fn test_game_loop_creation() -> anyhow::Result<()> {
        let renderer = TestRenderer::new()?;
        let input_source = NoopInput::default();
        let clock = GameClock::new(16); // ~60 FPS

        let mut game_loop = GameLoop::new(renderer, input_source, clock);

        // Initialize the game loop
        game_loop.initialize()?;

        // Check that entities were initialized
        assert!(game_loop.ecs_world.world.iter().count() > 0);

        Ok(())
    }
}
