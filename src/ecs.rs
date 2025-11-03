//! 游戏的 ECS（实体组件系统）实现

use hecs::{Entity, World};
use std::time::Duration;

use rand::SeedableRng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::event_bus::{EventBus, EventHandler, GameEvent, LogLevel, Priority};
use achievements::AchievementsManager;
use error::GameError;
use hero::{
    Bag, Hero,
    class::{Class, SkillState},
};
use items as game_items;
use save::SaveData;
use std::sync::{Arc, Mutex};

// 说明：在完全解耦的系统中，这些模块间的通信应该通过事件总线完成
// 例如，保存系统通过监听 GameSaved 事件来保存游戏状态
// 而不是直接依赖其他模块的结构体

/// 主要的 ECS 世界容器
pub struct ECSWorld {
    pub world: World,
    pub resources: Resources,
    pub event_bus: EventBus,
}

impl ECSWorld {
    pub fn new() -> Self {
        let mut ecs_world = Self {
            world: World::new(),
            resources: Resources::default(),
            event_bus: EventBus::new(),
        };

        // 注册默认的事件处理器
        ecs_world.register_default_handlers();

        ecs_world
    }

    /// 注册默认的事件处理器
    fn register_default_handlers(&mut self) {
        // 暂时不注册默认处理器
        // 事件处理将在 process_events 中直接完成
        // 外部模块可以根据需要注册自己的处理器
    }

    pub fn generate_and_set_dungeon(&mut self, max_depth: usize, seed: u64) -> anyhow::Result<()> {
        let dungeon = dungeon::Dungeon::generate(max_depth, seed)?;
        set_dungeon_instance(&mut self.world, dungeon);
        // 重新初始化 RNG 以确保游戏中的随机性一致
        self.resources.rng = StdRng::seed_from_u64(seed);
        self.resources.game_state.depth = 1;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.world.clear();
        self.resources = Resources::default();
        self.event_bus.clear();
    }

    /// 发布事件到事件总线
    pub fn publish_event(&mut self, event: GameEvent) {
        self.event_bus.publish(event);
    }

    /// 发布延迟事件（下一帧处理）
    pub fn publish_delayed_event(&mut self, event: GameEvent) {
        self.event_bus.publish_delayed(event);
    }

    /// 处理所有待处理的事件
    /// 这个方法在 ECSWorld 级别处理核心游戏状态更新
    /// 外部处理器（通过 subscribe）用于日志、UI 等非核心功能
    pub fn process_events(&mut self) {
        // 事件已通过订阅者模式处理（日志、统计等）
        // 这里处理核心游戏状态的更新
        let events: Vec<GameEvent> = self.event_bus.drain().collect();

        for event in events {
            self.handle_core_event(&event);
        }
    }

    /// 处理核心游戏状态事件（更新 Resources）
    /// 
    /// 注意：此方法设计为向后兼容，能够处理新旧事件类型而不会panic
    fn handle_core_event(&mut self, event: &GameEvent) {
        match event {
            GameEvent::DamageDealt {
                damage,
                is_critical,
                ..
            } => {
                let msg = if *is_critical {
                    format!("暴击！造成 {} 点伤害", damage)
                } else {
                    format!("造成 {} 点伤害", damage)
                };
                self.resources.game_state.message_log.push(msg);
            }

            GameEvent::EntityDied { entity_name, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("{} 已死亡", entity_name));
            }

            GameEvent::ItemPickedUp { item_name, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("拾取了 {}", item_name));
            }

