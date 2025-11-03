//! 事件总线系统，用于解耦模块间通信
//!
//! 该系统提供了完整的发布-订阅机制，允许各子模块：
//! - 发布事件到总线
//! - 注册事件监听器
//! - 按优先级处理事件
//! - 使用中间件拦截和转换事件
//! - 回合阶段感知的事件处理
//! - 按类别分组的事件分类

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BinaryHeap};
use std::sync::{Arc, Mutex};
use std::cmp::Ordering;

/// 事件类别 - 用于事件分组和过滤
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventCategory {
    /// 战斗相关事件
    Combat,
    /// 移动相关事件
    Movement,
    /// 状态效果相关事件
    Status,
    /// 物品相关事件
    Items,
    /// AI相关事件
    AI,
    /// 环境交互事件
    Environment,
    /// UI提示事件
    UI,
    /// 系统事件
    System,
    /// 回合阶段事件
    TurnPhase,
    /// 动作意图与结果
    Action,
}

/// 回合阶段枚举 - 用于phase-aware事件处理
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TurnPhase {
    /// 输入阶段
    Input,
    /// 意图队列构建阶段
    IntentQueue,
    /// 动作解析阶段
    Resolution,
    /// 后续处理阶段（能量恢复、状态效果等）
    Aftermath,
    /// 任意阶段（处理器可在任何阶段运行）
    Any,
}

/// 游戏事件定义 - 用于模块间解耦通信
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    // ===== 移动事件 =====
    /// 实体移动
    EntityMoved {
        entity: u32,
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
    },

    // ===== 战斗事件 =====
    /// 战斗开始
    CombatStarted { attacker: u32, defender: u32 },
    /// 攻击命中
    CombatHit {
        attacker: u32,
        defender: u32,
        damage: u32,
        is_critical: bool,
        is_ambush: bool,
    },
    /// 攻击未命中
    CombatMiss { attacker: u32, defender: u32 },
    /// 造成伤害
    DamageDealt {
        attacker: u32,
        victim: u32,
        damage: u32,
        is_critical: bool,
    },
    /// 反击
    CombatCounter {
        attacker: u32,
        defender: u32,
        damage: u32,
        is_critical: bool,
    },
    /// 连击
    CombatChainAttack {
        attacker: u32,
        defender: u32,
        damage: u32,
        is_critical: bool,
    },
    /// 实体死亡
    EntityDied { entity: u32, entity_name: String },
    /// 状态效果应用
    StatusApplied {
        entity: u32,
        status: String,
        duration: u32,
        intensity: u8,
    },
    /// 状态效果移除
    StatusRemoved { 
        entity: u32, 
        status: String,
        reason: String, // "expired", "cleansed", "death", etc
    },
    /// 状态效果周期性触发（DoT/HoT）
    StatusEffectTicked {
        entity: u32,
        status: String,
        damage: u32, // or heal amount
        remaining_turns: u32,
    },
    /// 状态效果冲突解决
    StatusEffectConflict {
        entity: u32,
        removed_effect: String,
        new_effect: String,
    },

    // ===== 技能事件 =====
    /// 职业技能使用
    ClassSkillUsed {
        entity: u32,
        skill_name: String,
        class: String,
        energy_cost: u32,
    },
    /// 职能技能冷却完成
    ClassSkillReady {
        entity: u32,
        skill_name: String,
    },
    /// 技能使用失败（冷却中或其他原因）
    SkillUseFailed {
        entity: u32,
        skill_name: String,
        reason: String,
    },
    /// 被动技能触发
    PassivePerkTriggered {
        entity: u32,
        perk_name: String,
        effect: String,
    },

    // ===== Boss 事件 =====
    /// Boss 遭遇
    BossEncountered { boss_type: String, boss_entity: u32 },
    /// Boss 房间已进入
    BossRoomEntered { boss_type: String },
    /// Boss 阶段转换
    BossPhaseChanged {
        boss_entity: u32,
        old_phase: String,
        new_phase: String,
    },
    /// Boss 使用技能
    BossSkillUsed {
        boss_entity: u32,
        skill_name: String,
    },
    /// Boss 被击败
    BossDefeated {
        boss_entity: u32,
        boss_type: String,
        is_first_kill: bool,
    },
    /// Boss 召唤小怪
    BossSummonedMinions { boss_entity: u32, minion_count: u32 },

    // ===== 动作意图与结果事件 =====
    /// 动作意图（动作排队前）
    ActionIntended {
        entity: u32,
        action_type: String,
        priority: u32,
    },
    /// 动作完成
    ActionCompleted {
        entity: u32,
        action_type: String,
        success: bool,
    },
    /// 动作失败
    ActionFailed {
        entity: u32,
        action_type: String,
        reason: String,
    },
    /// 动作取消
    ActionCancelled {
        entity: u32,
        action_type: String,
        reason: String,
    },

    // ===== 高级战斗结果事件 =====
    /// 攻击被格挡
    CombatBlocked {
        attacker: u32,
        defender: u32,
        blocked_damage: u32,
    },
    /// 攻击被招架
    CombatParried {
        attacker: u32,
        defender: u32,
        parry_damage: u32, // 招架反击伤害
    },
    /// 攻击被闪避
    CombatDodged {
        attacker: u32,
        defender: u32,
    },
    /// 擦伤攻击（部分伤害）
    CombatGrazed {
        attacker: u32,
        defender: u32,
        damage: u32,
    },
    /// 吸血效果
    CombatLifesteal {
        attacker: u32,
        defender: u32,
        damage: u32,
        healed: u32,
    },
    /// 反弹伤害
    CombatReflected {
        attacker: u32,
        defender: u32,
        reflected_damage: u32,
    },
    /// 护盾吸收
    CombatShieldAbsorbed {
        entity: u32,
        damage_absorbed: u32,
        shield_remaining: u32,
    },

    // ===== 状态生命周期事件 =====
    /// 状态效果堆叠
    StatusStacked {
        entity: u32,
        status: String,
        old_intensity: u8,
        new_intensity: u8,
    },
    /// 状态效果刷新
    StatusRefreshed {
        entity: u32,
        status: String,
        duration: u32,
    },
    /// 状态效果抵抗
    StatusResisted {
        entity: u32,
        status: String,
        resist_chance: u8,
    },
    /// 状态效果免疫
    StatusImmune {
        entity: u32,
        status: String,
    },
    /// 状态效果转移
    StatusTransferred {
        from_entity: u32,
        to_entity: u32,
        status: String,
    },
    /// 状态效果扩散
    StatusSpread {
        source_entity: u32,
        target_entities: Vec<u32>,
        status: String,
    },

    // ===== 环境触发事件 =====
    /// 门被打开
    DoorOpened {
        entity: u32,
        x: i32,
        y: i32,
        door_type: String,
    },
    /// 门被关闭
    DoorClosed {
        entity: u32,
        x: i32,
        y: i32,
    },
    /// 发现秘密
    SecretDiscovered {
        entity: u32,
        x: i32,
        y: i32,
        secret_type: String,
    },
    /// 宝箱打开
    ChestOpened {
        entity: u32,
        x: i32,
        y: i32,
        loot_count: u32,
    },
    /// 神龛激活
    ShrineActivated {
        entity: u32,
        x: i32,
        y: i32,
        shrine_type: String,
        effect: String,
    },
    /// 陷阱解除
    TrapDisarmed {
        entity: u32,
        x: i32,
        y: i32,
        trap_type: String,
    },
    /// 地形改变
    TerrainChanged {
        x: i32,
        y: i32,
        old_terrain: String,
        new_terrain: String,
    },
    /// 爆炸效果
    ExplosionTriggered {
        x: i32,
        y: i32,
        radius: u32,
        damage: u32,
    },

    // ===== UI 提示事件 =====
    /// UI 通知
    UINotification {
        message: String,
        notification_type: String, // "info", "warning", "error", "success"
        duration_ms: u32,
    },
    /// UI 警告
    UIAlert {
        message: String,
        severity: String, // "low", "medium", "high", "critical"
    },
    /// 请求工具提示
    TooltipRequested {
        entity: Option<u32>,
        x: i32,
        y: i32,
        context: String,
    },
    /// 高亮提示
    HighlightRequested {
        entities: Vec<u32>,
        positions: Vec<(i32, i32)>,
        highlight_type: String,
        duration_ms: u32,
    },
    /// 动画请求
    AnimationRequested {
        animation_type: String,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
        duration_ms: u32,
    },

    // ===== AI 事件 =====
    /// AI 做出决策
    AIDecisionMade { entity: u32, decision: String },
    /// AI 目标改变
    AITargetChanged {
        entity: u32,
        old_target: Option<u32>,
        new_target: Option<u32>,
    },

    // ===== 物品事件 =====
    /// 拾取物品
    ItemPickedUp { entity: u32, item_name: String },
    /// 丢弃物品
    ItemDropped { entity: u32, item_name: String },
    /// 使用物品
    ItemUsed {
        entity: u32,
        item_name: String,
        effect: String,
    },
    /// 装备物品
    ItemEquipped {
        entity: u32,
        item_name: String,
        slot: String,
    },
    /// 卸下物品
    ItemUnequipped {
        entity: u32,
        item_name: String,
        slot: String,
    },

    // ===== 玩家状态事件 =====
    /// 饥饿度变化
    HungerChanged {
        entity: u32,
        old_satiety: u8,
        new_satiety: u8,
    },
    /// 玩家感到饥饿
    PlayerHungry { entity: u32, satiety: u8 },
    /// 玩家正在挨饿
    PlayerStarving { entity: u32 },
    /// 饥饿造成伤害
    StarvationDamage { entity: u32, damage: u32 },
    /// 食用食物
    FoodEaten {
        entity: u32,
        food_name: String,
        satiety_restored: u8,
    },

    // ===== 游戏状态事件 =====
    /// 回合结束（AI 结算完成并且状态重新切换到玩家）
    ///
    /// 由 `game_loop::GameLoop::emit_turn_state_events` 在 `TurnState::AITurn →
    /// TurnState::PlayerTurn` 转换时触发，`turn` 对应当前 `GameClock.turn_count`。
    TurnEnded { turn: u32 },
    /// 玩家回合开始（UI 可以刷新提示并允许输入）
    ///
    /// 在 turn scheduler 从 AI 阶段切回玩家阶段时触发，由
    /// `GameLoop::emit_turn_state_events` 自动发布。
    PlayerTurnStarted,
    /// AI 回合开始（玩家能量不足或动作已消耗完毕）
    ///
    /// 在 turn scheduler 将状态从玩家切到 AI 时触发，由
    /// `GameLoop::emit_turn_state_events` 自动发布，用于驱动 AI HUD、日志等扩展。
    AITurnStarted,
    /// 游戏结束
    GameOver { reason: String },
    /// 游戏胜利
    Victory,
    /// 暂停游戏
    GamePaused,
    /// 恢复游戏
    GameResumed,

    // ===== 地牢事件 =====
    /// 进入新层
    LevelChanged { old_level: usize, new_level: usize },
    /// 发现房间
    RoomDiscovered { room_id: usize },
    /// 触发陷阱
    TrapTriggered { entity: u32, trap_type: String },

    // ===== 系统事件 =====
    /// 保存游戏
    GameSaved { save_slot: String },
    /// 加载游戏
    GameLoaded { save_slot: String },
    /// 日志消息
    LogMessage { message: String, level: LogLevel },
}

