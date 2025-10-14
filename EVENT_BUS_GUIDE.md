# 事件总线使用指南

## 概述

事件总线（EventBus）是 Terminal Pixel Dungeon 项目中用于解耦各子模块的核心通信机制。它支持：

- **发布-订阅模式**：模块可以发布事件并订阅感兴趣的事件
- **优先级系统**：控制事件处理器的执行顺序
- **事件过滤**：处理器可以选择性地处理特定事件
- **历史记录**：保留最近的事件历史用于调试
- **队列模式**：兼容游戏循环的单向数据流

## 核心概念

### 1. GameEvent 枚举

所有游戏事件都定义在 `GameEvent` 枚举中：

```rust
pub enum GameEvent {
    // 移动事件
    EntityMoved { entity: u32, from_x: i32, from_y: i32, to_x: i32, to_y: i32 },

    // 战斗事件
    CombatStarted { attacker: u32, defender: u32 },
    DamageDealt { attacker: u32, victim: u32, damage: u32, is_critical: bool },
    EntityDied { entity: u32, entity_name: String },

    // 物品事件
    ItemPickedUp { entity: u32, item_name: String },
    ItemUsed { entity: u32, item_name: String, effect: String },

    // 游戏状态事件
    GameOver { reason: String },
    Victory,

    // ... 更多事件
}
```

### 2. EventHandler Trait

所有事件处理器都需要实现 `EventHandler` trait：

```rust
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
```

### 3. 优先级

```rust
pub enum Priority {
    Critical = 0,  // 关键系统事件
    High = 1,      // 游戏核心逻辑
    Normal = 2,    // 默认优先级
    Low = 3,       // UI 更新等
    Lowest = 4,    // 日志记录等
}
```

## 基本使用

### 发布事件

#### 立即发布（当前帧）

```rust
// 在 ECSWorld 中
ecs_world.publish_event(GameEvent::DamageDealt {
    attacker: 1,
    victim: 2,
    damage: 10,
    is_critical: false,
});

// 或直接使用事件总线
event_bus.publish(GameEvent::PlayerTurnStarted);
```

#### 延迟发布（下一帧）

```rust
event_bus.publish_delayed(GameEvent::EntityDied {
    entity: 2,
    entity_name: "哥布林".to_string(),
});
```

### 订阅事件

#### 订阅特定类型的事件

```rust
// 创建自定义处理器
struct CombatLogger {
    log: Vec<String>,
}

impl EventHandler for CombatLogger {
    fn handle(&mut self, event: &GameEvent) {
        if let GameEvent::DamageDealt { damage, is_critical, .. } = event {
            let msg = if *is_critical {
                format!("暴击！造成 {} 点伤害", damage)
            } else {
                format!("造成 {} 点伤害", damage)
            };
            self.log.push(msg);
        }
    }

    fn name(&self) -> &str {
        "CombatLogger"
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

// 注册处理器
event_bus.subscribe("DamageDealt", Box::new(CombatLogger { log: Vec::new() }));
```

#### 订阅所有事件

```rust
// 使用内置的日志处理器
let messages = Arc::new(Mutex::new(Vec::new()));
event_bus.subscribe_all(Box::new(LoggingHandler::new(messages.clone())));
```

## 高级用法

### 1. 使用内置处理器

#### LoggingHandler - 日志记录

```rust
use std::sync::{Arc, Mutex};

let messages = Arc::new(Mutex::new(Vec::new()));
let logger = LoggingHandler::new(messages.clone());
event_bus.subscribe_all(Box::new(logger));

// 所有重要事件都会被记录到 messages 中
```

#### EventStatistics - 事件统计

```rust
let mut stats = EventStatistics::new();
event_bus.subscribe_all(Box::new(stats));

// 查询统计信息
println!("伤害事件数量: {}", stats.get_count("DamageDealt"));
println!("总事件数: {}", stats.total_events());
```

#### FilteredHandler - 过滤事件

```rust
let handler = FilteredHandler::new(
    vec!["DamageDealt", "EntityDied"],
    |event| {
        println!("战斗相关事件: {:?}", event);
    },
);
event_bus.subscribe_all(Box::new(handler));
```

### 2. 优先级控制

```rust
struct HealthUpdater;

impl EventHandler for HealthUpdater {
    fn handle(&mut self, event: &GameEvent) {
        // 更新生命值
    }

    fn name(&self) -> &str {
        "HealthUpdater"
    }

    // 高优先级 - 确保在 UI 更新前执行
    fn priority(&self) -> Priority {
        Priority::High
    }
}
```

### 3. 事件过滤

```rust
struct BossEventHandler;

impl EventHandler for BossEventHandler {
    fn handle(&mut self, event: &GameEvent) {
        // 只处理与 Boss 相关的战斗事件
    }

    fn name(&self) -> &str {
        "BossEventHandler"
    }

    fn should_handle(&self, event: &GameEvent) -> bool {
        match event {
            GameEvent::DamageDealt { attacker, .. } => {
                // 只处理 Boss 攻击事件（假设 Boss entity ID > 1000）
                *attacker > 1000
            }
            _ => false,
        }
    }
}
```

### 4. 历史记录

```rust
// 获取最近 10 个事件
let recent_events = event_bus.get_history(10);

// 获取所有历史记录
let all_events = event_bus.full_history();

// 清空历史记录
event_bus.clear_history();
```

## 模块解耦示例

### Combat 模块发布事件

