//! 事件总线系统，用于解耦模块间通信

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// 事件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    PlayerMoved { x: i32, y: i32 },
    PlayerAttacked { target: String, damage: u32 },
    EnemySpawned { x: i32, y: i32, enemy_type: String },
    ItemPickedUp { item_name: String },
    ItemUsed { item_name: String },
    TurnElapsed,
    GameSaved,
    GameLoaded,
    LevelChanged { new_level: usize },
    EntityDied { entity_name: String },
}

/// 事件处理器类型
pub type EventHandler = Arc<dyn Fn(&GameEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>;

/// 事件总线系统
pub struct EventBus {
    handlers: Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 订阅特定类型的事件
    pub fn subscribe(&self, event_type: &str, handler: EventHandler) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.entry(event_type.to_string()).or_default().push(handler);
    }

    /// 发布事件
    pub fn publish(&self, event: &GameEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_type = self.get_event_type(event);
        let handlers = self.handlers.lock().unwrap();
        
        if let Some(event_handlers) = handlers.get(&event_type) {
            for handler in event_handlers {
                handler(event)?;
            }
        }
        
        Ok(())
    }
    
    /// 获取事件类型字符串
    fn get_event_type(&self, event: &GameEvent) -> String {
        match event {
            GameEvent::PlayerMoved { .. } => "PlayerMoved".to_string(),
            GameEvent::PlayerAttacked { .. } => "PlayerAttacked".to_string(),
            GameEvent::EnemySpawned { .. } => "EnemySpawned".to_string(),
            GameEvent::ItemPickedUp { .. } => "ItemPickedUp".to_string(),
            GameEvent::ItemUsed { .. } => "ItemUsed".to_string(),
            GameEvent::TurnElapsed => "TurnElapsed".to_string(),
            GameEvent::GameSaved => "GameSaved".to_string(),
            GameEvent::GameLoaded => "GameLoaded".to_string(),
            GameEvent::LevelChanged { .. } => "LevelChanged".to_string(),
            GameEvent::EntityDied { .. } => "EntityDied".to_string(),
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus() {
        let event_bus = EventBus::new();
        
        let called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let called_clone = called.clone();
        
        event_bus.subscribe(
            "TestEvent",
            Arc::new(move |event| {
                if let GameEvent::PlayerMoved { x, y } = event {
                    if *x == 5 && *y == 10 {
                        *called_clone.lock().unwrap() = true;
                    }
                }
                Ok(())
            }),
        );
        
        let result = event_bus.publish(&GameEvent::PlayerMoved { x: 5, y: 10 });
        assert!(result.is_ok());
        
        assert!(*called.lock().unwrap());
    }
}