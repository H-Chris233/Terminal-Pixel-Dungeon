# 事件总线迁移指南

从直接调用到事件驱动架构的迁移指南

## 概述

本指南帮助你将现有代码从**紧耦合的直接调用**迁移到**松耦合的事件驱动架构**。

## 为什么要迁移？

### 迁移前（紧耦合）❌
```rust
// combat.rs
pub fn deal_damage(&mut self, victim: &mut Entity, damage: u32) {
    victim.stats.hp -= damage;

    // 直接调用其他模块
    ui.show_damage_text(damage);           // 耦合到 UI
    sound.play_hit_sound();                // 耦合到音效
    achievement.check_damage(damage);      // 耦合到成就
    save.trigger_autosave();               // 耦合到保存

    if victim.stats.hp == 0 {
        victim.die();
        ui.show_death_animation();
        achievement.on_kill();
    }
}
```

**问题**：
- ❌ Combat 模块依赖 UI、Sound、Achievement、Save
- ❌ 添加新功能需要修改 Combat 代码
- ❌ 难以测试（需要模拟所有依赖）
- ❌ 无法独立开发各模块

### 迁移后（松耦合）✅
```rust
// combat.rs
pub fn deal_damage(&mut self, event_bus: &mut EventBus, victim_id: u32, damage: u32) {
    // 更新状态
    victim.stats.hp -= damage;

    // 发布事件
    event_bus.publish(GameEvent::DamageDealt {
        attacker: self.id,
        victim: victim_id,
        damage,
        is_critical: self.was_critical,
    });

    if victim.stats.hp == 0 {
        event_bus.publish_delayed(GameEvent::EntityDied {
            entity: victim_id,
            entity_name: victim.name.clone(),
        });
    }
}
```

**优势**：
- ✅ Combat 模块不依赖其他模块
- ✅ UI、Sound、Achievement 各自订阅事件
- ✅ 易于测试（只需要 EventBus）
- ✅ 可以动态添加/移除功能

## 迁移步骤

### 第 1 步：识别耦合点

查找代码中的直接调用：

```rust
// ❌ 需要迁移
ui.update_hp_bar(hp);
logger.log("玩家攻击");
save_system.mark_dirty();

// ✅ 已经是事件驱动
event_bus.publish(GameEvent::PlayerAttacked { ... });
```

### 第 2 步：定义事件

为每个耦合点定义相应的事件（如果还没有）：

```rust
// 在 src/event_bus.rs 中添加新事件
pub enum GameEvent {
    // 现有事件...

    // 新增事件
    HealthChanged {
        entity: u32,
        old_hp: u32,
        new_hp: u32,
        max_hp: u32,
    },
    ExperienceGained {
        entity: u32,
        amount: u32,
        source: String,
    },
    // ...
}
```

### 第 3 步：创建事件处理器

为每个模块创建处理器：

```rust
// ui/src/handlers.rs
pub struct UIEventHandler {
    ui_state: Arc<Mutex<UIState>>,
}

impl EventHandler for UIEventHandler {
    fn handle(&mut self, event: &GameEvent) {
        let mut ui = self.ui_state.lock().unwrap();

        match event {
            GameEvent::DamageDealt { victim, damage, .. } => {
                ui.show_damage_text(*victim, *damage);
            }
            GameEvent::HealthChanged { entity, new_hp, max_hp, .. } => {
                ui.update_hp_bar(*entity, *new_hp, *max_hp);
            }
            _ => {}
        }
    }

    fn name(&self) -> &str { "UIEventHandler" }
    fn priority(&self) -> Priority { Priority::Low }
}
```

### 第 4 步：替换直接调用

逐步替换直接调用为事件发布：

```rust
// 迁移前
fn on_player_attack(&mut self, target: &mut Enemy) {
    let damage = self.calculate_damage();
    target.take_damage(damage);

    // 直接调用
    self.ui.show_damage(damage);
    self.sound.play_attack();
}

// 迁移后
fn on_player_attack(&mut self, event_bus: &mut EventBus, target_id: u32) {
    let damage = self.calculate_damage();

    // 发布事件
    event_bus.publish(GameEvent::DamageDealt {
        attacker: self.player_id,
        victim: target_id,
        damage,
        is_critical: false,
    });
}
```

### 第 5 步：注册处理器

在游戏初始化时注册所有处理器：

```rust
// main.rs 或 game_loop.rs
fn initialize_event_handlers(ecs_world: &mut ECSWorld) {
    // UI 处理器
    let ui_handler = UIEventHandler::new(ui_state);
    ecs_world.event_bus.subscribe_all(Box::new(ui_handler));

    // 音效处理器
    let sound_handler = SoundEventHandler::new(audio_system);
    ecs_world.event_bus.subscribe_all(Box::new(sound_handler));

    // 成就处理器
    let achievement_handler = AchievementHandler::new();
    ecs_world.event_bus.subscribe_all(Box::new(achievement_handler));

    // 统计处理器
    let stats_handler = StatisticsHandler::new();
    ecs_world.event_bus.subscribe_all(Box::new(stats_handler));
}
```