            GameEvent::ItemUsed {
                item_name, effect, ..
            } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("使用了 {}，{}", item_name, effect));
            }

            GameEvent::LevelChanged {
                old_level,
                new_level,
            } => {
                self.resources.game_state.depth = *new_level;
                self.resources
                    .game_state
                    .message_log
                    .push(format!("从第 {} 层进入第 {} 层", old_level, new_level));
            }

            GameEvent::GameOver { reason } => {
                self.resources.game_state.game_state = GameStatus::GameOver {
                    reason: GameOverReason::Died("游戏结束"),
                };
                self.resources
                    .game_state
                    .message_log
                    .push(format!("游戏结束：{}", reason));
            }

            GameEvent::Victory => {
                self.resources.game_state.game_state = GameStatus::Victory;
                self.resources
                    .game_state
                    .message_log
                    .push("恭喜！你获得了胜利！".to_string());
            }

            GameEvent::LogMessage { message, level } => {
                let prefix = match level {
                    LogLevel::Debug => "[调试] ",
                    LogLevel::Info => "",
                    LogLevel::Warning => "[警告] ",
                    LogLevel::Error => "[错误] ",
                };
                self.resources
                    .game_state
                    .message_log
                    .push(format!("{}{}", prefix, message));
            }

            GameEvent::TrapTriggered { trap_type, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("触发了{}陷阱！", trap_type));
            }

            GameEvent::StatusApplied {
                status, duration, intensity, ..
            } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("受到{}效果影响，持续{}回合（强度：{}）", status, duration, intensity));
            }

            GameEvent::StatusRemoved { status, reason, .. } => {
                let msg = if reason == "expired" {
                    format!("{}效果已消失", status)
                } else {
                    format!("{}效果被{}移除", status, reason)
                };
                self.resources
                    .game_state
                    .message_log
                    .push(msg);
            }

            GameEvent::StatusEffectTicked {
                status, damage, remaining_turns, ..
            } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("{}造成{}点伤害（剩余{}回合）", status, damage, remaining_turns));
            }

            GameEvent::StatusEffectConflict {
                removed_effect,
                new_effect,
                ..
            } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("{}被{}覆盖", removed_effect, new_effect));
            }

            // 饥饿事件处理
            GameEvent::PlayerHungry { satiety, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("你感到饥饿...（饱食度：{}）", satiety));
            }

            GameEvent::PlayerStarving { .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push("你正在饿死！".to_string());
            }

            GameEvent::StarvationDamage { damage, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("饥饿造成了 {} 点伤害", damage));
            }

            GameEvent::FoodEaten { food_name, satiety_restored, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("你吃了{}，恢复了{}点饱食度", food_name, satiety_restored));
            }

            // ===== 新增事件处理 =====

            // 职业技能事件
            GameEvent::ClassSkillUsed { skill_name, class, energy_cost, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("{} 使用了技能 {}（消耗 {} 能量）", class, skill_name, energy_cost));
            }

            GameEvent::ClassSkillReady { skill_name, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("技能 {} 冷却完成", skill_name));
            }

            GameEvent::SkillUseFailed { skill_name, reason, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("技能 {} 使用失败：{}", skill_name, reason));
            }

            GameEvent::PassivePerkTriggered { perk_name, effect, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("被动技能 {} 触发：{}", perk_name, effect));
            }

            // 动作事件
            GameEvent::ActionCompleted { action_type, success, .. } => {
                if !success {
                    self.resources
                        .game_state
                        .message_log
                        .push(format!("动作 {} 执行失败", action_type));
                }
            }

            GameEvent::ActionFailed { action_type, reason, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("动作 {} 失败：{}", action_type, reason));
            }

            // 高级战斗事件
            GameEvent::CombatBlocked { blocked_damage, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("格挡了 {} 点伤害", blocked_damage));
            }

            GameEvent::CombatParried { parry_damage, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("招架反击造成 {} 点伤害", parry_damage));
            }

            GameEvent::CombatDodged { .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push("完全闪避了攻击".to_string());
            }

            GameEvent::CombatGrazed { damage, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("擦伤造成 {} 点伤害", damage));
            }

            GameEvent::CombatLifesteal { healed, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("吸血恢复了 {} 点生命", healed));
            }

            GameEvent::CombatReflected { reflected_damage, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("反弹了 {} 点伤害", reflected_damage));
            }

            GameEvent::CombatShieldAbsorbed { damage_absorbed, shield_remaining, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("护盾吸收 {} 点伤害（剩余：{}）", damage_absorbed, shield_remaining));
            }

            // 状态生命周期事件
            GameEvent::StatusStacked { status, new_intensity, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("{} 效果堆叠至 {} 层", status, new_intensity));
            }

            GameEvent::StatusRefreshed { status, duration, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("{} 效果刷新，持续 {} 回合", status, duration));
            }

            GameEvent::StatusResisted { status, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("抵抗了 {} 效果", status));
            }

            GameEvent::StatusImmune { status, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("免疫 {} 效果", status));
            }

            // 环境事件
            GameEvent::DoorOpened { door_type, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("打开了{}", door_type));
            }

            GameEvent::SecretDiscovered { secret_type, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("发现了{}！", secret_type));
            }

            GameEvent::ChestOpened { loot_count, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("打开宝箱，获得 {} 件物品", loot_count));
            }

            GameEvent::ShrineActivated { shrine_type, effect, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("激活{}：{}", shrine_type, effect));
            }

            GameEvent::TrapDisarmed { trap_type, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("成功解除{}陷阱", trap_type));
            }

            GameEvent::ExplosionTriggered { damage, radius, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("爆炸！半径 {} 造成 {} 点伤害", radius, damage));
            }

            // UI事件（通常由UI层直接处理，这里记录日志用于调试）
            GameEvent::UINotification { message, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(message.clone());
            }

            GameEvent::UIAlert { message, severity, .. } => {
                self.resources
                    .game_state
                    .message_log
                    .push(format!("[{}] {}", severity, message));
            }

            // 其他事件（静默处理，不产生日志）
            _ => {
                // 静默处理未匹配的事件，确保向后兼容性
                // 新事件不会导致panic，旧存档也能正常加载
            }
        }

        // 处理相关事件的成就跟踪
        self.handle_achievement_event(event);
    }

    /// 处理游戏事件的成就跟踪
    fn handle_achievement_event(&mut self, event: &GameEvent) {
        let newly_unlocked = match event {
            GameEvent::EntityDied { .. } => {
                // 跟踪敌人击杀数
                self.resources.achievements.on_kill()
            }

            GameEvent::LevelChanged { new_level, .. } => {
                // 跟踪到达的深度
                self.resources.achievements.on_level_change(*new_level)
            }

            GameEvent::ItemPickedUp { .. } => {
                // 跟踪收集的物品
                self.resources.achievements.on_item_pickup()
            }

            GameEvent::TurnEnded { turn } => {
                // 跟踪存活的回合数
                self.resources.achievements.on_turn_end(*turn)
            }

            GameEvent::BossDefeated { .. } => {
                // 跟踪击败的 Boss
                self.resources.achievements.on_boss_defeat()
            }

            _ => Vec::new(),
        };

        // 发布解锁通知
        for achievement_id in newly_unlocked {
            if let Some(achievement) = self.resources.achievements.get_achievement(achievement_id) {
                let message = format!(
                    "🏆 成就解锁: {} - {}",
                    achievement.name, achievement.description
                );
                self.event_bus.publish(GameEvent::LogMessage {
                    message,
                    level: LogLevel::Info,
                });
            }
        }
    }

    /// 帧结束时调用，准备处理下一帧事件
    pub fn next_frame(&mut self) {
        self.event_bus.next_frame();

        // 同步消息日志到 resources
        self.sync_message_log();
    }

    /// 同步事件处理器的消息日志到 Resources
    fn sync_message_log(&mut self) {
        // 这里可以从事件处理器获取日志并同步到 Resources
        // 目前保持简单实现
    }
}

// ========== 事件处理器实现 ==========

/// 游戏状态事件处理器
/// 负责处理游戏状态相关的事件，如伤害、死亡、物品使用等
pub struct GameStateHandler {
    message_log: Arc<Mutex<Vec<String>>>,
}

impl EventHandler for GameStateHandler {
    fn handle(&mut self, event: &GameEvent) {
        let message = match event {
            GameEvent::DamageDealt {
                damage,
                is_critical,
                ..
            } => Some(if *is_critical {
                format!("暴击！造成 {} 点伤害", damage)
            } else {
                format!("造成 {} 点伤害", damage)
            }),

            GameEvent::EntityDied { entity_name, .. } => Some(format!("{} 已死亡", entity_name)),

            GameEvent::ItemPickedUp { item_name, .. } => Some(format!("拾取了 {}", item_name)),

            GameEvent::ItemUsed {
                item_name, effect, ..
            } => Some(format!("使用了 {}，{}", item_name, effect)),

            GameEvent::LevelChanged {
                old_level,
                new_level,
            } => Some(format!("从第 {} 层进入第 {} 层", old_level, new_level)),

            GameEvent::GameOver { reason } => Some(format!("游戏结束：{}", reason)),

            GameEvent::Victory => Some("恭喜！你获得了胜利！".to_string()),

            GameEvent::LogMessage { message, level } => {
                let prefix = match level {
                    LogLevel::Debug => "[调试] ",
                    LogLevel::Info => "",
                    LogLevel::Warning => "[警告] ",
                    LogLevel::Error => "[错误] ",
                };
                Some(format!("{}{}", prefix, message))
            }

            GameEvent::TrapTriggered { trap_type, .. } => {
                Some(format!("触发了{}陷阱！", trap_type))
            }

            GameEvent::StatusApplied {
                status, duration, intensity, ..
            } => Some(format!("受到{}效果影响，持续{}回合（强度：{}）", status, duration, intensity)),

            GameEvent::StatusRemoved { status, reason, .. } => {
                if reason == "expired" {
                    Some(format!("{}效果已消失", status))
                } else {
                    Some(format!("{}效果被{}移除", status, reason))
                }
            }

            GameEvent::StatusEffectTicked {
                status, damage, remaining_turns, ..
            } => Some(format!("{}造成{}点伤害（剩余{}回合）", status, damage, remaining_turns)),

            GameEvent::StatusEffectConflict {
                removed_effect,
                new_effect,
                ..
            } => Some(format!("{}被{}覆盖", removed_effect, new_effect)),

            _ => None,
        };

        if let Some(msg) = message {
            if let Ok(mut log) = self.message_log.lock() {
                log.push(msg);
            }
        }
    }