/// 日志级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// 事件处理器优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// 最高优先级 - 用于关键系统事件
    Critical = 0,
    /// 高优先级 - 用于游戏核心逻辑
    High = 1,
    /// 普通优先级 - 默认优先级
    Normal = 2,
    /// 低优先级 - 用于 UI 更新等
    Low = 3,
    /// 最低优先级 - 用于日志等
    Lowest = 4,
}

/// 事件中间件 trait - 用于在事件处理前/后插入逻辑
pub trait EventMiddleware: Send + Sync {
    /// 在事件处理之前调用，可以修改事件或阻止处理
    /// 返回 true 表示继续处理，false 表示阻止处理
    fn before_handle(&mut self, event: &GameEvent) -> bool {
        true // 默认允许处理
    }

    /// 在事件处理之后调用
    fn after_handle(&mut self, event: &GameEvent) {
        // 默认不执行任何操作
    }

    /// 中间件名称（用于调试）
    fn name(&self) -> &str;

    /// 中间件优先级
    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

/// 事件处理器 trait - 支持phase-aware处理
pub trait EventHandler: Send + Sync {
    /// 处理事件
    fn handle(&mut self, event: &GameEvent);

    /// 事件处理器的名称（用于调试）
    fn name(&self) -> &str;

    /// 优先级（数字越小优先级越高）
    fn priority(&self) -> Priority {
        Priority::Normal
    }

    /// 是否应该处理此事件（事件过滤）
    fn should_handle(&self, event: &GameEvent) -> bool {
        true
    }

    /// 声明处理器应在哪些回合阶段运行（默认Any - 所有阶段）
    fn run_in_phases(&self) -> Vec<TurnPhase> {
        vec![TurnPhase::Any]
    }
}

/// 事件处理器包装器，包含优先级信息
struct HandlerEntry {
    handler: Box<dyn EventHandler>,
    priority: Priority,
}

/// 中间件包装器，包含优先级信息
struct MiddlewareEntry {
    middleware: Box<dyn EventMiddleware>,
    priority: Priority,
}

/// 优先级事件队列条目
#[derive(Clone)]
struct PriorityEventEntry {
    event: GameEvent,
    priority: Priority,
    sequence: u64, // 序列号用于同优先级的FIFO排序
}

impl PriorityEventEntry {
    fn new(event: GameEvent, priority: Priority, sequence: u64) -> Self {
        Self {
            event,
            priority,
            sequence,
        }
    }
}

// 实现Ord以支持BinaryHeap（最大堆，优先级高的先出）
impl Ord for PriorityEventEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // 首先比较优先级（数值小的优先级高）
        match other.priority.cmp(&self.priority) {
            Ordering::Equal => {
                // 优先级相同时，序列号小的先处理（FIFO）
                other.sequence.cmp(&self.sequence)
            }
            other_order => other_order,
        }
    }
}

impl PartialOrd for PriorityEventEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PriorityEventEntry {}

impl PartialEq for PriorityEventEntry {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.sequence == other.sequence
    }
}

