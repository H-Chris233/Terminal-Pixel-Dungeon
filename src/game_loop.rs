//! 游戏循环，协调 `docs/turn_system.md` 中描述的确定性阶段管道。
//!
//! 游戏循环现在被重组为明确的回合阶段：
//! - PreInput: 时间/时钟更新
//! - Input: 输入处理和菜单
//! - IntentGathering: AI 决策
//! - ActionResolution: 移动、FOV、战斗、效果、库存、饥饿、地牢
//! - PostTurnUpkeep: 回合结束时的清理和更新
//! - Render: 渲染输出
//!
//! 每个阶段根据游戏状态（菜单、运行中、暂停等）有条件地执行。

use crate::core::GameEngine;
use crate::ecs::*;
use crate::input::*;
use crate::renderer::*;
use crate::systems::*;
use crate::systems::EffectPhase;
use crate::turn_system::{TurnPhase, TurnState, TurnSystem};
use anyhow;
use save::{AutoSave, SaveSystem};
use std::time::{Duration, Instant};

/// 系统执行阶段，定义了确定性的回合顺序
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemPhase {
    /// 预输入阶段：时间和时钟更新
    PreInput,
    /// 输入阶段：输入处理和菜单
    Input,
    /// 意图收集阶段：AI 决策
    IntentGathering,
    /// 动作解决阶段：执行游戏逻辑
    ActionResolution,
    /// 回合后维护阶段：能量、状态效果、清理
    PostTurnUpkeep,
    /// 渲染阶段：显示输出
    Render,
}

/// 主游戏循环，按阶段运行 ECS 系统
pub struct GameLoop<R: Renderer, I: InputSource, C: Clock> {
    pub game_engine: GameEngine,
    pub ecs_world: ECSWorld,
    pub renderer: R,
    pub input_source: I,
    pub clock: C,
    
    // 分阶段的系统
    pub pre_input_systems: Vec<Box<dyn System>>,
    pub input_systems: Vec<Box<dyn System>>,
    pub intent_gathering_systems: Vec<Box<dyn System>>,
    pub action_resolution_systems: Vec<Box<dyn System>>,
    pub post_turn_upkeep_systems: Vec<Box<dyn System>>,
    pub render_systems: Vec<Box<dyn System>>,
    
    pub turn_system: TurnSystem,
    pub is_running: bool,
    pub save_system: Option<AutoSave>,
    
    // 回合计时器
    turn_start_time: Option<Instant>,
    last_turn_duration: Duration,
}

impl<R: Renderer, I: InputSource<Event = crate::input::InputEvent>, C: Clock> GameLoop<R, I, C> {
    pub fn new(renderer: R, input_source: I, clock: C) -> Self {
        // 分阶段的系统定义
        // 系统执行顺序遵循因果关系：移动 → FOV → AI意图 → 战斗 → 状态清理
        
        let pre_input_systems: Vec<Box<dyn System>> = vec![
            Box::new(TimeSystem),
        ];
        
        let input_systems: Vec<Box<dyn System>> = vec![
            Box::new(InputSystem),
            Box::new(MenuSystem),
        ];
        
        let intent_gathering_systems: Vec<Box<dyn System>> = vec![
            Box::new(AISystem),
        ];
        
        let action_resolution_systems: Vec<Box<dyn System>> = vec![
            Box::new(MovementSystem),
            Box::new(FOVSystem),
            Box::new(CombatSystem),
            Box::new(EffectSystem::new()),
            Box::new(InventorySystem),
            Box::new(HungerSystem),
            Box::new(DungeonSystem),
        ];
        
        let post_turn_upkeep_systems: Vec<Box<dyn System>> = vec![
            Box::new(EnergySystem),
        ];
        
        let render_systems: Vec<Box<dyn System>> = vec![
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
            pre_input_systems,
            input_systems,
            intent_gathering_systems,
            action_resolution_systems,
            post_turn_upkeep_systems,
            render_systems,
            turn_system: TurnSystem::new(),
            is_running: true,
            save_system,
            turn_start_time: None,
            last_turn_duration: Duration::from_millis(0),
        }
    }

    /// 初始化游戏状态
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        self.renderer.init()?;

        // 设置初始状态为主菜单，默认选中第一项
        self.ecs_world.resources.game_state.game_state =
            GameStatus::MainMenu { selected_option: 0 };

        // 初始化基础实体（确保世界非空，便于测试与渲染）
        self.initialize_entities();