```rust
// src/combat/src/combat_manager.rs

impl CombatManager {
    pub fn resolve_attack(&mut self, event_bus: &mut EventBus) {
        // 发布战斗开始事件
        event_bus.publish(GameEvent::CombatStarted {
            attacker: self.attacker_id,
            defender: self.defender_id,
        });

        // 计算伤害
        let damage = self.calculate_damage();

        // 发布伤害事件
        event_bus.publish(GameEvent::DamageDealt {
            attacker: self.attacker_id,
            victim: self.defender_id,
            damage,
            is_critical: self.is_critical_hit(),
        });

        // 如果目标死亡，发布延迟事件
        if self.is_target_dead() {
            event_bus.publish_delayed(GameEvent::EntityDied {
                entity: self.defender_id,
                entity_name: self.defender_name.clone(),
            });
        }
    }
}
```

### UI 模块订阅事件

```rust
// src/ui/src/states/game.rs

struct GameUIHandler {
    message_log: Vec<String>,
}

impl EventHandler for GameUIHandler {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::DamageDealt { damage, is_critical, .. } => {
                let msg = if *is_critical {
                    format!("💥 暴击！造成 {} 点伤害", damage)
                } else {
                    format!("⚔️ 造成 {} 点伤害", damage)
                };
                self.message_log.push(msg);
            }
            GameEvent::EntityDied { entity_name, .. } => {
                self.message_log.push(format!("💀 {} 已死亡", entity_name));
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "GameUIHandler"
    }

    fn priority(&self) -> Priority {
        Priority::Low  // UI 更新优先级较低
    }
}
```

### Save 模块订阅事件

```rust
// src/save/src/lib.rs

struct AutoSaveHandler {
    save_system: SaveSystem,
    last_save_time: Instant,
}

impl EventHandler for AutoSaveHandler {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::TurnEnded { .. } => {
                // 每隔 5 分钟自动保存
                if self.last_save_time.elapsed() > Duration::from_secs(300) {
                    // 触发自动保存
                    self.last_save_time = Instant::now();
                }
            }
            GameEvent::GameOver { .. } | GameEvent::Victory => {
                // 游戏结束时立即保存
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "AutoSaveHandler"
    }

    fn priority(&self) -> Priority {
        Priority::High  // 保存系统优先级较高
    }
}
```

## 最佳实践

### 1. 事件命名规范

- 使用过去时表示已发生的事件：`EntityMoved`, `DamageDealt`, `ItemUsed`
- 使用现在时表示状态事件：`PlayerTurnStarted`, `GamePaused`

### 2. 事件粒度

- **太细**：`PixelDrawn`, `MemoryAllocated` ❌
- **太粗**：`GameUpdated`, `SomethingChanged` ❌
- **适中**：`EntityMoved`, `DamageDealt`, `ItemPickedUp` ✅

### 3. 避免循环依赖

```rust
// ❌ 错误：处理器内部发布相同类型的事件
impl EventHandler for BadHandler {
    fn handle(&mut self, event: &GameEvent) {
        if let GameEvent::DamageDealt { .. } = event {
            // 不要在处理器内部直接发布事件！
            // self.event_bus.publish(GameEvent::DamageDealt { ... });
        }
    }
}

// ✅ 正确：使用延迟发布或发布不同类型的事件
impl EventHandler for GoodHandler {
    fn handle(&mut self, event: &GameEvent) {
        if let GameEvent::DamageDealt { victim, .. } = event {
            // 检查是否死亡，发布不同类型的事件
            if self.check_death(*victim) {
                // 通过外部接口发布延迟事件
            }
        }
    }
}
```

### 4. 优先级使用指南

- **Critical**: 游戏崩溃处理、紧急保存
- **High**: 核心游戏逻辑（战斗、移动）
- **Normal**: 一般游戏功能（物品、AI）
- **Low**: UI 更新、音效播放
- **Lowest**: 日志记录、统计收集

### 5. 性能考虑

```rust
// ❌ 避免在热路径中频繁发布事件
for pixel in pixels {
    event_bus.publish(GameEvent::PixelUpdated { ... }); // 太频繁！
}

// ✅ 批量处理或只在关键时刻发布事件
if important_change {
    event_bus.publish(GameEvent::ScreenUpdated { ... });
}
```

## 游戏循环集成

```rust
// src/game_loop.rs

impl GameLoop {
    fn update_turn(&mut self) -> anyhow::Result<()> {
        // 1. 运行所有系统
        for system in &mut self.systems {
            system.run(&mut self.ecs_world.world, &mut self.ecs_world.resources)?;
        }

        // 2. 处理当前帧的所有事件
        self.ecs_world.process_events();

        // 3. 准备处理下一帧事件
        self.ecs_world.next_frame();

        Ok(())
    }
}
```

## 调试技巧

### 1. 启用事件历史

```rust
let mut event_bus = EventBus::with_history_size(1000);

// 游戏崩溃时查看最近的事件
if game_crashed {
    for event in event_bus.get_history(20) {
        eprintln!("最近的事件: {:?}", event);
    }
}
```

### 2. 使用事件统计

```rust
let stats = EventStatistics::new();
event_bus.subscribe_all(Box::new(stats));

// 定期打印统计信息
println!("伤害事件: {}", stats.get_count("DamageDealt"));
println!("移动事件: {}", stats.get_count("EntityMoved"));
println!("总事件数: {}", stats.total_events());
```

### 3. 日志所有事件

```rust
// 开发模式下记录所有事件
#[cfg(debug_assertions)]
{
    let messages = Arc::new(Mutex::new(Vec::new()));
    event_bus.subscribe_all(Box::new(LoggingHandler::new(messages.clone())));
}
```

## 总结

事件总线系统为项目提供了：

✅ **解耦**：模块之间不需要直接依赖
✅ **灵活**：可以动态添加和移除事件处理器
✅ **可测试**：容易模拟和测试事件流
✅ **可追踪**：事件历史便于调试
✅ **高性能**：优先级系统确保关键逻辑先执行

通过合理使用事件总线，可以构建出松耦合、易维护的游戏架构。