### 第 6 步：测试

确保功能正常工作：

```rust
#[test]
fn test_damage_event_flow() {
    let mut event_bus = EventBus::new();
    let ui_handler = MockUIHandler::new();

    event_bus.subscribe_all(Box::new(ui_handler));

    // 发布事件
    event_bus.publish(GameEvent::DamageDealt {
        attacker: 1,
        victim: 2,
        damage: 50,
        is_critical: true,
    });

    // 验证 UI 处理器收到事件
    // assert!(ui_handler.received_damage_event());
}
```

## 实际迁移案例

### 案例 1：战斗系统

#### 迁移前
```rust
// combat/src/combat_manager.rs
impl CombatManager {
    pub fn execute_attack(
        &mut self,
        attacker: &mut Entity,
        defender: &mut Entity,
        ui: &mut UI,
        sound: &mut SoundSystem,
    ) {
        let damage = self.calculate_damage(attacker, defender);
        defender.stats.hp -= damage;

        // 紧耦合调用
        ui.show_damage_popup(damage);
        sound.play_hit_sound();

        if defender.stats.hp == 0 {
            ui.show_death_animation(defender.id);
            sound.play_death_sound();
        }
    }
}
```

#### 迁移后
```rust
// combat/src/combat_manager.rs
impl CombatManager {
    pub fn execute_attack(
        &mut self,
        event_bus: &mut EventBus,
        attacker_id: u32,
        defender_id: u32,
    ) {
        let damage = self.calculate_damage(attacker_id, defender_id);

        // 发布战斗事件
        event_bus.publish(GameEvent::CombatStarted {
            attacker: attacker_id,
            defender: defender_id,
        });

        event_bus.publish(GameEvent::DamageDealt {
            attacker: attacker_id,
            victim: defender_id,
            damage,
            is_critical: self.was_critical_hit(),
        });

        // 检查死亡
        if self.is_defender_dead(defender_id) {
            event_bus.publish_delayed(GameEvent::EntityDied {
                entity: defender_id,
                entity_name: self.get_entity_name(defender_id),
            });
        }
    }
}

// ui/src/combat_ui_handler.rs
pub struct CombatUIHandler {
    ui: Arc<Mutex<UI>>,
}

impl EventHandler for CombatUIHandler {
    fn handle(&mut self, event: &GameEvent) {
        let mut ui = self.ui.lock().unwrap();

        match event {
            GameEvent::DamageDealt { damage, .. } => {
                ui.show_damage_popup(*damage);
            }
            GameEvent::EntityDied { entity, .. } => {
                ui.show_death_animation(*entity);
            }
            _ => {}
        }
    }

    fn name(&self) -> &str { "CombatUIHandler" }
    fn priority(&self) -> Priority { Priority::Low }
}

// sound/src/combat_sound_handler.rs
pub struct CombatSoundHandler {
    audio: Arc<Mutex<SoundSystem>>,
}

impl EventHandler for CombatSoundHandler {
    fn handle(&mut self, event: &GameEvent) {
        let mut audio = self.audio.lock().unwrap();

        match event {
            GameEvent::DamageDealt { .. } => {
                audio.play_hit_sound();
            }
            GameEvent::EntityDied { .. } => {
                audio.play_death_sound();
            }
            _ => {}
        }
    }

    fn name(&self) -> &str { "CombatSoundHandler" }
    fn priority(&self) -> Priority { Priority::Low }
}
```

### 案例 2：物品系统

#### 迁移前
```rust
// items/src/inventory.rs
impl Inventory {
    pub fn use_item(
        &mut self,
        item_id: usize,
        player: &mut Player,
        ui: &mut UI,
    ) -> Result<()> {
        let item = self.items.get(item_id)?;

        match item.effect {
            ItemEffect::Healing(amount) => {
                player.heal(amount);
                ui.show_message(&format!("恢复了 {} 点生命", amount));
            }
            ItemEffect::Buff(stat, amount) => {
                player.apply_buff(stat, amount);
                ui.show_message(&format!("获得了 {} 增益", stat));
            }
        }

        self.items.remove(item_id);
        Ok(())
    }
}
```

#### 迁移后
```rust
// items/src/inventory.rs
impl Inventory {
    pub fn use_item(
        &mut self,
        event_bus: &mut EventBus,
        item_id: usize,
        player_id: u32,
    ) -> Result<()> {
        let item = self.items.get(item_id)?;

        // 发布物品使用事件
        event_bus.publish(GameEvent::ItemUsed {
            entity: player_id,
            item_name: item.name.clone(),
            effect: format!("{:?}", item.effect),
        });

        // 根据效果发布具体事件
        match item.effect {
            ItemEffect::Healing(amount) => {
                event_bus.publish(GameEvent::HealthChanged {
                    entity: player_id,
                    old_hp: self.current_hp,
                    new_hp: self.current_hp + amount,
                    max_hp: self.max_hp,
                });
            }
            ItemEffect::Buff(stat, amount) => {
                event_bus.publish(GameEvent::StatusApplied {
                    entity: player_id,
                    status: format!("{}增益", stat),
                    duration: 10,
                });
            }
        }

        self.items.remove(item_id);
        Ok(())
    }
}
```

