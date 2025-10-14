//! 事件总线系统，用于解耦模块间通信
//!
//! 该系统提供了完整的发布-订阅机制，允许各子模块：
//! - 发布事件到总线
//! - 注册事件监听器
//! - 按优先级处理事件
//! - 使用中间件拦截和转换事件

use hecs::Entity;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::any::Any;

/// 游戏事件定义 - 用于模块间解耦通信
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    // ===== 移动事件 =====
    /// 实体移动
    EntityMoved { entity: u32, from_x: i32, from_y: i32, to_x: i32, to_y: i32 },

    // ===== 战斗事件 =====
    /// 战斗开始
    CombatStarted { attacker: u32, defender: u32 },
    /// 造成伤害
    DamageDealt { attacker: u32, victim: u32, damage: u32, is_critical: bool },
    /// 实体死亡
    EntityDied { entity: u32, entity_name: String },
    /// 状态效果应用
    StatusApplied { entity: u32, status: String, duration: u32 },
    /// 状态效果移除
    StatusRemoved { entity: u32, status: String },

    // ===== AI 事件 =====
    /// AI 做出决策
    AIDecisionMade { entity: u32, decision: String },
    /// AI 目标改变
    AITargetChanged { entity: u32, old_target: Option<u32>, new_target: Option<u32> },

    // ===== 物品事件 =====
    /// 拾取物品
    ItemPickedUp { entity: u32, item_name: String },
    /// 丢弃物品
    ItemDropped { entity: u32, item_name: String },
    /// 使用物品
    ItemUsed { entity: u32, item_name: String, effect: String },
    /// 装备物品
    ItemEquipped { entity: u32, item_name: String, slot: String },
    /// 卸下物品
    ItemUnequipped { entity: u32, item_name: String, slot: String },

    // ===== 游戏状态事件 =====
    /// 回合结束
    TurnEnded { turn: u32 },
    /// 玩家回合开始
    PlayerTurnStarted,
    /// AI 回合开始
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

/// 事件处理器 trait
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
}

/// 事件处理器包装器，包含优先级信息
struct HandlerEntry {
    handler: Box<dyn EventHandler>,
    priority: Priority,
}

/// 增强的事件总线 - 支持订阅模式和队列模式
///
/// 设计思路：
/// 1. 保留原有的队列模式（适合游戏循环）
/// 2. 添加订阅者模式（适合模块解耦）
/// 3. 支持事件优先级和过滤
pub struct EventBus {
    /// 当前帧的事件队列
    events: Vec<GameEvent>,
    /// 下一帧的事件队列
    next_frame_events: Vec<GameEvent>,
    /// 注册的事件处理器（按事件类型分组）
    handlers: HashMap<&'static str, Vec<HandlerEntry>>,
    /// 全局事件处理器（处理所有事件）
    global_handlers: Vec<HandlerEntry>,
    /// 事件历史（用于调试和回放）
    history: Vec<GameEvent>,
    /// 历史记录的最大长度
    max_history: usize,
}

impl EventBus {
    pub fn new() -> Self {
        Self::with_history_size(100)
    }

    /// 创建一个指定历史记录大小的事件总线
    pub fn with_history_size(max_history: usize) -> Self {
        Self {
            events: Vec::new(),
            next_frame_events: Vec::new(),
            handlers: HashMap::new(),
            global_handlers: Vec::new(),
            history: Vec::new(),
            max_history,
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
        self.global_handlers.sort_by(|a, b| a.priority.cmp(&b.priority));
    }

    /// 分发事件给所有订阅者
    fn dispatch_to_handlers(&mut self, event: &GameEvent) {
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
        self.global_handlers.len() +
        self.handlers.values().map(|v| v.len()).sum::<usize>()
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
        }
    }
}

// ========== 辅助宏和类型 ==========

/// 创建一个简单的事件处理器的宏
///
/// # 示例
/// ```
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
            GameEvent::DamageDealt { damage, is_critical, .. } => {
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
            GameEvent::ItemUsed { item_name, effect, .. } => {
                format!("使用了 {}，{}", item_name, effect)
            }
            GameEvent::LevelChanged { old_level, new_level } => {
                format!("从第 {} 层进入第 {} 层", old_level, new_level)
            }
            GameEvent::LogMessage { message, .. } => {
                message.clone()
            }
            GameEvent::TrapTriggered { trap_type, .. } => {
                format!("触发了{}陷阱！", trap_type)
            }
            GameEvent::StatusApplied { status, duration, .. } => {
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
        event_bus.subscribe("DamageDealt", Box::new(PriorityTestHandler {
            priority: Priority::Low,
            execution_order: execution_order.clone(),
        }));
        event_bus.subscribe("DamageDealt", Box::new(PriorityTestHandler {
            priority: Priority::Critical,
            execution_order: execution_order.clone(),
        }));
        event_bus.subscribe("DamageDealt", Box::new(PriorityTestHandler {
            priority: Priority::Normal,
            execution_order: execution_order.clone(),
        }));

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

        let mut handler = FilteredHandler::new(
            vec!["DamageDealt", "EntityDied"],
            move |_event| {
                *call_count_clone.lock().unwrap() += 1;
            },
        );

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

    // ========== 集成测试 ==========

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
}