        Ok(())
    }

    /// 初始化起始实体
    fn initialize_entities(&mut self) {
        // 如果可用，从地牢确定玩家起始位置
        let (start_x, start_y, start_z) =
            if let Some(dungeon) = crate::ecs::get_dungeon_clone(&self.ecs_world.world) {
                let lvl = dungeon.current_level();
                (lvl.stair_up.0, lvl.stair_up.1, dungeon.depth as i32 - 1)
            } else {
                (10, 10, 0)
            };

        // 添加玩家实体
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
            crate::ecs::PlayerProgress::new(
                10,
                hero::class::Class::Warrior,
                hero::class::SkillState::default(),
            ), // 初始力量10，战士职业
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
            crate::ecs::Player, // 玩家标记组件
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
                    // Convert internal KeyEvent to crossterm KeyEvent
                    let crossterm_key = crossterm::event::KeyEvent::new(
                        match key_event.code {
                            KeyCode::Char(c) => crossterm::event::KeyCode::Char(c),
                            KeyCode::Enter => crossterm::event::KeyCode::Enter,
                            KeyCode::Esc => crossterm::event::KeyCode::Esc,
                            KeyCode::Backspace => crossterm::event::KeyCode::Backspace,
                            KeyCode::Delete => crossterm::event::KeyCode::Delete,
                            KeyCode::Tab => crossterm::event::KeyCode::Tab,
                            KeyCode::Left => crossterm::event::KeyCode::Left,
                            KeyCode::Right => crossterm::event::KeyCode::Right,
                            KeyCode::Up => crossterm::event::KeyCode::Up,
                            KeyCode::Down => crossterm::event::KeyCode::Down,
                            KeyCode::PageUp => crossterm::event::KeyCode::PageUp,
                            KeyCode::PageDown => crossterm::event::KeyCode::PageDown,
                            KeyCode::Home => crossterm::event::KeyCode::Home,
                            KeyCode::End => crossterm::event::KeyCode::End,
                            KeyCode::Insert => crossterm::event::KeyCode::Insert,
                            KeyCode::F(n) => crossterm::event::KeyCode::F(n),
                            KeyCode::Null => crossterm::event::KeyCode::Null,
                        },
                        crossterm::event::KeyModifiers::empty(),
                    );
                    
                    // Convert key event to player action (state-aware)
                    if let Some(action) = key_event_to_player_action(
                        crossterm_key,
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

        // 开始回合计时（仅在玩家回合开始时）
        if self.turn_system.is_player_turn() && self.turn_start_time.is_none() {
            self.turn_start_time = Some(Instant::now());
        }

        // 运行分阶段的系统
        self.run_phased_systems()?;

        // 处理回合周期
        self.turn_system
            .process_turn_cycle(&mut self.ecs_world.world, &mut self.ecs_world.resources)?;

        // 检查是否完成了一个完整回合（AI回合结束，回到玩家回合）
        let current_turn_state = self.turn_system.state.clone();
        if matches!(prev_turn_state, TurnState::AITurn)
            && matches!(current_turn_state, TurnState::PlayerTurn)
        {
            // 记录回合时长
            if let Some(start) = self.turn_start_time.take() {
                self.last_turn_duration = start.elapsed();
            }
            
            // 在回合结束时触发自动保存
            self.try_autosave_at_turn_end()?;
        }

        self.emit_turn_state_events(prev_turn_state, current_turn_state);

        // 桥接：根据 GameStatus 变化发布事件
        self.bridge_status_events(prev_status);

        // 事件入队后立刻处理当前帧事件（避免 UI 状态不同步）
        self.ecs_world.process_events();

        // 准备处理下一帧事件
        self.ecs_world.next_frame();

        Ok(())
    }

    /// 运行分阶段的系统，根据游戏状态和回合阶段有条件地执行
    fn run_phased_systems(&mut self) -> anyhow::Result<()> {
        let game_state = self.ecs_world.resources.game_state.game_state;
        
        // 检查是否在菜单/暂停状态
        let is_menu_or_paused = matches!(
            game_state,
            GameStatus::MainMenu { .. }
                | GameStatus::Paused { .. }
                | GameStatus::Options { .. }
                | GameStatus::Inventory { .. }
                | GameStatus::Help
                | GameStatus::CharacterInfo
                | GameStatus::ConfirmQuit { .. }
                | GameStatus::ClassSelection { .. }
        );

        // 第1阶段：PreInput - 时间系统（总是运行）
        self.run_system_phase(SystemPhase::PreInput)?;

        // 第2阶段：Input - 输入和菜单（总是运行）
        self.run_system_phase(SystemPhase::Input)?;

        // 如果在菜单或暂停状态，跳过游戏逻辑阶段
        if is_menu_or_paused {
            // 只运行渲染阶段
            return Ok(());
        }

        // 第3阶段：IntentGathering - AI决策（仅在AI回合）
        if self.turn_system.is_ai_turn() {
            self.run_system_phase(SystemPhase::IntentGathering)?;
        }

        // 第4阶段：ActionResolution - 游戏逻辑（在任何非暂停状态）
        self.run_system_phase(SystemPhase::ActionResolution)?;

        // 第5阶段：PostTurnUpkeep - 回合后维护（在回合结束时）
        if matches!(self.turn_system.state, TurnState::AITurn) {
            self.run_system_phase(SystemPhase::PostTurnUpkeep)?;
        }

        Ok(())
    }

    /// 运行特定阶段的系统
    fn run_system_phase(&mut self, phase: SystemPhase) -> anyhow::Result<()> {
        let systems = match phase {
            SystemPhase::PreInput => &mut self.pre_input_systems,
            SystemPhase::Input => &mut self.input_systems,
            SystemPhase::IntentGathering => &mut self.intent_gathering_systems,
            SystemPhase::ActionResolution => &mut self.action_resolution_systems,
            SystemPhase::PostTurnUpkeep => &mut self.post_turn_upkeep_systems,
            SystemPhase::Render => &mut self.render_systems,
        };

        for system in systems {
            // 特殊处理使用事件版本的系统
            match system.name() {
                "MovementSystem" => {
                    match MovementSystem::run_with_events(&mut self.ecs_world) {
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
                "CombatSystem" => {
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
                "EffectSystem" => {
                    match EffectSystem::run_with_events(&mut self.ecs_world, EffectPhase::EndOfTurn) {
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
                "HungerSystem" => {
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
                "DungeonSystem" => {
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
                "AISystem" => {
                    match AISystem::run_with_events(&mut self.ecs_world) {
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
                _ => {
                    // 运行标准系统
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
            }
        }

        // 处理所有待处理的事件
        self.ecs_world.process_events();

        Ok(())
    }

    /// 在回合结束时尝试自动保存
    fn try_autosave_at_turn_end(&mut self) -> anyhow::Result<()> {
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
                self.ecs_world
                    .publish_event(GameEvent::GameOver { reason: msg });
            }
            GameStatus::Victory => {
                self.ecs_world.publish_event(GameEvent::Victory);
            }
            _ => {}
        }
    }

    /// Update game state by running all systems
    fn update(&mut self) -> anyhow::Result<()> {
        for system in &mut self.pre_input_systems {
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
        Ok(())
    }

    /// Render the game state
    fn render(&mut self) -> anyhow::Result<()> {
        // 渲染阶段总是运行，即使在菜单/暂停状态
        self.run_system_phase(SystemPhase::Render)?;
        Ok(())
    }

    /// Cleanup before exit
    fn cleanup(&mut self) -> anyhow::Result<()> {
        self.renderer.cleanup()
    }

    /// Reinitialize the game with a new character class
    fn reinitialize_with_class(&mut self, class: hero::class::Class) -> anyhow::Result<()> {
        // 清空当前世界
        self.ecs_world.clear();

        // 重新生成地牢
        self.ecs_world.generate_and_set_dungeon(5, 42)?;

        // 获取起始位置
        let (start_x, start_y, _start_z) =
            if let Some(dungeon) = crate::ecs::get_dungeon_clone(&self.ecs_world.world) {
                let lvl = dungeon.current_level();
                (lvl.stair_up.0, lvl.stair_up.1, dungeon.depth as i32 - 1)
            } else {
                (10, 10, 0)
            };

        // 创建基于职业的玩家实体
        let factory = crate::core::entity_factory::EntityFactory::new();
        factory.create_player(
            &mut self.ecs_world.world,
            start_x,
            start_y,
            class,
        );

        // 生成一些敌人
        factory.create_monster(&mut self.ecs_world.world, start_x + 5, start_y, "goblin");
        factory.create_monster(&mut self.ecs_world.world, start_x - 5, start_y, "rat");

        Ok(())
    }

    /// 获取上一回合的时长
    pub fn last_turn_duration(&self) -> Duration {
        self.last_turn_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::test_helpers::MockInputSource;
    use crate::renderer::test_helpers::MockRenderer;
    use std::time::Duration;

    /// Mock clock for testing
    pub struct MockClock;

    impl Clock for MockClock {
        fn now(&self) -> std::time::SystemTime {
            std::time::SystemTime::now()
        }

        fn elapsed(&self, since: std::time::SystemTime) -> Duration {
            since.elapsed().unwrap_or(Duration::from_secs(0))
        }

        fn sleep(&self, _duration: Duration) {
            // No-op for tests
        }

        fn tick_rate(&self) -> Duration {
            Duration::from_millis(16) // ~60 FPS
        }
    }

    #[test]
    fn test_game_loop_initialization() {
        let renderer = MockRenderer;
        let input = MockInputSource::new();
        let clock = MockClock;

        let mut game_loop = GameLoop::new(renderer, input, clock);
        assert!(game_loop.initialize().is_ok());
        assert!(game_loop.is_running);
    }

    #[test]
    fn test_phased_system_execution_order() {
        let renderer = MockRenderer;
        let input = MockInputSource::new();
        let clock = MockClock;

        let game_loop = GameLoop::new(renderer, input, clock);

        // 验证系统阶段数量
        assert!(!game_loop.pre_input_systems.is_empty());
        assert!(!game_loop.input_systems.is_empty());
        assert!(!game_loop.intent_gathering_systems.is_empty());
        assert!(!game_loop.action_resolution_systems.is_empty());
        assert!(!game_loop.post_turn_upkeep_systems.is_empty());
        assert!(!game_loop.render_systems.is_empty());

        // 验证 action_resolution 系统顺序
        let action_systems: Vec<&str> = game_loop
            .action_resolution_systems
            .iter()
            .map(|s| s.name())
            .collect();

        // 验证因果顺序：移动 → FOV → 战斗 → 效果
        assert_eq!(action_systems[0], "MovementSystem");
        assert_eq!(action_systems[1], "FOVSystem");
        assert_eq!(action_systems[2], "CombatSystem");
        assert_eq!(action_systems[3], "EffectSystem");
    }

    #[test]
    fn test_menu_state_short_circuits_game_logic() {
        let renderer = MockRenderer;
        let input = MockInputSource::new();
        let clock = MockClock;

        let mut game_loop = GameLoop::new(renderer, input, clock);
        game_loop.initialize().unwrap();

        // 设置为主菜单状态
        game_loop.ecs_world.resources.game_state.game_state =
            GameStatus::MainMenu { selected_option: 0 };

        // 运行分阶段系统应该跳过游戏逻辑
        let result = game_loop.run_phased_systems();
        assert!(result.is_ok());

        // 验证玩家回合状态未改变（因为游戏逻辑被跳过）
        assert!(game_loop.turn_system.is_player_turn());
    }

    #[test]
    fn test_turn_timing_hooks() {
        let renderer = MockRenderer;
        let input = MockInputSource::new();
        let clock = MockClock;

        let mut game_loop = GameLoop::new(renderer, input, clock);
        game_loop.initialize().unwrap();

        // 初始状态
        assert_eq!(game_loop.last_turn_duration(), Duration::from_millis(0));

        // 模拟回合开始
        game_loop.turn_system.state = TurnState::PlayerTurn;
        game_loop.turn_start_time = Some(Instant::now());

        // 模拟回合结束
        std::thread::sleep(Duration::from_millis(10));
        let prev_state = TurnState::AITurn;
        let current_state = TurnState::PlayerTurn;

        if let Some(start) = game_loop.turn_start_time.take() {
            game_loop.last_turn_duration = start.elapsed();
        }

        // 验证记录了回合时长
        assert!(game_loop.last_turn_duration() > Duration::from_millis(5));
    }

    #[test]
    fn test_no_state_leakage_between_turns() {
        let renderer = MockRenderer;
        let input = MockInputSource::new();
        let clock = MockClock;

        let mut game_loop = GameLoop::new(renderer, input, clock);
        game_loop.initialize().unwrap();

        // 设置为运行状态
        game_loop.ecs_world.resources.game_state.game_state = GameStatus::Running;

        // 添加一些待处理的动作
        game_loop
            .ecs_world
            .resources
            .input_buffer
            .pending_actions
            .push(PlayerAction::Wait);

        // 运行一个回合
        let _ = game_loop.run_phased_systems();

        // 验证动作已被处理或保留（不应泄漏到其他地方）
        let total_actions = game_loop.ecs_world.resources.input_buffer.pending_actions.len()
            + game_loop
                .ecs_world
                .resources
                .input_buffer
                .completed_actions
                .len();

        // 动作应该被处理（总数可能为0或1，取决于系统是否处理了它）
        assert!(total_actions <= 1);
    }

    #[test]
    fn test_autosave_triggered_at_turn_end() {
        let renderer = MockRenderer;
        let input = MockInputSource::new();
        let clock = MockClock;

        let mut game_loop = GameLoop::new(renderer, input, clock);
        game_loop.initialize().unwrap();

        // 确保有保存系统
        assert!(game_loop.save_system.is_some());

        // 模拟回合结束（从AI回合回到玩家回合）
        game_loop.turn_system.state = TurnState::PlayerTurn;
        let _prev_state = TurnState::AITurn;

        // 尝试自动保存
        let result = game_loop.try_autosave_at_turn_end();
        assert!(result.is_ok());
    }
}
