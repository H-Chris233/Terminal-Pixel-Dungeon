[根目录](../../CLAUDE.md) > **achievements**

# Achievements 成就模块

## 模块职责

成就系统模块追踪玩家的游戏进度并在满足条件时解锁成就：
- 击杀类成就（首杀、屠戮者等）
- 探索类成就（深层探索等）
- 收集类成就（囤积者等）
- 生存类成就（生存者等）
- 杂项成就（幸运、财富）
- 与事件总线集成，自动追踪进度

## 入口与启动

- **入口文件**: `src/achievements/src/lib.rs`
- 核心结构体: `AchievementsManager`

## 对外接口

### AchievementsManager
- `new()` - 创建管理器（含所有默认成就）
- `on_kill()` / `on_level_change()` / `on_item_pickup()` / `on_turn_end()` / `on_boss_defeat()` / `on_gold_collected()` - 事件处理
- `check_and_unlock()` - 检查并解锁成就
- `is_unlocked(id)` - 检查成就是否解锁
- `unlocked_achievements()` - 获取已解锁成就列表
- `drain_newly_unlocked()` - 获取新解锁成就
- `unlock_percentage()` - 解锁百分比

## 关键依赖与配置

- **Cargo.toml**: `src/achievements/Cargo.toml`
- 依赖: `serde`, `bincode`
- 纯数据模块，无运行时依赖

## 数据模型

### AchievementId
`FirstBlood`, `SlayerI`, `SlayerII`, `SlayerIII`, `BossSlayer`, `DeepDiver`, `Spelunker`, `MasterExplorer`, `Hoarder`, `Collector`, `TreasureHunter`, `Survivor`, `Veteran`, `Legend`, `Lucky`, `Wealthy`

### AchievementCriteria
`KillCount(u32)`, `ReachDepth(usize)`, `CollectItems(u32)`, `SurviveTurns(u32)`, `DefeatBoss`, `CollectGold(u32)`, `FindRareItem`

### AchievementProgress
追踪: kills, depth, items_collected, turns_survived, bosses_defeated, gold_collected

## 测试与质量

- **测试文件**: `src/achievements/src/tests.rs`
- 测试内容: 成就解锁逻辑、进度追踪

## 常见问题

### 事件总线集成
- 成就系统通过 `ECSWorld::handle_achievement_event()` 自动追踪
- 在 `src/ecs.rs` 中映射 `GameEvent` 到成就事件

## 相关文件清单

| 文件 | 用途 |
|------|------|
| `src/achievements/src/lib.rs` | 模块入口、AchievementsManager |
| `src/achievements/src/achievement.rs` | 成就定义和类型 |
| `src/achievements/src/criteria.rs` | 条件和进度追踪 |
| `src/achievements/src/tests.rs` | 测试 |
| `src/achievements/examples/basic_usage.rs` | 使用示例 |

## 变更记录 (Changelog)

### 2026-05-28
- 初始化模块文档
