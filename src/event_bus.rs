//! 事件总线系统，用于解耦模块间通信

use hecs::Entity;
use serde::{Serialize, Deserialize};

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

/// 简化的事件总线 - 使用队列模式而非订阅模式
/// 这样更适合游戏循环的单向数据流
pub struct EventBus {
    /// 当前帧的事件队列
    events: Vec<GameEvent>,
    /// 下一帧的事件队列
    next_frame_events: Vec<GameEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            next_frame_events: Vec::new(),
        }
    }

    /// 发布事件（添加到当前帧队列）
    pub fn publish(&mut self, event: GameEvent) {
        self.events.push(event);
    }

    /// 发布延迟事件（添加到下一帧队列）
    pub fn publish_delayed(&mut self, event: GameEvent) {
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

    /// 帧结束时调用，将下一帧事件移到当前帧
    pub fn next_frame(&mut self) {
        std::mem::swap(&mut self.events, &mut self.next_frame_events);
        self.next_frame_events.clear();
    }

    /// 清空所有事件
    pub fn clear(&mut self) {
        self.events.clear();
        self.next_frame_events.clear();
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

#[cfg(test)]
mod tests {
    use super::*;

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
}