# 事件总线快速入门

5 分钟快速掌握 Terminal Pixel Dungeon 的事件系统！

## 1. 发布事件（2 分钟）

### 基本发布
```rust
use terminal_pixel_dungeon::event_bus::GameEvent;

// 在你的代码中获取 ECSWorld
let mut ecs_world = /* ... */;

// 发布事件 - 立即触发所有订阅者
ecs_world.publish_event(GameEvent::DamageDealt {
    attacker: 1,
    victim: 2,
    damage: 50,
    is_critical: true,
});
```

### 延迟发布
```rust
// 下一帧才处理（避免借用冲突）
ecs_world.publish_delayed_event(GameEvent::EntityDied {
    entity: 2,
    entity_name: "哥布林".to_string(),
});
```

## 2. 订阅事件（3 分钟）

### 步骤 1: 创建处理器

```rust
use terminal_pixel_dungeon::event_bus::{EventHandler, GameEvent, Priority};

struct MyHandler {
    message_log: Vec<String>,
}

impl EventHandler for MyHandler {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::DamageDealt { damage, is_critical, .. } => {
                let msg = if *is_critical {
                    format!("💥 暴击！{} 点伤害", damage)
                } else {
                    format!("⚔️ {} 点伤害", damage)
                };
                self.message_log.push(msg);
            }
            GameEvent::EntityDied { entity_name, .. } => {
                self.message_log.push(format!("💀 {} 已死亡", entity_name));
            }
            _ => {} // 忽略其他事件
        }
    }

    fn name(&self) -> &str {
        "MyHandler"
    }

    // 可选：设置优先级
    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
```

### 步骤 2: 注册处理器

```rust
let handler = MyHandler {
    message_log: Vec::new(),
};

// 订阅所有事件
ecs_world.event_bus.subscribe_all(Box::new(handler));

// 或者只订阅特定类型
ecs_world.event_bus.subscribe("DamageDealt", Box::new(handler));
```

## 3. 完整示例

```rust
use terminal_pixel_dungeon::event_bus::{EventBus, EventHandler, GameEvent, Priority};

// 1. 创建战斗日志处理器
struct CombatLogger {
    logs: Vec<String>,
}

impl EventHandler for CombatLogger {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::CombatStarted { attacker, defender } => {
                self.logs.push(format!("战斗开始！{} vs {}", attacker, defender));
            }
            GameEvent::DamageDealt { attacker, victim, damage, is_critical } => {
                let msg = if *is_critical {
                    format!("{} 暴击 {} 造成 {} 点伤害！", attacker, victim, damage)
                } else {
                    format!("{} 攻击 {} 造成 {} 点伤害", attacker, victim, damage)
                };
                self.logs.push(msg);
            }
            GameEvent::EntityDied { entity_name, .. } => {
                self.logs.push(format!("{} 已被击败", entity_name));
            }
            _ => {}
        }
    }

    fn name(&self) -> &str { "CombatLogger" }
    fn priority(&self) -> Priority { Priority::Low }
}

// 2. 使用
fn main() {
    let mut event_bus = EventBus::new();

    // 注册日志处理器
    let logger = CombatLogger { logs: Vec::new() };
    event_bus.subscribe_all(Box::new(logger));

    // 模拟战斗
    event_bus.publish(GameEvent::CombatStarted {
        attacker: 1,
        defender: 2,
    });

    event_bus.publish(GameEvent::DamageDealt {
        attacker: 1,
        victim: 2,
        damage: 50,
        is_critical: true,
    });

    event_bus.publish(GameEvent::EntityDied {
        entity: 2,
        entity_name: "哥布林".to_string(),
    });

    // 日志会自动记录到 logger.logs 中
}
```

## 4. 常见事件类型

### 战斗
```rust
GameEvent::CombatStarted { attacker, defender }
GameEvent::DamageDealt { attacker, victim, damage, is_critical }
GameEvent::EntityDied { entity, entity_name }
```

### 移动
```rust
GameEvent::EntityMoved { entity, from_x, from_y, to_x, to_y }
```

### 物品
```rust
GameEvent::ItemPickedUp { entity, item_name }
GameEvent::ItemUsed { entity, item_name, effect }
GameEvent::ItemEquipped { entity, item_name, slot }
```

### 游戏状态
```rust
GameEvent::GameOver { reason }
GameEvent::Victory
GameEvent::LevelChanged { old_level, new_level }
```

### 日志
```rust
GameEvent::LogMessage { message, level }
```

## 5. 进阶技巧

### 事件过滤
```rust
impl EventHandler for MyHandler {
    fn should_handle(&self, event: &GameEvent) -> bool {
        // 只处理高伤害事件
        match event {
            GameEvent::DamageDealt { damage, .. } => *damage > 100,
            _ => false,
        }
    }

    // ... 其他方法
}
```

### 优先级控制
```rust
impl EventHandler for CriticalHandler {
    fn priority(&self) -> Priority {
        Priority::Critical // 最先执行
    }
}

impl EventHandler for LogHandler {
    fn priority(&self) -> Priority {
        Priority::Lowest // 最后执行
    }
}
```

### 使用内置处理器
```rust
use terminal_pixel_dungeon::event_bus::{LoggingHandler, EventStatistics};
use std::sync::{Arc, Mutex};

// 日志处理器
let messages = Arc::new(Mutex::new(Vec::new()));
event_bus.subscribe_all(Box::new(LoggingHandler::new(messages.clone())));

// 统计处理器
let stats = EventStatistics::new();
event_bus.subscribe_all(Box::new(stats));
```

## 6. 常见问题

### Q: 何时使用 `publish` vs `publish_delayed`？
A:
- `publish`: 立即处理，用于大多数情况
- `publish_delayed`: 下一帧处理，用于避免借用冲突

### Q: 如何访问处理器的状态？
A: 使用 `Arc<Mutex<T>>` 共享状态：
```rust
let state = Arc::new(Mutex::new(MyState::new()));
let handler = MyHandler::new(state.clone());
event_bus.subscribe_all(Box::new(handler));

// 稍后访问
let current_state = state.lock().unwrap();
```

### Q: 处理器执行顺序？
A: 按优先级执行：Critical → High → Normal → Low → Lowest

### Q: 如何调试事件？
A: 使用事件历史：
```rust
// 获取最近10个事件
let history = event_bus.get_history(10);
for event in history {
    println!("{:?}", event);
}
```

## 7. 下一步

- 📖 阅读完整指南：`EVENT_BUS_GUIDE.md`
- 🔍 查看示例代码：`examples/event_handlers.rs`
- 📋 了解架构设计：`EVENT_BUS_SUMMARY.md`
- 💻 查看源代码：`src/event_bus.rs`

## 8. 快速参考

| 操作 | 代码 |
|------|------|
| 发布事件 | `ecs_world.publish_event(event)` |
| 延迟发布 | `ecs_world.publish_delayed_event(event)` |
| 订阅所有 | `event_bus.subscribe_all(Box::new(handler))` |
| 订阅特定 | `event_bus.subscribe("EventType", Box::new(handler))` |
| 查看历史 | `event_bus.get_history(10)` |
| 清空历史 | `event_bus.clear_history()` |

---

**就是这么简单！** 🎉

现在你可以开始使用事件总线解耦你的代码了。