    fn name(&self) -> &str {
        "GameStateHandler"
    }

    fn priority(&self) -> Priority {
        Priority::High
    }
}

/// 跨系统共享的全局资源
pub struct Resources {
    /// 游戏时间跟踪
    pub clock: GameClock,

    /// 当前游戏状态
    pub game_state: GameState,

    /// 玩家输入缓冲区
    pub input_buffer: InputBuffer,

    /// 游戏配置
    pub config: GameConfig,

    /// 随机数生成器状态
    pub rng: StdRng,

    /// 地牢状态标记实体（实际地牢存储为组件）
    pub dungeon: Option<hecs::Entity>,

    /// 成就管理器
    pub achievements: AchievementsManager,

    /// 战斗意图队列
    pub combat_intents: Vec<CombatIntent>,

    /// 后续处理队列（死亡、战利品、经验）
    pub aftermath_queue: Vec<AftermathEvent>,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            clock: GameClock::default(),
            game_state: GameState::default(),
            input_buffer: InputBuffer::default(),
            config: GameConfig::new(),
            rng: StdRng::seed_from_u64(12345), // 默认种子
            dungeon: None,
            achievements: AchievementsManager::new(),
            combat_intents: Vec::new(),
            aftermath_queue: Vec::new(),
        }
    }
}

impl Resources {
    /// 使用特定种子创建新的 Resources
    pub fn with_seed(seed: u64) -> Self {
        Self {
            clock: GameClock::default(),
            game_state: GameState::default(),
            input_buffer: InputBuffer::default(),
            config: GameConfig::new(),
            rng: StdRng::seed_from_u64(seed),
            dungeon: None,
            achievements: AchievementsManager::new(),
            combat_intents: Vec::new(),
            aftermath_queue: Vec::new(),
        }
    }

    /// 重新设置 RNG 种子（用于存档/读档）
    pub fn reseed_rng(&mut self, seed: u64) {
        self.rng = StdRng::seed_from_u64(seed);
    }
}

pub struct GameClock {
    pub current_time: std::time::SystemTime,
    pub elapsed_time: Duration,
    pub turn_count: u32,
    pub tick_rate: Duration,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            current_time: std::time::SystemTime::now(),
            elapsed_time: Duration::from_secs(0),
            turn_count: 0,
            tick_rate: Duration::from_millis(16), // ~60 FPS
        }
    }
}

