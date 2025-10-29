# 回合系统架构

本文档描述了为终端像素地牢提供动力的能量驱动回合架构。它解释了状态机、阶段管道和事件总线如何协同调度动作、何时消耗或再生能量，以及如何在不破坏存档兼容性的情况下扩展系统。

> **交叉引用**：高级 ECS + 模块划分在[项目 README](../README.md) 中有介绍。事件总线使用模式在 [EVENT_BUS_GUIDE.md](EVENT_BUS_GUIDE.md) 中详述。此处引用的 UI 功能在 [UI_IMPROVEMENTS.md](../UI_IMPROVEMENTS.md) 中引入。

## 状态机概览

`TurnSystem` 公开了一个最小的状态机，在玩家和 AI 角色之间交替控制，同时允许中间处理状态以供未来扩展：

```
┌────────────┐      玩家行动         ┌────────────────────┐
│ PlayerTurn │ ─────────────────────▶ │ ProcessingPlayer…  │
└────────────┘                        └────────────────────┘
      ▲                                         │
      │                                         │完成 + 能量消耗
      │      AI 获得控制                         ▼
┌────────────┐      并解析          ┌────────────────────┐
│  AITurn    │ ◀───────────────────── │ ProcessingAI…      │
└────────────┘                        └────────────────────┘
```

* **`PlayerTurn`** – 游戏等待输入并运行面向玩家的系统（移动、战斗解算等）。
* **`ProcessingPlayerAction`** – 为多步骤玩家动作保留（当前未使用但保留用于确定性重播支持）。
* **`AITurn`** – AI 控制器消耗能量并重复行动，直到玩家恢复满能量。
* **`ProcessingAIActions`** – 为脚本化多动作 AI 行为保留。

虽然目前只有 `PlayerTurn` 和 `AITurn` 处于活动状态，但记录完整的状态图可以明确将来在何处附加钩子。

## 阶段管道

在每一帧期间，`GameLoop` 以确定性顺序执行系统。该顺序同时作为动作优先级列表——早期系统可以在后期系统检查相同输入缓冲区之前入队或解析动作。

| 顺序 | 阶段/系统                  | 职责 |
|-----:|--------------------------|------|
| 1    | `InputSystem`            | 轮询设备并将 `PlayerAction` 推送到 `pending_actions` 队列。 |
| 2    | `MenuSystem`             | 立即消费菜单/导航动作（它们从不消耗能量）。 |
| 3    | `TimeSystem`             | 推进全局 `turn_count` 并为每个 `Energy` 组件再生基线能量。 |
| 4    | `MovementSystem`         | 解析逐步移动；成功的移动被复制到 `completed_actions`，以便能量可以被扣除一次。 |
| 5    | `AISystem`               | 基于世界快照规划 AI 角色意图。 |
| 6    | `CombatSystem::run_with_events` | 将攻击动作转换为战斗事件，向总线发出伤害/状态事件。 |
| 7    | `FOVSystem`              | 为本帧位置改变的实体重建视野。 |
| 8    | `EffectSystem`           | 触发激活的状态效果并可能入队次要动作。 |
| 9    | `EnergySystem`           | 保留用于向后兼容性，但在回合调度器显式管理能量时跳过。 |
| 10   | `InventorySystem`        | 应用排队的库存交互（使用/丢弃）。 |
| 11   | `HungerSystem::run_with_events` | 消耗饱食度并发出饥饿/饥饿警告。 |
| 12   | `DungeonSystem`          | 处理关卡转换、陷阱触发等。 |
| 13   | `RenderingSystem`        | 产生 UI 帧，包括 `UI_IMPROVEMENTS.md` 中记录的饥饿和状态指示器。

在所有系统运行之后，游戏循环刷新事件总线（`process_events`），通过 `TurnSystem::process_turn_cycle` 推进回合状态，将任何 `GameStatus` 更改桥接到事件，再次处理剩余事件，最后通过 `next_frame()` 交换事件缓冲区。

## 能量调度

### 能量再生
每一帧，`TimeSystem` 都会为每个 `Energy` 组件添加其 `regeneration_rate`（通常为 1）。这确保了实体即使在等待时也能积累能量。

### 能量消耗
在管道结束时，如果玩家或 AI 执行了消耗能量的动作（移动、攻击、使用物品），`TurnSystem::process_turn_cycle` 会检查 `completed_actions` 并扣除固定数量（通常为 100 能量）。

### 回合切换
当玩家的能量降至零以下时，控制权转移到 `AITurn`。AI 角色依次行动（如果它们的能量允许）。一旦所有 AI 都用完能量或玩家恢复到满能量，循环就会回到 `PlayerTurn`。

## 扩展回合系统

### 添加新动作
1. 在 `ecs.rs` 的 `PlayerAction` 枚举中定义新的动作变体。
2. 在相关系统（例如 `MovementSystem`、`InventorySystem`）中处理它。
3. 如果动作成功，将其推送到 `completed_actions`，以便 `TurnSystem` 扣除能量。
4. 可选地发出一个 `GameEvent` 以通知其他系统。

### 添加新效果
1. 在 `status_effect.rs` 或 `effect.rs` 中定义新的效果类型。
2. 在 `EffectSystem` 中实现其逐帧逻辑。
3. 发出相应的事件（例如 `StatusApplied`、`StatusEffectTicked`）。

### 保存兼容性
回合状态（`TurnState`）是可序列化的。添加新字段时：
- 使用 `#[serde(default)]` 为旧存档提供合理的默认值。
- 在 `SaveData` 中记录版本号，如果需要迁移逻辑。

## 调试和测试

- **日志**：事件总线可以配置为将所有事件记录到控制台或文件。
- **确定性**：给定相同的 RNG 种子，管道应该产生相同的结果。
- **单元测试**：每个系统都可以独立测试，使用模拟 `World` 和 `Resources`。

## 未来改进

- **多步骤动作**：使用 `ProcessingPlayerAction` 和 `ProcessingAIActions` 状态进行复杂的脚本序列。
- **暂停/重播**：使用事件日志和种子重播游戏。
- **优先级队列**：允许更快的实体更频繁地行动（例如，急速效果）。

---

有关实现细节，请参阅：
- `src/turn_system.rs` - 回合状态机实现
- `src/game_loop.rs` - 阶段管道协调
- `src/systems.rs` - 各个系统实现
- `docs/EVENT_BUS_GUIDE.md` - 事件总线使用指南