## 常见模式

### 模式 1：状态更新 + 事件发布

```rust
// 先更新状态
entity.hp -= damage;

// 再发布事件
event_bus.publish(GameEvent::HealthChanged { ... });
```

### 模式 2：使用延迟事件避免借用冲突

```rust
// 收集需要延迟处理的事件
let mut dead_entities = Vec::new();

for entity in entities {
    if entity.hp <= 0 {
        dead_entities.push(entity.id);
    }
}

// 在循环外发布延迟事件
for entity_id in dead_entities {
    event_bus.publish_delayed(GameEvent::EntityDied { ... });
}
```

### 模式 3：链式事件

```rust
// 第一个事件触发第二个事件
impl EventHandler for ChainHandler {
    fn handle(&mut self, event: &GameEvent) {
        match event {
            GameEvent::EntityDied { entity, .. } => {
                // 触发掉落事件
                self.trigger_loot_drop(*entity);
            }
            _ => {}
        }
    }
}
```

## 迁移检查清单

- [ ] 识别所有模块间的直接调用
- [ ] 为每个调用定义对应的事件类型
- [ ] 创建事件处理器替代直接调用
- [ ] 替换代码中的直接调用为事件发布
- [ ] 在初始化时注册所有处理器
- [ ] 编写测试验证功能正确
- [ ] 更新文档说明新的事件流程
- [ ] 删除旧的耦合代码

## 迁移优先级

建议按以下顺序迁移：

1. **高优先级**（核心游戏逻辑）
   - 战斗系统
   - 移动系统
   - 物品系统

2. **中优先级**（游戏功能）
   - AI 系统
   - 地牢生成
   - 状态效果

3. **低优先级**（外围系统）
   - UI 更新
   - 音效播放
   - 统计收集

## 性能考虑

### 事件发布开销

```rust
// 低开销：O(1)
event_bus.publish(event);

// 中等开销：O(n) n=订阅者数量
// 订阅者会立即处理事件

// 几乎无开销：只是添加到队列
event_bus.publish_delayed(event);
```

### 优化建议

1. **批量处理**
```rust
// ❌ 避免在循环中频繁发布
for entity in entities {
    event_bus.publish(GameEvent::EntityUpdated { ... });
}

// ✅ 批量处理或只在关键时刻发布
if important_change {
    event_bus.publish(GameEvent::BatchUpdated { count: entities.len() });
}
```

2. **使用事件过滤**
```rust
impl EventHandler for MyHandler {
    fn should_handle(&self, event: &GameEvent) -> bool {
        // 只处理感兴趣的事件
        matches!(event, GameEvent::DamageDealt { .. })
    }
}
```

## 调试技巧

### 1. 启用事件日志
```rust
#[cfg(debug_assertions)]
{
    let debugger = EventDebugger::new(true);
    event_bus.subscribe_all(Box::new(debugger));
}
```

### 2. 追踪事件流
```rust
let history = event_bus.get_history(20);
for event in history {
    println!("{:?}", event);
}
```

### 3. 统计事件
```rust
let stats = EventStatistics::new();
event_bus.subscribe_all(Box::new(stats));

// 稍后查看
println!("伤害事件: {}", stats.get_count("DamageDealt"));
```

## 故障排查

### 问题：事件没有被处理

**可能原因**：
1. 处理器没有注册
2. 事件类型不匹配
3. `should_handle()` 返回 false

**解决方法**：
```rust
// 检查订阅者数量
println!("订阅者: {}", event_bus.subscriber_count());

// 检查历史记录
println!("最近事件: {:?}", event_bus.get_history(5));
```

### 问题：借用冲突

**解决方法**：使用延迟事件
```rust
// ❌ 可能导致借用冲突
event_bus.publish(GameEvent::EntityDied { ... });

// ✅ 延迟到下一帧处理
event_bus.publish_delayed(GameEvent::EntityDied { ... });
```

## 总结

迁移到事件驱动架构的关键步骤：

1. **识别** - 找出所有模块间的直接依赖
2. **定义** - 为每个交互定义事件
3. **创建** - 实现事件处理器
4. **替换** - 用事件发布替代直接调用
5. **测试** - 确保功能正确
6. **优化** - 根据性能需求调整

**好处**：
- ✅ 模块解耦
- ✅ 易于测试
- ✅ 灵活扩展
- ✅ 维护简单

继续阅读：
- 完整指南：`EVENT_BUS_GUIDE.md`
- 示例代码：`examples/event_handlers.rs`
- 快速入门：`EVENT_BUS_QUICKSTART.md`