#[derive(Default)]
pub struct GameState {
    pub game_state: GameStatus,
    pub depth: usize,
    pub message_log: Vec<String>,
    pub terminal_width: u16,
    pub terminal_height: u16,
    pub frame_count: u64,              // 渲染帧计数器，用于动画和缓存管理
    pub selected_class: Option<Class>, // 临时存储选中的职业，用于初始化游戏
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub enum GameStatus {
    #[default]
    Running,
    Paused {
        selected_option: usize,
    },
    GameOver {
        reason: GameOverReason,
    },
    Victory,
    MainMenu {
        selected_option: usize,
    },
    ClassSelection {
        cursor: usize,
    },
    Inventory {
        selected_item: usize,
    },
    Options {
        selected_option: usize,
    },
    Help,
    CharacterInfo,
    // 确认退出对话框
    ConfirmQuit {
        return_to: ReturnTo,
        selected_option: usize, // 0: 是, 1: 否
    },
}

/// 退出对话框返回目的地
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ReturnTo {
    Running,
    MainMenu,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameOverReason {
    Died(&'static str),     // 死亡原因 - 使用静态字符串避免Copy问题
    Defeated(&'static str), // 被敌人击败
    Starved,                // 饿死
    Trapped(&'static str),  // 陷阱
    Quit,                   // 主动退出
}

impl Default for GameOverReason {
    fn default() -> Self {
        GameOverReason::Died("未知原因")
    }
}

#[derive(Default)]
pub struct InputBuffer {
    pub pending_actions: Vec<PlayerAction>,
    /// 本帧成功完成且需要扣除能量的动作
    pub completed_actions: Vec<PlayerAction>,
}

#[derive(Clone, Debug)]
pub enum PlayerAction {
    Move(Direction),
    Attack(Position),
    UseItem(usize),
    DropItem(usize),
    EquipItem(usize),
    UnequipItem(usize),
    ThrowItem(usize, Direction),
    Descend,
    Ascend,
    Wait,
    Quit,

    // 菜单相关动作
    OpenInventory,
    OpenOptions,
    OpenHelp,
    OpenCharacterInfo,
    CloseMenu,

    // 菜单导航
    MenuNavigate(NavigateDirection),
    MenuSelect,
    MenuBack,
}

#[derive(Clone, Copy, Debug)]
pub enum NavigateDirection {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// 战斗意图 - 在战斗阶段执行的攻击行为
#[derive(Clone, Debug)]
pub struct CombatIntent {
    pub attacker: Entity,
    pub defender: Entity,
    pub attacker_pos: Position,
    pub defender_pos: Position,
    pub is_player: bool,
    pub priority: u32, // 更高优先级先执行
}

impl CombatIntent {
    pub fn new(
        attacker: Entity,
        defender: Entity,
        attacker_pos: Position,
        defender_pos: Position,
        is_player: bool,
    ) -> Self {
        Self {
            attacker,
            defender,
            attacker_pos,
            defender_pos,
            is_player,
            priority: if is_player { 1000 } else { 100 },
        }
    }
}

/// 战斗结果类型
#[derive(Clone, Debug)]
pub enum CombatOutcome {
    Hit {
        damage: u32,
        is_critical: bool,
        is_ambush: bool,
    },
    Miss,
    Counter {
        damage: u32,
        is_critical: bool,
    },
    ChainAttack {
        damage: u32,
        is_critical: bool,
    },
}

/// 后续处理事件 - 在后续阶段处理的事件
#[derive(Clone, Debug)]
pub enum AftermathEvent {
    Death {
        entity: Entity,
        entity_id: u32,
        entity_name: String,
        killer: Option<Entity>,
    },
    LootDrop {
        entity: Entity,
        position: Position,
    },
    ExperienceGain {
        entity: Entity,
        amount: u32,
    },
}

#[derive(Default)]
pub struct GameConfig {
    pub fov_range: u8,
    pub max_depth: usize,
    pub save_directory: String,
}

impl GameConfig {
    pub fn new() -> Self {
        Self {
            fov_range: 8,
            max_depth: 10,
            save_directory: "saves".to_string(),
        }
    }
}

// 玩家标记组件
#[derive(Clone, Debug)]
pub struct Player;

// 基础组件
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32, // 地牢层数
}

impl Position {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub terrain_type: TerrainType,
    pub is_passable: bool,
    pub blocks_sight: bool,
    pub has_items: bool,
    pub has_monster: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TerrainType {
    Floor,
    Wall,
    Door,
    StairsDown,
    StairsUp,
    Water,
    Trap,
    Barrel,
    Empty,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Renderable {
    pub symbol: char,
    pub fg_color: Color,
    pub bg_color: Option<Color>,
    pub order: u8, // 渲染顺序
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Color {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    White,
    Black,
    Reset,
    Rgb(u8, u8, u8),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Actor {
    pub name: String,
    pub faction: Faction,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Faction {
    Player,
    Enemy,
    Neutral,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub hp: u32,
    pub max_hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub accuracy: u32,
    pub evasion: u32,
    pub level: u32,
    pub experience: u32,
    #[serde(default)]
    pub class: Option<Class>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<ItemSlot>,
    pub max_slots: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemSlot {
    pub item: Option<ECSItem>,
    pub quantity: u32,
}

/// 增强的 ECS 物品组件（支持 items 模块的完整功能）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ECSItem {
    pub name: String,
    pub item_type: ItemType,
    pub value: u32,
    pub identified: bool,

    // ========== 扩展属性（支持 items 模块） ==========
    pub quantity: u32,        // 堆叠数量（药水、卷轴、食物等）
    pub level: i32,           // 升级等级（武器、护甲）
    pub cursed: bool,         // 是否被诅咒
    pub charges: Option<u32>, // 充能次数（法杖、魔法石）

    /// 详细数据（可选）：序列化的 items::Item
    /// 用于存储完整的 items 模块对象，实现完全兼容
    pub detailed_data: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ItemType {
    Weapon { damage: u32 },
    Armor { defense: u32 },
    Consumable { effect: ConsumableEffect },
    Throwable { damage: (u32, u32), range: u8 },
    Key,
    Quest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConsumableEffect {
    Healing {
        amount: u32,
    },
    Damage {
        amount: u32,
    },
    Buff {
        stat: StatType,
        value: i32,
        duration: u32,
    },
    Teleport,
    Identify,
}

impl ECSItem {
    /// 创建基础物品（不带详细数据）
    pub fn new_basic(name: String, item_type: ItemType, value: u32) -> Self {
        Self {
            name,
            item_type,
            value,
            identified: false,
            quantity: 1,
            level: 0,
            cursed: false,
            charges: None,
            detailed_data: None,
        }
    }

    /// 从 items::Item 创建 ECSItem（包含完整数据）
    pub fn from_items_item(item: &items::Item) -> Result<Self, Box<dyn std::error::Error>> {
        // 序列化完整的 items::Item
        let detailed_data = bincode::encode_to_vec(item, bincode::config::standard())?;

        // 映射基础类型
        let item_type = Self::map_item_kind_to_type(&item.kind);

        Ok(Self {
            name: item.name.clone(),
            item_type,
            value: item.value(),
            identified: !item.needs_identify(),
            quantity: item.quantity,
            level: 0,      // items::Item 没有直接的 level 字段
            cursed: false, // 需要根据具体物品类型判断
            charges: None, // 需要根据具体物品类型提取
            detailed_data: Some(detailed_data),
        })
    }

    /// 将 items::ItemKind 映射到 ItemType
    fn map_item_kind_to_type(kind: &items::ItemKind) -> ItemType {
        match kind {
            items::ItemKind::Weapon(w) => ItemType::Weapon {
                damage: w.damage.0, // 使用 damage 元组的第一个值（最小伤害）
            },
            items::ItemKind::Armor(a) => ItemType::Armor {
                defense: a.defense as u32,
            },
            items::ItemKind::Potion(_) => ItemType::Consumable {
                effect: ConsumableEffect::Healing { amount: 10 }, // 简化处理
            },
            items::ItemKind::Food(_) => ItemType::Consumable {
                effect: ConsumableEffect::Healing { amount: 5 },
            },
            items::ItemKind::Scroll(_) => ItemType::Consumable {
                effect: ConsumableEffect::Identify,
            },
            items::ItemKind::Throwable(t) => ItemType::Throwable {
                damage: t.damage,
                range: t.range,
            },
            items::ItemKind::Herb(_) => ItemType::Consumable {
                effect: ConsumableEffect::Healing { amount: 8 },
            },
            _ => ItemType::Quest, // 其他类型映射为任务物品
        }
    }

    /// 转换回 items::Item（如果有详细数据）
    pub fn to_items_item(&self) -> Result<items::Item, Box<dyn std::error::Error>> {
        if let Some(ref data) = self.detailed_data {
            let (item, _): (items::Item, _) =
                bincode::decode_from_slice(data, bincode::config::standard())?;
            Ok(item)
        } else {
            Err("No detailed data available".into())
        }
    }

    /// 是否为可堆叠物品
    pub fn is_stackable(&self) -> bool {
        matches!(
            self.item_type,
            ItemType::Consumable { .. } | ItemType::Throwable { .. }
        )
    }

    /// 是否可用
    pub fn is_usable(&self) -> bool {
        matches!(
            self.item_type,
            ItemType::Consumable { .. } | ItemType::Throwable { .. }
        )
    }

    /// 是否可装备
    pub fn is_equippable(&self) -> bool {
        matches!(
            self.item_type,
            ItemType::Weapon { .. } | ItemType::Armor { .. }
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatType {
    Hp,
    Attack,
    Defense,
    Accuracy,
    Evasion,
}

/// FOV（视野）算法类型
///
/// 支持三种经典 Roguelike 视野算法：
/// - ShadowCasting: 阴影投射（最真实，性能中等）
/// - DiamondWalls: 菱形墙算法（适合正交移动）
/// - RayCasting: 光线投射/Bresenham（性能最优）
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FovAlgorithm {
    /// 阴影投射算法（默认，最真实）
    ShadowCasting,
    /// 菱形墙算法（适合正交地图）
    DiamondWalls,
    /// 光线投射/Bresenham算法（性能最佳）
    RayCasting,
}

impl Default for FovAlgorithm {
    fn default() -> Self {
        FovAlgorithm::ShadowCasting
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Viewshed {
    pub range: u8,
    pub visible_tiles: Vec<Position>,
    pub memory: Vec<Position>, // previously seen tiles
    pub dirty: bool,
    pub algorithm: FovAlgorithm, // 使用的 FOV 算法
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Energy {
    pub current: u32,
    pub max: u32,
    pub regeneration_rate: u32,
}

/// Status effects component - stores active effects on an entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusEffects {
    pub effects: Vec<combat::effect::Effect>,
    pub last_tick_turn: u32, // Track when effects were last processed
}

impl StatusEffects {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            last_tick_turn: 0,
        }
    }

    pub fn add_effect(&mut self, new_effect: combat::effect::Effect) {
        use combat::effect::EffectType;

        // Check for conflicts (mutually exclusive effects)
        if self.has_conflicting_effect(new_effect.effect_type()) {
            // Remove conflicting effects
            self.remove_conflicting_effects(new_effect.effect_type());
        }

        // Check if effect is stackable
        if new_effect.is_stackable() {
            // For DoT/HoT effects, add as separate instance
            self.effects.push(new_effect);
        } else {
            // For non-stackable effects, replace or extend existing
            if let Some(existing) = self.effects.iter_mut().find(|e| e.effect_type() == new_effect.effect_type()) {
                // Take max duration and intensity
                let max_turns = existing.turns().max(new_effect.turns());
                let max_intensity = existing.intensity().max(new_effect.intensity());
                *existing = combat::effect::Effect::with_intensity(
                    new_effect.effect_type(),
                    max_turns,
                    max_intensity
                );
            } else {
                self.effects.push(new_effect);
            }
        }
    }

    pub fn has_conflicting_effect(&self, effect_type: EffectType) -> bool {
        use combat::effect::EffectType;
        self.effects.iter().any(|e| {
            matches!(
                (e.effect_type(), effect_type),
                (EffectType::Burning, EffectType::Frost)
                    | (EffectType::Frost, EffectType::Burning)
                    | (EffectType::Haste, EffectType::Slow)
                    | (EffectType::Slow, EffectType::Haste)
                    | (EffectType::Invisibility, EffectType::Light)
                    | (EffectType::Light, EffectType::Invisibility)
            )
        })
    }

    pub fn remove_conflicting_effects(&mut self, effect_type: EffectType) {
        use combat::effect::EffectType;
        self.effects.retain(|e| {
            !matches!(
                (e.effect_type(), effect_type),
                (EffectType::Burning, EffectType::Frost)
                    | (EffectType::Frost, EffectType::Burning)
                    | (EffectType::Haste, EffectType::Slow)
                    | (EffectType::Slow, EffectType::Haste)
                    | (EffectType::Invisibility, EffectType::Light)
                    | (EffectType::Light, EffectType::Invisibility)
            )
        });
    }

    pub fn has_effect(&self, effect_type: EffectType) -> bool {
        self.effects.iter().any(|e| e.effect_type() == effect_type)
    }

    pub fn remove_effect(&mut self, effect_type: EffectType) {
        self.effects.retain(|e| e.effect_type() != effect_type);
    }

    pub fn clear(&mut self) {
        self.effects.clear();
    }
}

impl Default for StatusEffects {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export EffectType for convenience
pub use combat::effect::EffectType;

#[derive(Clone, Debug)]
pub struct AI {
    pub ai_type: AIType,
    pub target: Option<Entity>,
    pub state: AIState,
}

impl AI {
    pub fn range(&self) -> u8 {
        match &self.ai_type {
            AIType::Aggressive => 10, // Default aggressive range
            AIType::Passive => 2,
            AIType::Neutral => 5,
            AIType::Patrol { .. } => 10,
        }
    }
}

// AI cannot be serialized due to Entity type

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AIType {
    Passive,
    Aggressive,
    Neutral,
    Patrol { path: Vec<Position> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AIState {
    Idle,
    Patrolling,
    Chasing,
    Fleeing,
    Attacking,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Effects {
    pub active_effects: Vec<ActiveEffect>,
}

// ========== 新增组件：玩家专属属性 ==========

/// 饥饿系统组件（模拟 SPD 的饱食度机制）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hunger {
    pub satiety: u8,           // 饱食度（0-10，SPD标准）
    pub last_hunger_turn: u32, // 上次饥饿减少的回合数
    #[serde(default)]
    pub turn_accumulator: u32, // 累积的回合数（用于更精确的饥饿计算）
}

impl Default for Hunger {
    fn default() -> Self {
        Self {
            satiety: 5, // 默认半饱状态
            last_hunger_turn: 0,
            turn_accumulator: 0,
        }
    }
}

impl Hunger {
    pub fn new(satiety: u8) -> Self {
        Self {
            satiety: satiety.min(10),
            last_hunger_turn: 0,
            turn_accumulator: 0,
        }
    }

    /// 是否处于饥饿状态
    pub fn is_starving(&self) -> bool {
        self.satiety == 0
    }

    /// 是否处于饥饿警告状态
    pub fn is_hungry(&self) -> bool {
        self.satiety <= 2
    }

    /// 进食恢复饱食度
    pub fn feed(&mut self, amount: u8) {
        self.satiety = (self.satiety + amount).min(10);
    }

    /// 每回合自动减少饱食度（每20回合减1）
    pub fn on_turn(&mut self, current_turn: u32) {
        if current_turn - self.last_hunger_turn >= 20 {
            self.satiety = self.satiety.saturating_sub(1);
            self.last_hunger_turn = current_turn;
        }
    }
}

/// 财富组件（金币系统）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wealth {
    pub gold: u32,
}

impl Default for Wealth {
    fn default() -> Self {
        Self { gold: 0 }
    }
}

impl Wealth {
    pub fn new(gold: u32) -> Self {
        Self { gold }
    }

    pub fn add_gold(&mut self, amount: u32) {
        self.gold = self.gold.saturating_add(amount);
    }

    pub fn remove_gold(&mut self, amount: u32) -> bool {
        if self.gold >= amount {
            self.gold -= amount;
            true
        } else {
            false
        }
    }

    pub fn can_afford(&self, amount: u32) -> bool {
        self.gold >= amount
    }
}

/// 玩家进度组件（回合、力量、职业等）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerProgress {
    pub turns: u32,   // 游戏总回合数
    pub strength: u8, // 力量值（影响装备需求）
    pub class: Class, // 职业类型
    #[serde(default)]
    pub skill_state: SkillState, // 职业技能状态
}

impl Default for PlayerProgress {
    fn default() -> Self {
        Self {
            turns: 0,
            strength: 10,
            class: Class::default(),
            skill_state: SkillState::default(),
        }
    }
}

impl PlayerProgress {
    pub fn new(strength: u8, class: Class, skill_state: SkillState) -> Self {
        Self {
            turns: 0,
            strength,
            class,
            skill_state,
        }
    }

    pub fn advance_turn(&mut self) {
        self.turns += 1;
    }

    pub fn add_strength(&mut self, amount: u8) {
        self.strength = self.strength.saturating_add(amount);
    }
}

// ========== Boss 相关组件 ==========

/// Boss 标记组件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BossComponent {
    pub boss_type: combat::boss::BossType,
    pub current_phase: combat::boss::BossPhase,
    pub shield: u32,
}

/// Boss 技能冷却组件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BossSkillComponent {
    pub cooldowns: combat::boss::SkillCooldowns,
    pub available_skills: Vec<combat::boss::BossSkill>,
}

/// Boss 击败记录组件（记录玩家击败的 Boss）
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BossDefeatRecord {
    pub defeated_bosses: Vec<combat::boss::BossType>,
    pub first_kill_rewards_claimed: Vec<combat::boss::BossType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub effect_type: combat::effect::EffectType,
    pub duration: u32,
    pub intensity: u32,
}

// Functions to convert between ECS components and hero module structures
impl From<&Stats> for Hero {
    fn from(stats: &Stats) -> Self {
        let class = stats.class.clone().unwrap_or_default();
        let mut hero = Hero::with_seed(class, 12345);
        hero.hp = stats.hp;
        hero.max_hp = stats.max_hp;
        hero.base_attack = stats.attack;
        hero.base_defense = stats.defense;
        hero.level = stats.level;
        hero.experience = stats.experience;
        hero
    }
}

impl From<&Hero> for Stats {
    fn from(hero: &Hero) -> Self {
        Self {
            hp: hero.hp,
            max_hp: hero.max_hp,
            attack: hero.base_attack,
            defense: hero.base_defense,
            accuracy: 80, // Default accuracy
            evasion: 20,  // Default evasion
            level: hero.level,
            experience: hero.experience,
            class: Some(hero.class.clone()),
        }
    }
}

// ========== 新增：Hero 到新组件的转换 ==========

impl From<&Hero> for Hunger {
    fn from(hero: &Hero) -> Self {
        Self {
            satiety: hero.satiety,
            last_hunger_turn: 0,
            turn_accumulator: 0,
        }
    }
}

impl From<&Hero> for Wealth {
    fn from(hero: &Hero) -> Self {
        Self { gold: hero.gold }
    }
}

impl From<&Hero> for PlayerProgress {
    fn from(hero: &Hero) -> Self {
        Self {
            turns: hero.turns,
            strength: hero.strength,
            class: hero.class.clone(),
            skill_state: hero.class_skills.clone(),
        }
    }
}

impl ECSWorld {
    /// Convert ECS world to save data
    pub fn to_save_data(
        &self,
        turn_system: &crate::turn_system::TurnSystem,
    ) -> Result<SaveData, GameError> {
        // Extract hero data from ECS
        let mut hero: Option<Hero> = None;

        // Find the player entity and convert to hero
        if let Some((entity, _player_marker)) = self.world.query::<&Player>().iter().next() {
            // 从各个组件构建 Hero
            let mut new_hero = if let Ok(stats) = self.world.get::<&Stats>(entity) {
                Hero::from(&*stats)
            } else {
                Hero::default()
            };

            // 从 Inventory 组件恢复 bag
            if let Ok(inventory) = self.world.get::<&Inventory>(entity) {
                new_hero.bag = Bag::from(&*inventory);
            }

            // 从 Position 组件恢复位置
            if let Ok(pos) = self.world.get::<&Position>(entity) {
                new_hero.x = pos.x;
                new_hero.y = pos.y;
            }

            // ========== 新增：从新组件恢复数据 ==========

            // 从 Hunger 组件恢复饱食度
            if let Ok(hunger) = self.world.get::<&Hunger>(entity) {
                new_hero.satiety = hunger.satiety;
            }

            // 从 Wealth 组件恢复金币
            if let Ok(wealth) = self.world.get::<&Wealth>(entity) {
                new_hero.gold = wealth.gold;
            }

            // 从 PlayerProgress 组件恢复进度信息
            if let Ok(progress) = self.world.get::<&PlayerProgress>(entity) {
                new_hero.turns = progress.turns;
                new_hero.strength = progress.strength;
                new_hero.class = progress.class.clone();
                new_hero.class_skills = progress.skill_state.clone();
            }

            hero = Some(new_hero);
        }

        // Extract dungeon data
        let dungeon = get_dungeon_clone(&self.world).ok_or_else(|| GameError::InvalidLevelData)?;

        let hero = hero.ok_or_else(|| GameError::InvalidHeroData)?;
        let hero_class = hero.class.clone();
        let hero_skill_state = hero.class_skills.clone();

        // Extract player energy and hunger state
        let mut player_energy = 100u32;
        let mut player_hunger_last_turn = 0u32;
        if let Some((entity, _player_marker)) = self.world.query::<&Player>().iter().next() {
            if let Ok(energy) = self.world.get::<&Energy>(entity) {
                player_energy = energy.current;
            }
            if let Ok(hunger) = self.world.get::<&Hunger>(entity) {
                player_hunger_last_turn = hunger.last_hunger_turn;
            }
        }

        // Extract turn system state
        let turn_state = save::TurnStateData {
            current_phase: match turn_system.state {
                crate::turn_system::TurnState::PlayerTurn => save::TurnPhase::PlayerTurn,
                crate::turn_system::TurnState::ProcessingPlayerAction => {
                    save::TurnPhase::ProcessingPlayerAction
                }
                crate::turn_system::TurnState::AITurn => save::TurnPhase::AITurn,
                crate::turn_system::TurnState::ProcessingAIActions => {
                    save::TurnPhase::ProcessingAIActions
                }
            },
            player_action_taken: turn_system.player_action_taken(),
        };

        // Extract clock state
        let clock_state = save::ClockStateData {
            turn_count: self.resources.clock.turn_count,
            elapsed_time_secs: self.resources.clock.elapsed_time.as_secs_f64(),
        };

        // Extract non-player entity states (enemies, NPCs, etc.)
        let mut entities = Vec::new();
        for (entity, (pos, actor, stats)) in
            self.world.query::<(&Position, &Actor, &Stats)>().iter()
        {
            // Skip player entity
            if actor.faction == Faction::Player {
                continue;
            }

            // Get energy state
            let (energy_current, energy_max, energy_regen) =
                if let Ok(energy) = self.world.get::<&Energy>(entity) {
                    (energy.current, energy.max, energy.regeneration_rate)
                } else {
                    (100, 100, 1)
                };

            // Get active effects
            let mut active_effects = Vec::new();
            if let Ok(effects) = self.world.get::<&Effects>(entity) {
                for effect in &effects.active_effects {
                    active_effects.push(save::StatusEffectData {
                        effect_type: format!("{:?}", effect.effect_type),
                        duration: effect.duration,
                        intensity: effect.intensity,
                    });
                }
            }

            entities.push(save::EntityStateData {
                position: (pos.x, pos.y, pos.z),
                name: actor.name.clone(),
                hp: stats.hp,
                max_hp: stats.max_hp,
                energy_current,
                energy_max,
                energy_regen,
                active_effects,
            });
        }

        // Create save data
        let save_data = SaveData {
            version: save::SAVE_VERSION,
            metadata: save::SaveMetadata {
                timestamp: std::time::SystemTime::now(),
                dungeon_depth: self.resources.game_state.depth,
                hero_name: hero.name.clone(),
                hero_class,
                play_time: self.resources.clock.elapsed_time.as_secs_f64(),
            },
            hero_skill_state,
            hero,
            dungeon,
            game_seed: 0, // 需要保存实际的种子值
            turn_state,
            clock_state,
            player_energy,
            player_hunger_last_turn,
            entities,
        };

        Ok(save_data)
    }

    /// Load data from save into ECS world
    /// Returns (turn_state, turn_action_taken) for restoring the turn system
    pub fn from_save_data(
        &mut self,
        save_data: SaveData,
    ) -> Result<(crate::turn_system::TurnState, bool), GameError> {
        // Clear current world
        self.clear();

        // Set up resources from save data
        self.resources.rng = StdRng::seed_from_u64(save_data.game_seed);
        self.resources.game_state.depth = save_data.metadata.dungeon_depth;

        // Restore clock state
        self.resources.clock.turn_count = save_data.clock_state.turn_count;
        self.resources.clock.elapsed_time =
            Duration::from_secs_f64(save_data.clock_state.elapsed_time_secs);

        set_dungeon_instance(&mut self.world, save_data.dungeon);

        // Convert hero to ECS components and spawn player entity
        let mut hero = save_data.hero;
        hero.class = save_data.metadata.hero_class.clone();
        hero.class_skills = save_data.hero_skill_state.clone();

        let stats: Stats = (&hero).into();
        let inventory: Inventory = (&hero.bag).into();

        // ========== 新增：创建新组件 ==========
        let mut hunger: Hunger = (&hero).into();
        hunger.last_hunger_turn = save_data.player_hunger_last_turn;
        let wealth: Wealth = (&hero).into();
        let progress: PlayerProgress = (&hero).into();

        // Spawn player entity with converted components（包含新组件）
        self.world.spawn((
            Position::new(hero.x, hero.y, save_data.metadata.dungeon_depth as i32),
            Actor {
                name: hero.name.clone(),
                faction: Faction::Player,
            },
            Renderable {
                symbol: '@',
                fg_color: Color::Yellow,
                bg_color: Some(Color::Black),
                order: 10,
            },
            stats,
            inventory,
            hunger,   // 新增：饱食度组件
            wealth,   // 新增：财富组件
            progress, // 新增：玩家进度组件
            Viewshed {
                range: 8,
                visible_tiles: vec![],
                memory: vec![],
                dirty: true,
                algorithm: FovAlgorithm::default(), // 使用默认算法（ShadowCasting）
            },
            Energy {
                current: save_data.player_energy,
                max: 100,
                regeneration_rate: 1,
            },
            Player, // Player marker component
        ));

        // Restore non-player entities (enemies, NPCs, etc.)
        // Note: Full entity restoration would require more complex logic
        // For now, we'll skip this and let the game regenerate enemies
        // In a production system, you'd want to restore all entity data here

        // Convert turn state back
        let turn_state = match save_data.turn_state.current_phase {
            save::TurnPhase::PlayerTurn => crate::turn_system::TurnState::PlayerTurn,
            save::TurnPhase::ProcessingPlayerAction => {
                crate::turn_system::TurnState::ProcessingPlayerAction
            }
            save::TurnPhase::AITurn => crate::turn_system::TurnState::AITurn,
            save::TurnPhase::ProcessingAIActions => {
                crate::turn_system::TurnState::ProcessingAIActions
            }
        };

        Ok((turn_state, save_data.turn_state.player_action_taken))
    }
}

// Dungeon component and helper APIs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DungeonComponent(pub dungeon::Dungeon);

/// Get a cloned dungeon instance from the world if present
pub fn get_dungeon_clone(world: &World) -> Option<dungeon::Dungeon> {
    world
        .query::<&DungeonComponent>()
        .iter()
        .next()
        .map(|(_, dungeon_comp)| dungeon_comp.0.clone())
}

/// Set or replace the dungeon instance in the world. If no dungeon entity exists, one is created.
pub fn set_dungeon_instance(world: &mut World, dungeon: dungeon::Dungeon) {
    // Collect entity ids into a temporary vector to avoid holding a QueryBorrow while mutating
    let existing_entities: Vec<_> = world
        .query::<&DungeonComponent>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    if let Some(&entity) = existing_entities.first() {
        let _ = world.remove_one::<DungeonComponent>(entity);
        let _ = world.insert_one(entity, DungeonComponent(dungeon));
        return;
    }

    // No existing dungeon component, spawn a new entity with it
    let _ = world.spawn((DungeonComponent(dungeon),));
}

impl From<&Inventory> for Bag {
    fn from(inventory: &Inventory) -> Self {
        use game_items::ItemTrait;

        let mut bag = Bag::new();

        for slot in &inventory.items {
            if let Some(item) = &slot.item {
                if let Ok(mut source_item) = item.to_items_item() {
                    let quantity = slot.quantity.max(1);

                    if source_item.is_stackable() {
                        source_item.quantity = 1;
                    }

                    for _ in 0..quantity {
                        let _ = bag.add_item(source_item.clone());
                    }

                    continue;
                }

                let fallback_kind = match &item.item_type {
                    ItemType::Weapon { .. } => game_items::ItemKind::Weapon(
                        game_items::Weapon::new(1, game_items::weapon::WeaponKind::Dagger),
                    ),
                    ItemType::Armor { .. } => {
                        game_items::ItemKind::Armor(game_items::Armor::new(1))
                    }
                    ItemType::Consumable { .. } => game_items::ItemKind::Potion(
                        game_items::Potion::new_alchemy(game_items::potion::PotionKind::Healing),
                    ),
                    ItemType::Throwable { .. } => game_items::ItemKind::Throwable(
                        game_items::Throwable::new(game_items::ThrowableKind::Dart),
                    ),
                    ItemType::Key => game_items::ItemKind::Misc(game_items::MiscItem::new(
                        game_items::misc::MiscKind::Torch,
                    )),
                    ItemType::Quest => game_items::ItemKind::Misc(game_items::MiscItem::new(
                        game_items::misc::MiscKind::Gold(10),
                    )),
                };

                let fallback_item = game_items::Item::new(fallback_kind);
                let iterations = slot.quantity.max(1);
                for _ in 0..iterations {
                    let _ = bag.add_item(fallback_item.clone());
                }
            }
        }

        bag
    }
}

/// Convenience helper to get a mutable dungeon reference and run a closure on it
pub fn with_dungeon_mut<F>(world: &mut World, f: F)
where
    F: FnOnce(&mut dungeon::Dungeon),
{
    // Collect entity ids to avoid holding the query borrow while mutating
    let ids: Vec<_> = world
        .query::<&DungeonComponent>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    if let Some(&entity) = ids.first() {
        if let Ok(mut comp) = world.get::<&mut DungeonComponent>(entity) {
            f(&mut comp.0);
        }
    }
}

impl From<&Bag> for Inventory {
    fn from(bag: &Bag) -> Self {
        let mut items: Vec<ItemSlot> = Vec::new();

        const BAG_DEFAULT_CAPACITY: usize = 64;

        fn push_from_collection(
            collection: Vec<(game_items::Item, u32)>,
            slots: &mut Vec<ItemSlot>,
        ) {
            for (item, count) in collection {
                if let Ok(mut ecs_item) = ECSItem::from_items_item(&item) {
                    let quantity = count.max(1);
                    ecs_item.quantity = quantity;
                    ecs_item.identified = !item.needs_identify();
                    slots.push(ItemSlot {
                        item: Some(ecs_item),
                        quantity,
                    });
                }
            }
        }

        push_from_collection(
            bag.weapons()
                .items()
                .into_iter()
                .map(|(weapon, count)| (game_items::Item::from(weapon), count))
                .collect(),
            &mut items,
        );

        push_from_collection(
            bag.armors()
                .items()
                .into_iter()
                .map(|(armor, count)| (game_items::Item::from(armor), count))
                .collect(),
            &mut items,
        );

        push_from_collection(
            bag.potions()
                .items()
                .into_iter()
                .map(|(potion, count)| (game_items::Item::from(potion), count))
                .collect(),
            &mut items,
        );

        push_from_collection(
            bag.scrolls()
                .items()
                .into_iter()
                .map(|(scroll, count)| (game_items::Item::from(scroll), count))
                .collect(),
            &mut items,
        );

        push_from_collection(
            bag.wands()
                .items()
                .into_iter()
                .map(|(wand, count)| (game_items::Item::from(wand), count))
                .collect(),
            &mut items,
        );

        push_from_collection(
            bag.rings()
                .items()
                .into_iter()
                .map(|(ring, count)| (game_items::Item::from(ring), count))
                .collect(),
            &mut items,
        );

        push_from_collection(
            bag.seeds()
                .items()
                .into_iter()
                .map(|(seed, count)| (game_items::Item::from(seed), count))
                .collect(),
            &mut items,
        );

        push_from_collection(
            bag.stones()
                .items()
                .into_iter()
                .map(|(stone, count)| (game_items::Item::from(stone), count))
                .collect(),
            &mut items,
        );

        push_from_collection(
            bag.food()
                .items()
                .into_iter()
                .map(|(food, count)| (game_items::Item::from(food), count))
                .collect(),
            &mut items,
        );

        push_from_collection(
            bag.misc()
                .items()
                .into_iter()
                .map(|(misc, count)| (game_items::Item::from(misc), count))
                .collect(),
            &mut items,
        );

        push_from_collection(
            bag.throwables()
                .items()
                .into_iter()
                .map(|(throwable, count)| (game_items::Item::from(throwable), count))
                .collect(),
            &mut items,
        );

        push_from_collection(
            bag.herbs()
                .items()
                .into_iter()
                .map(|(herb, count)| (game_items::Item::from(herb), count))
                .collect(),
            &mut items,
        );

        let item_count = items.len();
        Inventory {
            items,
            max_slots: BAG_DEFAULT_CAPACITY.max(item_count + 8),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::GameEvent;

    #[test]
    fn test_event_bus_integration() {
        let mut world = ECSWorld::new();

        // 测试事件发布
        world.publish_event(GameEvent::LogMessage {
            message: "测试消息".to_string(),
            level: LogLevel::Info,
        });

        assert_eq!(world.event_bus.len(), 1);

        // 测试事件处理
        world.process_events();

        // 检查日志是否被添加
        assert_eq!(world.resources.game_state.message_log.len(), 1);
        assert_eq!(world.resources.game_state.message_log[0], "测试消息");

        // 事件应该被清空
        assert_eq!(world.event_bus.len(), 0);
    }

    #[test]
    fn test_combat_events() {
        let mut world = ECSWorld::new();

        // 创建玩家和敌人实体
        let player = world.world.spawn((
            Position::new(0, 0, 0),
            Actor {
                name: "Player".to_string(),
                faction: Faction::Player,
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
                class: Some(Class::Warrior),
            },
        ));

        let enemy = world.world.spawn((
            Position::new(1, 0, 0),
            Actor {
                name: "Goblin".to_string(),
                faction: Faction::Enemy,
            },
            Stats {
                hp: 30,
                max_hp: 30,
                attack: 5,
                defense: 2,
                accuracy: 60,
                evasion: 10,
                level: 1,
                experience: 0,
                class: None,
            },
        ));

        // 发布战斗开始事件
        world.publish_event(GameEvent::CombatStarted {
            attacker: player.id(),
            defender: enemy.id(),
        });

        // 发布伤害事件
        world.publish_event(GameEvent::DamageDealt {
            attacker: player.id(),
            victim: enemy.id(),
            damage: 10,
            is_critical: false,
        });

        // 处理事件
        world.process_events();

        // 检查日志
        assert!(world.resources.game_state.message_log.len() > 0);
        assert!(world.resources.game_state.message_log[0].contains("造成 10 点伤害"));
    }

    #[test]
    fn test_delayed_events() {
        let mut world = ECSWorld::new();

        // 发布延迟事件
        world.publish_delayed_event(GameEvent::LogMessage {
            message: "延迟消息".to_string(),
            level: LogLevel::Info,
        });

        // 当前帧应该没有事件
        assert_eq!(world.event_bus.len(), 0);

        // 移到下一帧
        world.next_frame();

        // 现在应该有事件了
        assert_eq!(world.event_bus.len(), 1);

        // 处理事件
        world.process_events();

        // 检查日志
        assert_eq!(world.resources.game_state.message_log.len(), 1);
        assert_eq!(world.resources.game_state.message_log[0], "延迟消息");
    }

    #[test]
    fn test_game_over_event() {
        let mut world = ECSWorld::new();

        // 初始状态应该是 Running
        assert_eq!(world.resources.game_state.game_state, GameStatus::Running);

        // 发布游戏结束事件
        world.publish_event(GameEvent::GameOver {
            reason: "测试失败".to_string(),
        });

        // 处理事件
        world.process_events();

        // 检查游戏状态
        assert!(matches!(
            world.resources.game_state.game_state,
            GameStatus::GameOver { .. }
        ));
        assert!(
            world
                .resources
                .game_state
                .message_log
                .iter()
                .any(|msg| msg.contains("游戏结束"))
        );
    }

    #[test]
    fn test_level_change_event() {
        let mut world = ECSWorld::new();

        // 初始深度为 0
        assert_eq!(world.resources.game_state.depth, 0);

        // 发布层级变化事件
        world.publish_event(GameEvent::LevelChanged {
            old_level: 1,
            new_level: 2,
        });

        // 处理事件
        world.process_events();

        // 检查深度是否更新
        assert_eq!(world.resources.game_state.depth, 2);
        assert!(
            world
                .resources
                .game_state
                .message_log
                .iter()
                .any(|msg| msg.contains("从第 1 层进入第 2 层"))
        );
    }

    #[test]
    fn test_herb_and_throwable_roundtrip_conversion() {
        let mut bag = Bag::new();

        let mut herb_item = game_items::Item::new(game_items::ItemKind::Herb(
            game_items::Herb::new(game_items::HerbKind::Sungrass),
        ));
        if let game_items::ItemKind::Herb(ref mut herb) = herb_item.kind {
            herb.quantity = 3;
        }
        herb_item.quantity = 3;
        bag.add_item(herb_item).expect("failed to add herb stack");

        let mut throwable_item = game_items::Item::new(game_items::ItemKind::Throwable(
            game_items::Throwable::new(game_items::ThrowableKind::Shuriken),
        ));
        if let game_items::ItemKind::Throwable(ref mut throwable) = throwable_item.kind {
            throwable.quantity = 4;
        }
        throwable_item.quantity = 4;
        bag.add_item(throwable_item)
            .expect("failed to add throwable stack");

        let inventory: Inventory = (&bag).into();
        let reconstructed: Bag = (&inventory).into();

        let herb_total: u32 = reconstructed
            .herbs()
            .items()
            .into_iter()
            .map(|(_, count)| count)
            .sum();
        assert_eq!(herb_total, 3);

        let throwable_total: u32 = reconstructed
            .throwables()
            .items()
            .into_iter()
            .map(|(_, count)| count)
            .sum();
        assert_eq!(throwable_total, 4);
    }
}
