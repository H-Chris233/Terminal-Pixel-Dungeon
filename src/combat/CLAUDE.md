[根目录](../../CLAUDE.md) > **combat**

# Combat 战斗模块

## 模块职责

战斗模块是游戏核心机制之一，负责所有战斗相关的逻辑：
- 战斗解算（命中判定、伤害计算）
- 暴击、潜行攻击机制
- 状态效果系统（15+ 种效果）
- AI 敌人行为（Boss 战）
- 视野系统（FOV 算法）

## 入口与启动

- **入口文件**: `src/combat/src/lib.rs`
- 导出 `Combat` 结构体作为主要 API，提供 `engage()`、`resolve_attack()` 等方法
- 使用 `Combatant` trait 抽象攻击者和防御者

## 对外接口

### Combatant trait
- `src/combat/src/combatant.rs` - 定义战斗者接口：
  - `attack_power()`, `defense()`, `accuracy()`, `evasion()`
  - `crit_bonus()`, `take_damage()`
  - `is_alive()`, `exp_value()`, `name()`, `id()`

### Combat 结构体
- `src/combat/src/lib.rs` - 主要战斗 API：
  - `Combat::engage()` - 两个战斗者之间的完整战斗（攻击+反击）
  - `Combat::perform_attack_with_ambush()` - 考虑潜行的攻击
  - `Combat::calculate_hit_chance()` - 命中计算
  - `Combat::calculate_damage()` - 伤害计算
  - `Combat::is_critical()` - 暴击判定

### VisionSystem
- `src/combat/src/vision.rs` - 视野和潜行检测

### Boss 系统
- `src/combat/src/boss.rs` - Boss 类型、阶段、技能、战利品

### Status Effect 系统
- `src/combat/src/status_effect.rs` - 状态效果定义和逻辑
- `src/combat/src/effect.rs` - 效果类型和枚举

## 关键依赖与配置

- **Cargo.toml**: `src/combat/Cargo.toml`
- 依赖: `items`, `rand`, `serde`, `bincode`, `strum`
- 注意: `combat` 模块依赖 `items` 模块

## 数据模型

### 核心常量
```rust
BASE_HIT_CHANCE = 0.8   // 基础命中率 80%
CRIT_MULTIPLIER = 1.5    // 暴击 1.5 倍
SURPRISE_ATTACK_MODIFIER = 2.0  // 潜行 2 倍
MIN_DAMAGE = 1
DEFENSE_CAP = 0.8        // 最多减伤 80%
```

### CombatResult
`logs`, `defeated`, `experience`, `events` (CombatEvent 列表)

### CombatEvent
`CombatStarted`, `DamageDealt`, `EntityDied`, `Ambush`

### Enemy
定义敌人的属性、掉落、AI 行为

### Boss
`BossType` 枚举、`BossPhase`、`BossSkill`、`SkillCooldowns`

## 测试与质量

- **测试文件**: `src/combat/src/tests.rs`
- 测试范围: 战斗命中、伤害计算、潜行攻击、Boss 战斗
- 运行: `cargo test -p combat`

## 常见问题

### 战斗平衡
- 所有数值模仿 Shattered Pixel Dungeon
- 修改常量需同时更新文档中的公式说明

### 模块间通信
- 战斗模块通过 `CombatEvent` 枚举发布事件
- 事件最终由 `game_loop.rs` 中的系统桥接到 `event_bus.rs`

## 相关文件清单

| 文件 | 用途 |
|------|------|
| `src/combat/src/lib.rs` | 模块入口、Combat 结构体 |
| `src/combat/src/combatant.rs` | Combatant trait |
| `src/combat/src/combat_manager.rs` | 战斗管理器 |
| `src/combat/src/vision.rs` | 视野系统 |
| `src/combat/src/effect.rs` | 效果类型 |
| `src/combat/src/status_effect.rs` | 状态效果 |
| `src/combat/src/enemy.rs` | 敌人定义 |
| `src/combat/src/boss.rs` | Boss 系统 |
| `src/combat/src/tests.rs` | 测试 |

## 变更记录 (Changelog)

### 2026-05-28
- 初始化模块文档
