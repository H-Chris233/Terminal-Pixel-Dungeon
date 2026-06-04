[根目录](../../CLAUDE.md) > **core**

# Core 核心模块

## 模块职责

核心模块提供游戏引擎的统一接口和工具：
- `GameEngine` - 核心游戏引擎，整合 ECS World
- `EntityFactory` - 创建游戏实体（玩家、敌人、物品）
- `GameState` - 游戏状态管理

## 入口与启动

- **入口文件**: `src/core.rs` + `src/core/entity_factory.rs` + `src/core/game_state.rs`
- 属于主 crate 的一部分（无独立 Cargo.toml）

## 对外接口

### GameEngine
- `src/core.rs` - 核心引擎
- 包含: `world: Arc<Mutex<World>>`, `game_state: GameState`, `entity_factory: EntityFactory`
- `new()` - 创建引擎
- `update()` - 更新游戏状态

### EntityFactory
- `src/core/entity_factory.rs` - 实体创建工厂
- `create_player(world, x, y, class)` - 创建基于职业的玩家
- `create_monster(world, x, y, monster_type)` - 创建怪物

### GameState
- `src/core/game_state.rs` - 游戏状态结构体

## 相关文件清单

| 文件 | 用途 |
|------|------|
| `src/core.rs` | 模块入口、GameEngine 结构体 |
| `src/core/entity_factory.rs` | 实体工厂 |
| `src/core/game_state.rs` | 游戏状态 |

## 变更记录 (Changelog)

### 2026-05-28
- 初始化模块文档
