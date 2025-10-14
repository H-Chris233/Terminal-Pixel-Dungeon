# 事件总线使用指南

## 概述

事件总线是一个用于解耦模块间通信的系统，允许各子模块通过发布-订阅模式进行交互，而无需直接依赖其他模块。

## 架构设计

### 核心组件

1. **ECSWorld** - 包含世界状态和事件总线的主要容器
2. **EventBus** - 处理事件发布和订阅的总线系统
3. **GameEvent** - 定义游戏中可能发生的所有事件类型

## 事件类型

事件总线支持以下事件类型：

### 移动事件
- `EntityMoved { entity: u32, from_x: i32, from_y: i32, to_x: i32, to_y: i32 }` - 实体移动

### 战斗事件
- `CombatStarted { attacker: u32, defender: u32 }` - 战斗开始
- `DamageDealt { attacker: u32, victim: u32, damage: u32, is_critical: bool }` - 造成伤害
- `EntityDied { entity: u32, entity_name: String }` - 实体死亡
- `StatusApplied { entity: u32, status: String, duration: u32 }` - 状态效果应用
- `StatusRemoved { entity: u32, status: String }` - 状态效果移除

### AI 事件
- `AIDecisionMade { entity: u32, decision: String }` - AI 做出决策
- `AITargetChanged { entity: u32, old_target: Option<u32>, new_target: Option<u32> }` - AI 目标改变

### 物品事件
- `ItemPickedUp { entity: u32, item_name: String }` - 拾取物品
- `ItemDropped { entity: u32, item_name: String }` - 丢弃物品
- `ItemUsed { entity: u32, item_name: String, effect: String }` - 使用物品
- `ItemEquipped { entity: u32, item_name: String, slot: String }` - 装备物品
- `ItemUnequipped { entity: u32, item_name: String, slot: String }` - 卸下物品

### 游戏状态事件
- `TurnEnded { turn: u32 }` - 回合结束
- `PlayerTurnStarted` - 玩家回合开始
- `AITurnStarted` - AI 回合开始
- `GameOver { reason: String }` - 游戏结束
- `Victory` - 游戏胜利
- `GamePaused` - 暂停游戏
- `GameResumed` - 恢复游戏

### 地牢事件
- `LevelChanged { old_level: usize, new_level: usize }` - 进入新层
- `RoomDiscovered { room_id: usize }` - 发现房间
- `TrapTriggered { entity: u32, trap_type: String }` - 触发陷阱

### 系统事件
- `GameSaved { save_slot: String }` - 保存游戏
- `GameLoaded { save_slot: String }` - 加载游戏
- `LogMessage { message: String, level: LogLevel }` - 日志消息

## 使用方法

### 发布事件

在系统中发布事件：

```rust
// 在 ECSWorld 上下文中
ecs_world.publish_event(GameEvent::DamageDealt {
    attacker: attacker_id,
    victim: victim_id,
    damage: 10,
    is_critical: false,
});

// 发布延迟事件（下帧处理）
ecs_world.publish_delayed_event(GameEvent::LevelChanged {
    old_level: 1,
    new_level: 2,
});
```

### 订阅事件

创建事件处理器：

```rust
use crate::event_bus::{EventHandler, GameEvent, Priority};

pub struct MyEventHandler {
    // 你的状态
}

impl EventHandler for MyEventHandler {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::DamageDealt { damage, .. } => {
                println!("造成了 {} 点伤害", damage);
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "MyEventHandler"
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

// 注册处理器
let mut handler = MyEventHandler { /* 初始化 */ };
ecs_world.event_bus.subscribe_all(Box::new(handler));
```

### 事件过滤

可以创建只处理特定类型事件的处理器：

```rust
use crate::event_bus::{FilteredHandler};

let combat_handler = FilteredHandler::new(
    vec!["DamageDealt", "EntityDied"],  // 只处理这些事件类型
    move |event| {
        match event {
            GameEvent::DamageDealt { damage, .. } => {
                // 处理伤害事件
            }
            GameEvent::EntityDied { .. } => {
                // 处理死亡事件
            }
            _ => {}
        }
    }
);
```

## 优先级系统

处理器按以下优先级执行：

- `Critical` - 最高优先级（用于关键系统事件）
- `High` - 高优先级（用于游戏核心逻辑）
- `Normal` - 普通优先级（默认）
- `Low` - 低优先级（用于 UI 更新等）
- `Lowest` - 最低优先级（用于日志等）

## 中间件

事件总线支持中间件，允许在事件处理前后插入逻辑：

```rust
use crate::event_bus::{EventMiddleware, GameEvent, Priority};

pub struct LoggingMiddleware {
    // 日志记录相关字段
}

impl EventMiddleware for LoggingMiddleware {
    fn before_handle(&mut self, event: &GameEvent) -> bool {
        println!("处理事件前: {:?}", event.event_type());
        true  // 返回 true 继续处理，false 阻止处理
    }

    fn after_handle(&mut self, event: &GameEvent) {
        println!("事件处理后: {:?}", event.event_type());
    }

    fn name(&self) -> &str {
        "LoggingMiddleware"
    }

    fn priority(&self) -> Priority {
        Priority::Lowest
    }
}

// 注册中间件
ecs_world.event_bus.register_middleware(Box::new(LoggingMiddleware { /* 初始化 */ }));
```

## 模块集成

### 与 ECS 系统集成

在游戏循环中，事件总线在系统执行后处理：

```rust
// 在系统执行后处理事件
ecs_world.process_events();

// 准备下一帧的事件处理
ecs_world.next_frame();
```

### 模块间通信

通过事件总线实现模块解耦：

- 战斗模块发生事件 → UI 模块更新界面
- 玩家拾取物品 → 保存模块记录状态
- 进入新楼层 → 音效模块播放音乐

## 最佳实践

1. **事件粒度** - 事件应足够具体以便精确处理，但不过于细化
2. **数据完整性** - 确保事件包含所有必要的上下文信息
3. **事件命名** - 使用清晰、一致的命名约定
4. **性能考虑** - 避免发布过于频繁的事件
5. **错误处理** - 事件处理器应能处理错误情况

## 调试

事件总线提供调试功能：

```rust
// 获取事件历史记录
let recent_events = ecs_world.event_bus.get_history(10);

// 获取订阅者数量
let subscriber_count = ecs_world.event_bus.subscriber_count();

// 清除历史记录
ecs_world.event_bus.clear_history();
```