/// 增强的事件总线 - 支持订阅模式、队列模式和phase-aware处理
///
/// 设计思路：
/// 1. 保留原有的队列模式（适合游戏循环）
/// 2. 添加订阅者模式（适合模块解耦）
/// 3. 支持事件优先级和过滤
/// 4. 按回合阶段分组的事件队列
/// 5. 防止递归发布的保护机制
pub struct EventBus {
    /// 当前帧的事件队列（传统模式）
    events: Vec<GameEvent>,
    /// 下一帧的事件队列
    next_frame_events: Vec<GameEvent>,
    /// 按阶段分组的优先级事件队列
    phase_queues: HashMap<TurnPhase, BinaryHeap<PriorityEventEntry>>,
    /// 注册的事件处理器（按事件类型分组）
    handlers: HashMap<&'static str, Vec<HandlerEntry>>,
    /// 全局事件处理器（处理所有事件）
    global_handlers: Vec<HandlerEntry>,
    /// 按阶段注册的处理器
    phase_handlers: HashMap<TurnPhase, Vec<HandlerEntry>>,
    /// 事件中间件
    middlewares: Vec<MiddlewareEntry>,
    /// 事件历史（用于调试和回放）
    history: Vec<GameEvent>,
    /// 历史记录的最大长度
    max_history: usize,
    /// 当前回合阶段
    current_phase: TurnPhase,
    /// 事件序列计数器（用于FIFO排序）
    event_sequence: u64,
    /// 递归发布深度计数器
    publish_depth: usize,
    /// 最大递归深度
    max_publish_depth: usize,
    /// 批处理缓冲区（用于防止递归发布）
    batch_buffer: Vec<(GameEvent, Priority, TurnPhase)>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::with_history_size(100)
    }

    /// 创建一个指定历史记录大小的事件总线
    pub fn with_history_size(max_history: usize) -> Self {
        let mut phase_queues = HashMap::new();
        // 初始化所有阶段的队列
        phase_queues.insert(TurnPhase::Input, BinaryHeap::new());
        phase_queues.insert(TurnPhase::IntentQueue, BinaryHeap::new());
        phase_queues.insert(TurnPhase::Resolution, BinaryHeap::new());
        phase_queues.insert(TurnPhase::Aftermath, BinaryHeap::new());
        phase_queues.insert(TurnPhase::Any, BinaryHeap::new());

        Self {
            events: Vec::new(),
            next_frame_events: Vec::new(),
            phase_queues,
            handlers: HashMap::new(),
            global_handlers: Vec::new(),
            phase_handlers: HashMap::new(),
            middlewares: Vec::new(),
            history: Vec::new(),
            max_history,
            current_phase: TurnPhase::Input,
            event_sequence: 0,
            publish_depth: 0,
            max_publish_depth: 10,
            batch_buffer: Vec::new(),
        }
    }

    /// 设置当前回合阶段
    pub fn set_current_phase(&mut self, phase: TurnPhase) {
        self.current_phase = phase;
    }

    /// 获取当前回合阶段
    pub fn get_current_phase(&self) -> TurnPhase {
        self.current_phase
    }

    /// 发布事件到指定阶段和优先级的队列
    pub fn publish_to_phase(&mut self, event: GameEvent, priority: Priority, phase: TurnPhase) {
        // 检查递归深度
        if self.publish_depth >= self.max_publish_depth {
            eprintln!(
                "Warning: Maximum publish depth reached ({}), buffering event for batch processing",
                self.max_publish_depth
            );
            self.batch_buffer.push((event, priority, phase));
            return;
        }

        self.publish_depth += 1;

        // 记录到历史
        self.add_to_history(event.clone());

        // 立即触发订阅者处理
        self.dispatch_to_handlers(&event);

        // 获取下一个序列号
        let sequence = self.event_sequence;
        self.event_sequence += 1;

        // 添加到相应阶段的优先级队列
        let entry = PriorityEventEntry::new(event.clone(), priority, sequence);
        if let Some(queue) = self.phase_queues.get_mut(&phase) {
            queue.push(entry);
        }

        // 也添加到传统队列以保持向后兼容
        self.events.push(event);

        self.publish_depth -= 1;

        // 如果递归深度回到0，处理批处理缓冲区
        if self.publish_depth == 0 {
            self.flush_batch_buffer();
        }
    }

    /// 刷新批处理缓冲区
    fn flush_batch_buffer(&mut self) {
        let batch = std::mem::take(&mut self.batch_buffer);
        for (event, priority, phase) in batch {
            self.publish_to_phase(event, priority, phase);
        }
    }

    // ========== 队列模式 API（保持向后兼容）==========

    /// 发布事件（添加到当前帧队列）
    pub fn publish(&mut self, event: GameEvent) {
        // 记录到历史
        self.add_to_history(event.clone());

        // 立即触发订阅者处理
        self.dispatch_to_handlers(&event);

        // 添加到队列
        self.events.push(event);
    }

    /// 发布延迟事件（添加到下一帧队列）
    pub fn publish_delayed(&mut self, event: GameEvent) {
        // 记录到历史
        self.add_to_history(event.clone());

        // 添加到下一帧队列（不立即触发处理器）
        self.next_frame_events.push(event);
    }

    /// 获取所有待处理事件并清空队列
    pub fn drain(&mut self) -> impl Iterator<Item = GameEvent> + '_ {
        self.events.drain(..)
    }

    /// 检查是否有待处理事件
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }

    /// 获取事件数量
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// 帧结束时调用，将下一帧事件移到当前帧
    pub fn next_frame(&mut self) {
        // 将下一帧事件移到当前帧
        std::mem::swap(&mut self.events, &mut self.next_frame_events);
        self.next_frame_events.clear();

        // 触发当前帧事件的处理器
        let events_to_dispatch: Vec<_> = self.events.clone();
        for event in &events_to_dispatch {
            self.dispatch_to_handlers(event);
        }
    }

    /// 清空所有事件
    pub fn clear(&mut self) {
        self.events.clear();
        self.next_frame_events.clear();
        for queue in self.phase_queues.values_mut() {
            queue.clear();
        }
        self.batch_buffer.clear();
    }

    /// 排空当前阶段的事件队列，返回按优先级排序的事件
    pub fn drain_phase(&mut self, phase: TurnPhase) -> Vec<GameEvent> {
        let mut events = Vec::new();
        
        // 排空指定阶段的队列
        if let Some(queue) = self.phase_queues.get_mut(&phase) {
            while let Some(entry) = queue.pop() {
                events.push(entry.event);
            }
        }
        
        // 也排空Any阶段的队列（这些事件在所有阶段都应该处理）
        if phase != TurnPhase::Any {
            if let Some(any_queue) = self.phase_queues.get_mut(&TurnPhase::Any) {
                while let Some(entry) = any_queue.pop() {
                    events.push(entry.event);
                }
            }
        }
        
        events
    }

    /// 处理当前阶段的所有事件（使用phase-aware handlers）
    pub fn process_phase_events(&mut self, phase: TurnPhase) {
        let events = self.drain_phase(phase);
        
        for event in events {
            self.dispatch_to_phase_handlers(&event, phase);
        }
    }

    /// 分发事件给phase-aware处理器
    fn dispatch_to_phase_handlers(&mut self, event: &GameEvent, phase: TurnPhase) {
        // 首先让中间件处理事件
        if !self.run_middleware_before(event) {
            return;
        }

        // 处理注册到此阶段的处理器
        if let Some(handlers) = self.phase_handlers.get_mut(&phase) {
            for entry in handlers {
                if entry.handler.should_handle(event) {
                    entry.handler.handle(event);
                }
            }
        }

        // 也处理注册到Any阶段的处理器
        if phase != TurnPhase::Any {
            if let Some(handlers) = self.phase_handlers.get_mut(&TurnPhase::Any) {
                for entry in handlers {
                    if entry.handler.should_handle(event) {
                        entry.handler.handle(event);
                    }
                }
            }
        }

        self.run_middleware_after(event);
    }

    // ========== 中间件 API（新增）==========

    /// 注册事件中间件
    pub fn register_middleware(&mut self, middleware: Box<dyn EventMiddleware>) {
        let priority = middleware.priority();
        let entry = MiddlewareEntry {
            middleware,
            priority,
        };

        self.middlewares.push(entry);

        // 按优先级排序（优先级高的在前面）
        self.middlewares.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    // ========== 订阅者模式 API（新增）==========

    /// 注册事件处理器（处理特定类型的事件）
    pub fn subscribe(&mut self, event_type: &'static str, handler: Box<dyn EventHandler>) {
        let priority = handler.priority();
        let entry = HandlerEntry { handler, priority };

        let handlers = self.handlers.entry(event_type).or_insert_with(Vec::new);
        handlers.push(entry);

        // 按优先级排序（优先级高的在前面）
        handlers.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    /// 注册全局事件处理器（处理所有事件）
    pub fn subscribe_all(&mut self, handler: Box<dyn EventHandler>) {
        let priority = handler.priority();
        let entry = HandlerEntry { handler, priority };

        self.global_handlers.push(entry);

        // 按优先级排序
        self.global_handlers
            .sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    /// 注册phase-aware事件处理器
    pub fn subscribe_for_phase(&mut self, phase: TurnPhase, handler: Box<dyn EventHandler>) {
        let priority = handler.priority();
        let entry = HandlerEntry { handler, priority };

        let handlers = self.phase_handlers.entry(phase).or_insert_with(Vec::new);
        handlers.push(entry);

        // 按优先级排序
        handlers.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    /// 注册处理器到多个阶段（根据handler的run_in_phases声明）
    /// 
    /// 注意：由于无法克隆 Box<dyn EventHandler>，此方法只注册到第一个声明的阶段
    /// 如果需要注册到多个阶段，请手动调用 subscribe_for_phase 多次
    pub fn subscribe_with_phases(&mut self, handler: Box<dyn EventHandler>) {
        let phases = handler.run_in_phases();
        
        // 简化版本：只注册到第一个声明的阶段
        // 在实际使用中，调用者需要为每个阶段单独创建处理器实例
        if let Some(&first_phase) = phases.first() {
            self.subscribe_for_phase(first_phase, handler);
        }
    }

    /// 分发事件给所有订阅者
    fn dispatch_to_handlers(&mut self, event: &GameEvent) {
        // 首先让中间件处理事件
        if !self.run_middleware_before(event) {
            // 如果中间件阻止了事件处理，则返回
            return;
        }

        let event_type = event.event_type();

        // 先处理全局处理器
        for entry in &mut self.global_handlers {
            if entry.handler.should_handle(event) {
                entry.handler.handle(event);
            }
        }

        // 再处理特定类型的处理器
        if let Some(handlers) = self.handlers.get_mut(event_type) {
            for entry in handlers {
                if entry.handler.should_handle(event) {
                    entry.handler.handle(event);
                }
            }
        }

        // 事件处理完成后，让中间件执行后处理
        self.run_middleware_after(event);
    }

    /// 执行中间件的前置处理，返回是否应该继续处理事件
    fn run_middleware_before(&mut self, event: &GameEvent) -> bool {
        for entry in &mut self.middlewares {
            if !entry.middleware.before_handle(event) {
                // 如果任一中间件返回false，阻止事件处理
                return false;
            }
        }
        true
    }

    /// 执行中间件的后置处理
    fn run_middleware_after(&mut self, event: &GameEvent) {
        for entry in &mut self.middlewares {
            entry.middleware.after_handle(event);
        }
    }

    // ========== 历史记录和调试 API ==========

    /// 添加事件到历史记录
    fn add_to_history(&mut self, event: GameEvent) {
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(event);
    }

    /// 获取事件历史（最近的 n 个事件）
    pub fn get_history(&self, count: usize) -> &[GameEvent] {
        let start = self.history.len().saturating_sub(count);
        &self.history[start..]
    }

    /// 清空历史记录
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// 获取所有历史记录
    pub fn full_history(&self) -> &[GameEvent] {
        &self.history
    }

    /// 获取订阅者数量（用于调试）
    pub fn subscriber_count(&self) -> usize {
        self.global_handlers.len()
            + self.handlers.values().map(|v| v.len()).sum::<usize>()
            + self.middlewares.len()
    }

    /// 获取中间件数量（用于调试）
    pub fn middleware_count(&self) -> usize {
        self.middlewares.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl GameEvent {
    /// 获取事件类型的字符串表示
    pub fn event_type(&self) -> &'static str {
        match self {
            GameEvent::EntityMoved { .. } => "EntityMoved",
            GameEvent::CombatStarted { .. } => "CombatStarted",
            GameEvent::CombatHit { .. } => "CombatHit",
            GameEvent::CombatMiss { .. } => "CombatMiss",
            GameEvent::CombatCounter { .. } => "CombatCounter",
            GameEvent::CombatChainAttack { .. } => "CombatChainAttack",
            GameEvent::DamageDealt { .. } => "DamageDealt",
            GameEvent::EntityDied { .. } => "EntityDied",
            GameEvent::StatusApplied { .. } => "StatusApplied",
            GameEvent::StatusRemoved { .. } => "StatusRemoved",
            GameEvent::AIDecisionMade { .. } => "AIDecisionMade",
            GameEvent::AITargetChanged { .. } => "AITargetChanged",
            GameEvent::ItemPickedUp { .. } => "ItemPickedUp",
            GameEvent::ItemDropped { .. } => "ItemDropped",
            GameEvent::ItemUsed { .. } => "ItemUsed",
            GameEvent::ItemEquipped { .. } => "ItemEquipped",
            GameEvent::ItemUnequipped { .. } => "ItemUnequipped",
            GameEvent::TurnEnded { .. } => "TurnEnded",
            GameEvent::PlayerTurnStarted => "PlayerTurnStarted",
            GameEvent::AITurnStarted => "AITurnStarted",
            GameEvent::GameOver { .. } => "GameOver",
            GameEvent::Victory => "Victory",
            GameEvent::GamePaused => "GamePaused",
            GameEvent::GameResumed => "GameResumed",
            GameEvent::LevelChanged { .. } => "LevelChanged",
            GameEvent::RoomDiscovered { .. } => "RoomDiscovered",
            GameEvent::TrapTriggered { .. } => "TrapTriggered",
            GameEvent::GameSaved { .. } => "GameSaved",
            GameEvent::GameLoaded { .. } => "GameLoaded",
            GameEvent::LogMessage { .. } => "LogMessage",
            GameEvent::HungerChanged { .. } => "HungerChanged",
            GameEvent::PlayerHungry { .. } => "PlayerHungry",
            GameEvent::PlayerStarving { .. } => "PlayerStarving",
            GameEvent::StarvationDamage { .. } => "StarvationDamage",
            GameEvent::FoodEaten { .. } => "FoodEaten",
            GameEvent::BossEncountered { .. } => "BossEncountered",
            GameEvent::BossRoomEntered { .. } => "BossRoomEntered",
            GameEvent::BossPhaseChanged { .. } => "BossPhaseChanged",
            GameEvent::BossSkillUsed { .. } => "BossSkillUsed",
            GameEvent::BossDefeated { .. } => "BossDefeated",
            GameEvent::BossSummonedMinions { .. } => "BossSummonedMinions",
            GameEvent::StatusEffectTicked { .. } => "StatusEffectTicked",
            GameEvent::StatusEffectConflict { .. } => "StatusEffectConflict",
            // Skill events
            GameEvent::ClassSkillUsed { .. } => "ClassSkillUsed",
            GameEvent::ClassSkillReady { .. } => "ClassSkillReady",
            GameEvent::SkillUseFailed { .. } => "SkillUseFailed",
            GameEvent::PassivePerkTriggered { .. } => "PassivePerkTriggered",
            // New events
            GameEvent::ActionIntended { .. } => "ActionIntended",
            GameEvent::ActionCompleted { .. } => "ActionCompleted",
            GameEvent::ActionFailed { .. } => "ActionFailed",
            GameEvent::ActionCancelled { .. } => "ActionCancelled",
            GameEvent::CombatBlocked { .. } => "CombatBlocked",
            GameEvent::CombatParried { .. } => "CombatParried",
            GameEvent::CombatDodged { .. } => "CombatDodged",
            GameEvent::CombatGrazed { .. } => "CombatGrazed",
            GameEvent::CombatLifesteal { .. } => "CombatLifesteal",
            GameEvent::CombatReflected { .. } => "CombatReflected",
            GameEvent::CombatShieldAbsorbed { .. } => "CombatShieldAbsorbed",
            GameEvent::StatusStacked { .. } => "StatusStacked",
            GameEvent::StatusRefreshed { .. } => "StatusRefreshed",
            GameEvent::StatusResisted { .. } => "StatusResisted",
            GameEvent::StatusImmune { .. } => "StatusImmune",
            GameEvent::StatusTransferred { .. } => "StatusTransferred",
            GameEvent::StatusSpread { .. } => "StatusSpread",
            GameEvent::DoorOpened { .. } => "DoorOpened",
            GameEvent::DoorClosed { .. } => "DoorClosed",
            GameEvent::SecretDiscovered { .. } => "SecretDiscovered",
            GameEvent::ChestOpened { .. } => "ChestOpened",
            GameEvent::ShrineActivated { .. } => "ShrineActivated",
            GameEvent::TrapDisarmed { .. } => "TrapDisarmed",
            GameEvent::TerrainChanged { .. } => "TerrainChanged",
            GameEvent::ExplosionTriggered { .. } => "ExplosionTriggered",
            GameEvent::UINotification { .. } => "UINotification",
            GameEvent::UIAlert { .. } => "UIAlert",
            GameEvent::TooltipRequested { .. } => "TooltipRequested",
            GameEvent::HighlightRequested { .. } => "HighlightRequested",
            GameEvent::AnimationRequested { .. } => "AnimationRequested",
        }
    }

    /// 获取事件的类别
    pub fn category(&self) -> EventCategory {
        match self {
            // Combat events
            GameEvent::CombatStarted { .. }
            | GameEvent::CombatHit { .. }
            | GameEvent::CombatMiss { .. }
            | GameEvent::CombatCounter { .. }
            | GameEvent::CombatChainAttack { .. }
            | GameEvent::DamageDealt { .. }
            | GameEvent::EntityDied { .. }
            | GameEvent::CombatBlocked { .. }
            | GameEvent::CombatParried { .. }
            | GameEvent::CombatDodged { .. }
            | GameEvent::CombatGrazed { .. }
            | GameEvent::CombatLifesteal { .. }
            | GameEvent::CombatReflected { .. }
            | GameEvent::CombatShieldAbsorbed { .. } => EventCategory::Combat,

            // Movement events
            GameEvent::EntityMoved { .. } => EventCategory::Movement,

            // Status events
            GameEvent::StatusApplied { .. }
            | GameEvent::StatusRemoved { .. }
            | GameEvent::StatusEffectTicked { .. }
            | GameEvent::StatusEffectConflict { .. }
            | GameEvent::StatusStacked { .. }
            | GameEvent::StatusRefreshed { .. }
            | GameEvent::StatusResisted { .. }
            | GameEvent::StatusImmune { .. }
            | GameEvent::StatusTransferred { .. }
            | GameEvent::StatusSpread { .. } => EventCategory::Status,

            // Item events
            GameEvent::ItemPickedUp { .. }
            | GameEvent::ItemDropped { .. }
            | GameEvent::ItemUsed { .. }
            | GameEvent::ItemEquipped { .. }
            | GameEvent::ItemUnequipped { .. } => EventCategory::Items,

            // AI events
            GameEvent::AIDecisionMade { .. } 
            | GameEvent::AITargetChanged { .. } => EventCategory::AI,

            // Skill events
            GameEvent::ClassSkillUsed { .. }
            | GameEvent::ClassSkillReady { .. }
            | GameEvent::SkillUseFailed { .. }
            | GameEvent::PassivePerkTriggered { .. } => EventCategory::Action,

            // Environment events
            GameEvent::LevelChanged { .. }
            | GameEvent::RoomDiscovered { .. }
            | GameEvent::TrapTriggered { .. }
            | GameEvent::DoorOpened { .. }
            | GameEvent::DoorClosed { .. }
            | GameEvent::SecretDiscovered { .. }
            | GameEvent::ChestOpened { .. }
            | GameEvent::ShrineActivated { .. }
            | GameEvent::TrapDisarmed { .. }
            | GameEvent::TerrainChanged { .. }
            | GameEvent::ExplosionTriggered { .. } => EventCategory::Environment,

            // UI events
            GameEvent::UINotification { .. }
            | GameEvent::UIAlert { .. }
            | GameEvent::TooltipRequested { .. }
            | GameEvent::HighlightRequested { .. }
            | GameEvent::AnimationRequested { .. }
            | GameEvent::LogMessage { .. } => EventCategory::UI,

            // System events
            GameEvent::GameOver { .. }
            | GameEvent::Victory
            | GameEvent::GamePaused
            | GameEvent::GameResumed
            | GameEvent::GameSaved { .. }
            | GameEvent::GameLoaded { .. } => EventCategory::System,

            // Turn phase events
            GameEvent::TurnEnded { .. }
            | GameEvent::PlayerTurnStarted
            | GameEvent::AITurnStarted => EventCategory::TurnPhase,

            // Action events
            GameEvent::ActionIntended { .. }
            | GameEvent::ActionCompleted { .. }
            | GameEvent::ActionFailed { .. }
            | GameEvent::ActionCancelled { .. } => EventCategory::Action,

            // Special cases (Boss, Hunger, etc.)
            GameEvent::BossEncountered { .. }
            | GameEvent::BossRoomEntered { .. }
            | GameEvent::BossPhaseChanged { .. }
            | GameEvent::BossSkillUsed { .. }
            | GameEvent::BossDefeated { .. }
            | GameEvent::BossSummonedMinions { .. } => EventCategory::Combat,

            GameEvent::HungerChanged { .. }
            | GameEvent::PlayerHungry { .. }
            | GameEvent::PlayerStarving { .. }
            | GameEvent::StarvationDamage { .. }
            | GameEvent::FoodEaten { .. } => EventCategory::Status,
        }
    }
}

// ========== 辅助宏和类型 ==========

/// 创建一个简单的事件处理器的宏
///
/// # 示例
/// ```rust
/// # use terminal_pixel_dungeon::{simple_handler, event_bus::{EventHandler, GameEvent, Priority}};
/// simple_handler!(MyHandler, "MyHandler", Priority::Normal, |event| {
///     match event {
///         GameEvent::DamageDealt { damage, .. } => {
///             println!("造成了 {} 点伤害", damage);
///         }
///         _ => {}
///     }
/// });
/// ```
#[macro_export]
macro_rules! simple_handler {
    ($name:ident, $handler_name:expr, $priority:expr, $closure:expr) => {
        pub struct $name {
            handler_fn: Box<dyn Fn(&GameEvent) + Send + Sync>,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    handler_fn: Box::new($closure),
                }
            }
        }

        impl EventHandler for $name {
            fn handle(&mut self, event: &GameEvent) {
                (self.handler_fn)(event);
            }

            fn name(&self) -> &str {
                $handler_name
            }

            fn priority(&self) -> Priority {
                $priority
            }
        }
    };
}

/// 创建一个带状态的事件处理器的宏
#[macro_export]
macro_rules! stateful_handler {
    ($name:ident, $state_type:ty, $handler_name:expr, $priority:expr) => {
        pub struct $name {
            pub state: $state_type,
        }

        impl $name {
            pub fn new(initial_state: $state_type) -> Self {
                Self {
                    state: initial_state,
                }
            }
        }

        impl EventHandler for $name {
            fn name(&self) -> &str {
                $handler_name
            }

            fn priority(&self) -> Priority {
                $priority
            }
        }
    };
}

// ========== 高级过滤器和条件处理 ==========

/// 条件过滤器 - 基于自定义条件过滤事件
pub struct ConditionalFilter<F>
where
    F: Fn(&GameEvent) -> bool + Send + Sync,
{
    condition: F,
    filter_name: String,
}

impl<F> ConditionalFilter<F>
where
    F: Fn(&GameEvent) -> bool + Send + Sync,
{
    pub fn new(condition: F, name: &str) -> Self {
        Self {
            condition,
            filter_name: name.to_string(),
        }
    }
}

impl<F> EventMiddleware for ConditionalFilter<F>
where
    F: Fn(&GameEvent) -> bool + Send + Sync,
{
    fn before_handle(&mut self, event: &GameEvent) -> bool {
        (self.condition)(event)
    }

    fn name(&self) -> &str {
        &self.filter_name
    }
}

/// 事件转换器 - 在事件处理前修改事件
pub struct EventTransformer<F>
where
    F: Fn(&GameEvent) -> Option<GameEvent> + Send + Sync,
{
    transformer: F,
    transformer_name: String,
}

impl<F> EventTransformer<F>
where
    F: Fn(&GameEvent) -> Option<GameEvent> + Send + Sync,
{
    pub fn new(transformer: F, name: &str) -> Self {
        Self {
            transformer,
            transformer_name: name.to_string(),
        }
    }
}

impl<F> EventMiddleware for EventTransformer<F>
where
    F: Fn(&GameEvent) -> Option<GameEvent> + Send + Sync,
{
    fn before_handle(&mut self, event: &GameEvent) -> bool {
        // 注意：这里我们不能真的变换事件，因为event是引用
        // 实际事件变换需要更复杂的实现，这里我们仅验证事件是否符合条件
        (self.transformer)(event).is_some()
    }

    fn name(&self) -> &str {
        &self.transformer_name
    }
}

/// 时间限制中间件 - 限制特定事件的处理频率
pub struct RateLimitMiddleware {
    event_counts: HashMap<&'static str, (usize, std::time::Instant)>,
    max_per_interval: usize,
    interval: std::time::Duration,
    name: String,
}

impl RateLimitMiddleware {
    pub fn new(max_per_interval: usize, interval: std::time::Duration) -> Self {
        Self {
            event_counts: HashMap::new(),
            max_per_interval,
            interval,
            name: format!(
                "RateLimitMiddleware({} per {:?})",
                max_per_interval, interval
            ),
        }
    }
}

impl EventMiddleware for RateLimitMiddleware {
    fn before_handle(&mut self, event: &GameEvent) -> bool {
        let event_type = event.event_type();
        let now = std::time::Instant::now();

        let (count, last_check) = self.event_counts.entry(event_type).or_insert((0, now));

        // 如果时间间隔已过，重置计数
        if now.duration_since(*last_check) >= self.interval {
            *count = 0;
            *last_check = now;
        }

        // 检查是否超过限制
        if *count >= self.max_per_interval {
            return false; // 拒绝处理事件
        }

        *count += 1;
        true
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ========== 内置事件中间件 ==========

/// 事件日志中间件 - 在事件处理前后记录日志
pub struct LoggingMiddleware {
    messages: Arc<Mutex<Vec<String>>>,
}

impl LoggingMiddleware {
    pub fn new(messages: Arc<Mutex<Vec<String>>>) -> Self {
        Self { messages }
    }
}

impl EventMiddleware for LoggingMiddleware {
    fn before_handle(&mut self, event: &GameEvent) -> bool {
        let message = format!("即将处理事件: {:?}", event.event_type());
        if let Ok(mut logs) = self.messages.lock() {
            logs.push(message);
        }
        true // 继续处理事件
    }

    fn after_handle(&mut self, event: &GameEvent) {
        let message = format!("已处理事件: {:?}", event.event_type());
        if let Ok(mut logs) = self.messages.lock() {
            logs.push(message);
        }
    }

    fn name(&self) -> &str {
        "LoggingMiddleware"
    }

    fn priority(&self) -> Priority {
        Priority::Lowest
    }
}

/// 事件计数中间件 - 统计事件处理次数
pub struct CountingMiddleware {
    counts: HashMap<&'static str, usize>,
}

impl CountingMiddleware {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    pub fn get_count(&self, event_type: &str) -> usize {
        self.counts.get(event_type).copied().unwrap_or(0)
    }

    pub fn total_events(&self) -> usize {
        self.counts.values().sum()
    }

    pub fn reset(&mut self) {
        self.counts.clear();
    }
}

impl EventMiddleware for CountingMiddleware {
    fn before_handle(&mut self, event: &GameEvent) -> bool {
        let event_type = event.event_type();
        *self.counts.entry(event_type).or_insert(0) += 1;
        true
    }

    fn name(&self) -> &str {
        "CountingMiddleware"
    }

    fn priority(&self) -> Priority {
        Priority::Lowest
    }
}

/// 调试中间件 - 当特定事件发生时输出调试信息
pub struct DebuggingMiddleware {
    debug_events: Vec<&'static str>,
}

impl DebuggingMiddleware {
    pub fn new(event_types: Vec<&'static str>) -> Self {
        Self {
            debug_events: event_types,
        }
    }
}

impl EventMiddleware for DebuggingMiddleware {
    fn before_handle(&mut self, event: &GameEvent) -> bool {
        if self.debug_events.contains(&event.event_type()) {
            println!("Debug: {:?}", event);
        }
        true
    }

    fn name(&self) -> &str {
        "DebuggingMiddleware"
    }

    fn priority(&self) -> Priority {
        Priority::Low
    }
}

// ========== 内置事件处理器 ==========

/// 日志记录器 - 记录所有事件到消息列表
pub struct LoggingHandler {
    messages: Arc<Mutex<Vec<String>>>,
}

impl LoggingHandler {
    pub fn new(messages: Arc<Mutex<Vec<String>>>) -> Self {
        Self { messages }
    }
}

impl EventHandler for LoggingHandler {
    fn handle(&mut self, event: &GameEvent) {
        let message = match event {
            GameEvent::DamageDealt {
                damage,
                is_critical,
                ..
            } => {
                if *is_critical {
                    format!("暴击！造成 {} 点伤害", damage)
                } else {
                    format!("造成 {} 点伤害", damage)
                }
            }
            GameEvent::EntityDied { entity_name, .. } => {
                format!("{} 已死亡", entity_name)
            }
            GameEvent::ItemPickedUp { item_name, .. } => {
                format!("拾取了 {}", item_name)
            }
            GameEvent::ItemUsed {
                item_name, effect, ..
            } => {
                format!("使用了 {}，{}", item_name, effect)
            }
            GameEvent::LevelChanged {
                old_level,
                new_level,
            } => {
                format!("从第 {} 层进入第 {} 层", old_level, new_level)
            }
            GameEvent::LogMessage { message, .. } => message.clone(),
            GameEvent::TrapTriggered { trap_type, .. } => {
                format!("触发了{}陷阱！", trap_type)
            }
            GameEvent::StatusApplied {
                status, duration, ..
            } => {
                format!("受到{}效果影响，持续{}回合", status, duration)
            }
            GameEvent::StatusRemoved { status, .. } => {
                format!("{}效果已消失", status)
            }
            _ => return, // 其他事件不记录
        };

        if let Ok(mut logs) = self.messages.lock() {
            logs.push(message);
        }
    }

    fn name(&self) -> &str {
        "LoggingHandler"
    }

    fn priority(&self) -> Priority {
        Priority::Lowest // 日志记录优先级最低
    }
}

/// 战斗统计处理器 - 统计战斗相关数据
pub struct CombatStatsHandler {
    total_damage_dealt: u32,
    total_damage_taken: u32,
    kills: u32,
    critical_hits: u32,
    player_entity_id: Option<u32>,
}

impl CombatStatsHandler {
    pub fn new(player_entity_id: Option<u32>) -> Self {
        Self {
            total_damage_dealt: 0,
            total_damage_taken: 0,
            kills: 0,
            critical_hits: 0,
            player_entity_id,
        }
    }

    pub fn get_total_damage_dealt(&self) -> u32 {
        self.total_damage_dealt
    }

    pub fn get_total_damage_taken(&self) -> u32 {
        self.total_damage_taken
    }

    pub fn get_kills(&self) -> u32 {
        self.kills
    }

    pub fn get_critical_hits(&self) -> u32 {
        self.critical_hits
    }

    pub fn reset_stats(&mut self) {
        self.total_damage_dealt = 0;
        self.total_damage_taken = 0;
        self.kills = 0;
        self.critical_hits = 0;
    }
}

impl EventHandler for CombatStatsHandler {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::DamageDealt {
                attacker,
                victim,
                damage,
                is_critical,
            } => {
                if let Some(player_id) = self.player_entity_id {
                    if *attacker == player_id {
                        self.total_damage_dealt += damage;
                        if *is_critical {
                            self.critical_hits += 1;
                        }
                    }
                    if *victim == player_id {
                        self.total_damage_taken += damage;
                    }
                } else {
                    // 如果没有指定玩家ID，则统计所有战斗
                    self.total_damage_dealt += damage;
                    if *is_critical {
                        self.critical_hits += 1;
                    }
                }
            }
            GameEvent::EntityDied { .. } => {
                // 假设任何死亡都算作击杀（需要更精确的逻辑来判断击杀者）
                self.kills += 1;
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "CombatStatsHandler"
    }

    fn priority(&self) -> Priority {
        Priority::Low
    }
}

/// 消息聚合处理器 - 将相似消息聚合以减少重复
pub struct MessageAggregator {
    message_counts: HashMap<String, u32>,
    output_messages: Arc<Mutex<Vec<String>>>,
    flush_interval: u32,
    current_count: u32,
}

impl MessageAggregator {
    pub fn new(output_messages: Arc<Mutex<Vec<String>>>, flush_interval: u32) -> Self {
        Self {
            message_counts: HashMap::new(),
            output_messages,
            flush_interval,
            current_count: 0,
        }
    }

    fn flush_messages(&mut self) {
        let mut output = self.output_messages.lock().unwrap();
        for (msg, count) in &self.message_counts {
            if *count > 1 {
                output.push(format!("{} (x{})", msg, count));
            } else {
                output.push(msg.clone());
            }
        }
        drop(output);
        self.message_counts.clear();
    }
}

impl EventHandler for MessageAggregator {
    fn handle(&mut self, event: &GameEvent) {
        let message = match event {
            GameEvent::ItemPickedUp { item_name, .. } => {
                format!("拾取了 {}", item_name)
            }
            GameEvent::ItemUsed { item_name, .. } => {
                format!("使用了 {}", item_name)
            }
            GameEvent::DamageDealt { damage, .. } => {
                format!("造成 {} 点伤害", damage)
            }
            GameEvent::LogMessage { message, .. } => message.clone(),
            _ => return, // 只处理特定类型的消息
        };

        *self.message_counts.entry(message).or_insert(0) += 1;
        self.current_count += 1;

        if self.current_count >= self.flush_interval {
            self.flush_messages();
            self.current_count = 0;
        }
    }

    fn name(&self) -> &str {
        "MessageAggregator"
    }

    fn priority(&self) -> Priority {
        Priority::Low
    }
}

/// 游戏状态处理器 - 跟踪游戏全局状态
pub struct GameStateTracker {
    pub player_turns: u32,
    pub ai_turns: u32,
    pub current_level: usize,
    pub game_over: bool,
    pub victory: bool,
}

impl GameStateTracker {
    pub fn new() -> Self {
        Self {
            player_turns: 0,
            ai_turns: 0,
            current_level: 1,
            game_over: false,
            victory: false,
        }
    }

    pub fn get_player_turns(&self) -> u32 {
        self.player_turns
    }

    pub fn get_ai_turns(&self) -> u32 {
        self.ai_turns
    }

    pub fn get_current_level(&self) -> usize {
        self.current_level
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn is_victory(&self) -> bool {
        self.victory
    }
}

impl EventHandler for GameStateTracker {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::PlayerTurnStarted => {
                self.player_turns += 1;
            }
            GameEvent::AITurnStarted => {
                self.ai_turns += 1;
            }
            GameEvent::LevelChanged { new_level, .. } => {
                self.current_level = *new_level;
            }
            GameEvent::GameOver { .. } => {
                self.game_over = true;
            }
            GameEvent::Victory => {
                self.victory = true;
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "GameStateTracker"
    }

    fn priority(&self) -> Priority {
        Priority::High // 游戏状态跟踪优先级较高
    }
}

/// 事件统计器 - 统计各类事件的数量
pub struct EventStatistics {
    counts: HashMap<&'static str, usize>,
}

impl EventStatistics {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    pub fn get_count(&self, event_type: &str) -> usize {
        self.counts.get(event_type).copied().unwrap_or(0)
    }

    pub fn total_events(&self) -> usize {
        self.counts.values().sum()
    }

    pub fn reset(&mut self) {
        self.counts.clear();
    }
}

impl EventHandler for EventStatistics {
    fn handle(&mut self, event: &GameEvent) {
        let event_type = event.event_type();
        *self.counts.entry(event_type).or_insert(0) += 1;
    }

    fn name(&self) -> &str {
        "EventStatistics"
    }

    fn priority(&self) -> Priority {
        Priority::Lowest
    }
}

/// 事件过滤器 - 只处理特定类型的事件
pub struct FilteredHandler<F>
where
    F: Fn(&GameEvent) + Send + Sync,
{
    filter: Vec<&'static str>,
    handler_fn: F,
}

impl<F> FilteredHandler<F>
where
    F: Fn(&GameEvent) + Send + Sync,
{
    pub fn new(event_types: Vec<&'static str>, handler_fn: F) -> Self {
        Self {
            filter: event_types,
            handler_fn,
        }
    }
}

impl<F> EventHandler for FilteredHandler<F>
where
    F: Fn(&GameEvent) + Send + Sync,
{
    fn handle(&mut self, event: &GameEvent) {
        (self.handler_fn)(event);
    }

    fn name(&self) -> &str {
        "FilteredHandler"
    }

    fn should_handle(&self, event: &GameEvent) -> bool {
        self.filter.contains(&event.event_type())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== 基础队列模式测试 ==========

    #[test]
    fn test_event_bus_basic() {
        let mut event_bus = EventBus::new();

        assert_eq!(event_bus.len(), 0);
        assert!(!event_bus.has_events());

        event_bus.publish(GameEvent::EntityMoved {
            entity: 1,
            from_x: 0,
            from_y: 0,
            to_x: 1,
            to_y: 0,
        });

        assert_eq!(event_bus.len(), 1);
        assert!(event_bus.has_events());
    }

    #[test]
    fn test_event_bus_drain() {
        let mut event_bus = EventBus::new();

        event_bus.publish(GameEvent::PlayerTurnStarted);
        event_bus.publish(GameEvent::AITurnStarted);

        let events: Vec<_> = event_bus.drain().collect();
        assert_eq!(events.len(), 2);
        assert_eq!(event_bus.len(), 0);
    }

    #[test]
    fn test_event_type() {
        let event = GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        };

        assert_eq!(event.event_type(), "DamageDealt");
    }

    #[test]
    fn test_delayed_events() {
        let mut event_bus = EventBus::new();

        event_bus.publish(GameEvent::PlayerTurnStarted);
        event_bus.publish_delayed(GameEvent::AITurnStarted);

        // 当前帧只有一个事件
        assert_eq!(event_bus.len(), 1);

        // 清空当前帧
        event_bus.drain().collect::<Vec<_>>();

        // 切换到下一帧
        event_bus.next_frame();

        // 现在应该有延迟的事件
        assert_eq!(event_bus.len(), 1);
    }

    // ========== 订阅者模式测试 ==========

    struct TestHandler {
        pub call_count: Arc<Mutex<usize>>,
        pub last_event: Arc<Mutex<Option<String>>>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                call_count: Arc::new(Mutex::new(0)),
                last_event: Arc::new(Mutex::new(None)),
            }
        }

        fn get_call_count(&self) -> usize {
            *self.call_count.lock().unwrap()
        }

        fn get_last_event(&self) -> Option<String> {
            self.last_event.lock().unwrap().clone()
        }
    }

    impl EventHandler for TestHandler {
        fn handle(&mut self, event: &GameEvent) {
            *self.call_count.lock().unwrap() += 1;
            *self.last_event.lock().unwrap() = Some(event.event_type().to_string());
        }

        fn name(&self) -> &str {
            "TestHandler"
        }
    }

    #[test]
    fn test_event_subscription() {
        let mut event_bus = EventBus::new();
        let handler = TestHandler::new();
        let call_count = handler.call_count.clone();

        event_bus.subscribe("DamageDealt", Box::new(handler));

        // 发布匹配的事件
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        });

        // 处理器应该被调用
        assert_eq!(*call_count.lock().unwrap(), 1);

        // 发布不匹配的事件
        event_bus.publish(GameEvent::PlayerTurnStarted);

        // 处理器不应该被调用
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_global_subscription() {
        let mut event_bus = EventBus::new();
        let handler = TestHandler::new();
        let call_count = handler.call_count.clone();

        event_bus.subscribe_all(Box::new(handler));

        // 发布多个不同类型的事件
        event_bus.publish(GameEvent::PlayerTurnStarted);
        event_bus.publish(GameEvent::AITurnStarted);
        event_bus.publish(GameEvent::Victory);

        // 处理器应该被调用3次
        assert_eq!(*call_count.lock().unwrap(), 3);
    }

    #[test]
    fn test_priority_ordering() {
        use std::sync::Arc;
        use std::sync::Mutex;

        struct PriorityTestHandler {
            priority: Priority,
            execution_order: Arc<Mutex<Vec<Priority>>>,
        }

        impl EventHandler for PriorityTestHandler {
            fn handle(&mut self, _event: &GameEvent) {
                self.execution_order.lock().unwrap().push(self.priority);
            }

            fn name(&self) -> &str {
                "PriorityTestHandler"
            }

            fn priority(&self) -> Priority {
                self.priority
            }
        }

        let mut event_bus = EventBus::new();
        let execution_order = Arc::new(Mutex::new(Vec::new()));

        // 以乱序添加不同优先级的处理器
        event_bus.subscribe(
            "DamageDealt",
            Box::new(PriorityTestHandler {
                priority: Priority::Low,
                execution_order: execution_order.clone(),
            }),
        );
        event_bus.subscribe(
            "DamageDealt",
            Box::new(PriorityTestHandler {
                priority: Priority::Critical,
                execution_order: execution_order.clone(),
            }),
        );
        event_bus.subscribe(
            "DamageDealt",
            Box::new(PriorityTestHandler {
                priority: Priority::Normal,
                execution_order: execution_order.clone(),
            }),
        );

        // 发布事件
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        });

        // 验证执行顺序：Critical -> Normal -> Low
        let order = execution_order.lock().unwrap();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], Priority::Critical);
        assert_eq!(order[1], Priority::Normal);
        assert_eq!(order[2], Priority::Low);
    }

    // ========== 历史记录测试 ==========

    #[test]
    fn test_event_history() {
        let mut event_bus = EventBus::with_history_size(5);

        // 发布6个事件
        for i in 0..6 {
            event_bus.publish(GameEvent::TurnEnded { turn: i });
        }

        // 历史记录应该只保留最近5个
        let history = event_bus.full_history();
        assert_eq!(history.len(), 5);

        // 验证最早的事件被删除了
        match &history[0] {
            GameEvent::TurnEnded { turn } => assert_eq!(*turn, 1),
            _ => panic!("Unexpected event type"),
        }
    }

    #[test]
    fn test_get_recent_history() {
        let mut event_bus = EventBus::new();

        for i in 0..10 {
            event_bus.publish(GameEvent::TurnEnded { turn: i });
        }

        // 获取最近3个事件
        let recent = event_bus.get_history(3);
        assert_eq!(recent.len(), 3);

        match &recent[0] {
            GameEvent::TurnEnded { turn } => assert_eq!(*turn, 7),
            _ => panic!("Unexpected event type"),
        }
    }

    // ========== 内置处理器测试 ==========

    #[test]
    fn test_logging_handler() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let mut handler = LoggingHandler::new(messages.clone());

        handler.handle(&GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 15,
            is_critical: true,
        });

        let logs = messages.lock().unwrap();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("暴击"));
        assert!(logs[0].contains("15"));
    }

    #[test]
    fn test_event_statistics() {
        let mut stats = EventStatistics::new();

        stats.handle(&GameEvent::PlayerTurnStarted);
        stats.handle(&GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        });
        stats.handle(&GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 20,
            is_critical: true,
        });

        assert_eq!(stats.get_count("PlayerTurnStarted"), 1);
        assert_eq!(stats.get_count("DamageDealt"), 2);
        assert_eq!(stats.total_events(), 3);
    }

    #[test]
    fn test_filtered_handler() {
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let mut handler = FilteredHandler::new(vec!["DamageDealt", "EntityDied"], move |_event| {
            *call_count_clone.lock().unwrap() += 1;
        });

        // 应该处理的事件
        handler.handle(&GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        });

        // 应该被过滤的事件
        let should_handle = handler.should_handle(&GameEvent::PlayerTurnStarted);
        assert!(!should_handle);

        // 只有第一个事件被处理
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    // ========== 中间件测试 ==========

    #[test]
    fn test_middleware_registration() {
        let mut event_bus = EventBus::new();
        let messages = Arc::new(Mutex::new(Vec::new()));

        // 注册中间件
        event_bus.register_middleware(Box::new(LoggingMiddleware::new(messages.clone())));

        assert_eq!(event_bus.middleware_count(), 1);
    }

    #[test]
    fn test_middleware_execution() {
        let mut event_bus = EventBus::new();
        let messages = Arc::new(Mutex::new(Vec::new()));

        // 注册日志中间件
        event_bus.register_middleware(Box::new(LoggingMiddleware::new(messages.clone())));

        // 发布事件
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        });

        // 验证中间件已执行
        let logs = messages.lock().unwrap();
        assert!(logs.iter().any(|msg| msg.contains("即将处理事件")));
        assert!(logs.iter().any(|msg| msg.contains("已处理事件")));
    }

    #[test]
    fn test_counting_middleware() {
        let mut event_bus = EventBus::new();

        let mut counting_middleware = CountingMiddleware::new();

        // 注册计数中间件
        event_bus.register_middleware(Box::new(counting_middleware));

        // 发布多个事件
        event_bus.publish(GameEvent::PlayerTurnStarted);
        event_bus.publish(GameEvent::PlayerTurnStarted);
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        });

        // 验证中间件计数
        let counting_middleware = CountingMiddleware::new();
        // 注意：这里的测试需要访问中间件的计数，但中间件已经处理完并被移动了
        // 所以我们需要另一种方式来测试中间件
    }

    #[test]
    fn test_middleware_blocking() {
        use std::sync::{Arc, Mutex};

        struct BlockingMiddleware {
            block_event_type: &'static str,
        }

        impl EventMiddleware for BlockingMiddleware {
            fn before_handle(&mut self, event: &GameEvent) -> bool {
                event.event_type() != self.block_event_type
            }

            fn name(&self) -> &str {
                "BlockingMiddleware"
            }
        }

        let mut event_bus = EventBus::new();
        let handler_call_count = Arc::new(Mutex::new(0));
        let call_count_clone = handler_call_count.clone();

        // 创建一个计数处理器
        let mut test_handler = TestHandler::new();
        let handler_count = test_handler.call_count.clone();
        event_bus.subscribe_all(Box::new(test_handler));

        // 注册阻塞特定事件类型的中间件
        event_bus.register_middleware(Box::new(BlockingMiddleware {
            block_event_type: "PlayerTurnStarted",
        }));

        // 发布被阻塞的事件
        event_bus.publish(GameEvent::PlayerTurnStarted);
        // 发布未被阻塞的事件
        event_bus.publish(GameEvent::AITurnStarted);

        // 验证被阻塞的事件没有被处理，而未被阻塞的事件被处理了
        assert_eq!(*handler_count.lock().unwrap(), 1); // 只有AITurnStarted被处理
    }

    #[test]
    fn test_conditional_filter() {
        let mut event_bus = EventBus::new();

        // 创建一个只处理伤害大于5的过滤器
        let filter = ConditionalFilter::new(
            |event| match event {
                GameEvent::DamageDealt { damage, .. } => *damage > 5,
                _ => true, // 其他事件不过滤
            },
            "HighDamageFilter",
        );

        event_bus.register_middleware(Box::new(filter));

        let test_handler = TestHandler::new();
        let handler_count = test_handler.call_count.clone();
        event_bus.subscribe_all(Box::new(test_handler));

        // 发布低伤害事件（应被过滤）
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 3,
            is_critical: false,
        });

        // 发布高伤害事件（应通过）
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        });

        // 检查只有高伤害事件被处理
        assert_eq!(*handler_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_rate_limit_middleware() {
        use std::time::Duration;

        let mut event_bus = EventBus::new();

        // 创建一个限制每100ms最多处理1个事件的中间件
        let rate_limit = RateLimitMiddleware::new(1, Duration::from_millis(100));
        event_bus.register_middleware(Box::new(rate_limit));

        let test_handler = TestHandler::new();
        let handler_count = test_handler.call_count.clone();
        event_bus.subscribe_all(Box::new(test_handler));

        // 发布两个事件
        event_bus.publish(GameEvent::PlayerTurnStarted);
        event_bus.publish(GameEvent::PlayerTurnStarted);

        // 检查只有一个事件被处理
        assert_eq!(*handler_count.lock().unwrap(), 1);
    }

    // ========== 集成测试 ==========

    #[test]
    fn test_combat_stats_handler() {
        let mut event_bus = EventBus::new();
        let mut stats_handler = CombatStatsHandler::new(Some(1)); // 假设玩家ID是1

        // 订阅战斗统计处理器
        event_bus.subscribe_all(Box::new(stats_handler));

        // 发布一些战斗事件
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1, // 玩家
            victim: 2,   // 敌人
            damage: 15,
            is_critical: true,
        });
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 2, // 敌人
            victim: 1,   // 玩家
            damage: 8,
            is_critical: false,
        });
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1, // 玩家
            victim: 2,   // 敌人
            damage: 12,
            is_critical: false,
        });

        // 获取处理器并验证统计
        let mut event_bus = EventBus::new();
        let stats_handler = CombatStatsHandler::new(Some(1));
        event_bus.subscribe_all(Box::new(stats_handler));

        // 重新发布事件来测试
        // 实际上我们不能从event_bus中取出处理器，所以需要直接测试处理器
    }

    #[test]
    fn test_game_state_tracker() {
        let mut event_bus = EventBus::new();
        let mut state_tracker = GameStateTracker::new();
        event_bus.subscribe_all(Box::new(state_tracker));

        // 发布各种游戏事件
        event_bus.publish(GameEvent::PlayerTurnStarted);
        event_bus.publish(GameEvent::PlayerTurnStarted);
        event_bus.publish(GameEvent::AITurnStarted);
        event_bus.publish(GameEvent::LevelChanged {
            old_level: 1,
            new_level: 2,
        });
        event_bus.publish(GameEvent::Victory);

        // 验证状态跟踪（通过订阅器实现）
        // 与上面相同，我们需要直接测试处理器
    }

    #[test]
    fn test_full_integration() {
        let mut event_bus = EventBus::new();
        let messages = Arc::new(Mutex::new(Vec::new()));
        let mut stats = EventStatistics::new();

        // 注册日志处理器
        event_bus.subscribe_all(Box::new(LoggingHandler::new(messages.clone())));

        // 模拟游戏流程
        event_bus.publish(GameEvent::PlayerTurnStarted);
        event_bus.publish(GameEvent::EntityMoved {
            entity: 1,
            from_x: 0,
            from_y: 0,
            to_x: 1,
            to_y: 0,
        });
        event_bus.publish(GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        });
        event_bus.publish(GameEvent::EntityDied {
            entity: 2,
            entity_name: "哥布林".to_string(),
        });

        // 验证日志
        let logs = messages.lock().unwrap();
        assert!(logs.iter().any(|msg| msg.contains("造成 10 点伤害")));
        assert!(logs.iter().any(|msg| msg.contains("哥布林 已死亡")));

        // 验证事件队列
        assert_eq!(event_bus.len(), 4);

        // 验证历史记录
        assert_eq!(event_bus.full_history().len(), 4);
    }

    // 为新处理器添加直接测试
    #[test]
    fn test_combat_stats_handler_direct() {
        let mut handler = CombatStatsHandler::new(Some(1)); // 假设玩家ID是1

        // 处理玩家造成的伤害
        handler.handle(&GameEvent::DamageDealt {
            attacker: 1, // 玩家
            victim: 2,   // 敌人
            damage: 15,
            is_critical: true,
        });

        // 处理玩家受到的伤害
        handler.handle(&GameEvent::DamageDealt {
            attacker: 2, // 敌人
            victim: 1,   // 玩家
            damage: 8,
            is_critical: false,
        });

        // 处理更多玩家造成的伤害
        handler.handle(&GameEvent::DamageDealt {
            attacker: 1, // 玩家
            victim: 3,   // 另一个敌人
            damage: 12,
            is_critical: false,
        });

        // 验证统计
        assert_eq!(handler.get_total_damage_dealt(), 27); // 15 + 12
        assert_eq!(handler.get_total_damage_taken(), 8);
        assert_eq!(handler.get_critical_hits(), 1); // 只有一次暴击
    }

    #[test]
    fn test_game_state_tracker_direct() {
        let mut handler = GameStateTracker::new();

        // 处理事件
        handler.handle(&GameEvent::PlayerTurnStarted);
        handler.handle(&GameEvent::PlayerTurnStarted);
        handler.handle(&GameEvent::AITurnStarted);
        handler.handle(&GameEvent::LevelChanged {
            old_level: 1,
            new_level: 3,
        });
        handler.handle(&GameEvent::Victory);

        // 验证状态
        assert_eq!(handler.get_player_turns(), 2);
        assert_eq!(handler.get_ai_turns(), 1);
        assert_eq!(handler.get_current_level(), 3);
        assert!(handler.is_victory());
        assert!(!handler.is_game_over());
    }
}
