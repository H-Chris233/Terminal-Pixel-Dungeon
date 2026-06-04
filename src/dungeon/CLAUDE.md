[根目录](../../CLAUDE.md) > **dungeon**

# Dungeon 地牢模块

## 模块职责

地牢模块负责游戏地牢的生成和管理：
- 多层地牢生成算法
- 房间布局（常规房间、Boss 房间）
- 走廊连接
- 陷阱系统
- 地图瓦片管理（墙壁、地板、水、门等）

## 入口与启动

- **入口文件**: `src/dungeon/src/lib.rs`
- 主结构体: `Dungeon` - 管理所有地牢层级
- 生成方法: `Dungeon::generate(max_depth, seed)` - 生成完整地牢

## 对外接口

### Dungeon 结构体
- `current_level()` / `current_level_mut()` - 获取当前层
- `is_passable(x, y)` - 路径检测
- `can_descend(x, y)` / `can_ascend(x, y)` - 楼梯检测
- `on_hero_enter(x, y)` - 统一处理英雄进入格子的交互
- `get_tile(x, y)` - 获取瓦片信息
- `update_visibility(x, y, radius)` - 更新视野

### Level
- `src/dungeon/src/level.rs` - 单层地牢的结构和生成逻辑
- 包含: 房间、走廊、敌人、物品、陷阱的生成

### BossRoom
- `src/dungeon/src/boss_room.rs` - Boss 房间布局

### Trap
- `src/dungeon/src/trap.rs` - 陷阱效果定义和触发逻辑

### TerrainType / TileInfo
- `src/dungeon/src/level/tiles.rs` - 瓦片类型和信息

## 关键依赖与配置

- **Cargo.toml**: `src/dungeon/Cargo.toml`
- 依赖: `combat`, `items`, `rand`, `rand_pcg`, `serde`, `bincode`

## 数据模型

### Dungeon
```rust
pub struct Dungeon {
    pub depth: usize,       // 当前深度
    pub levels: Vec<Level>, // 所有层级
    pub seed: u64,          // 随机种子
    pub max_depth: usize,   // 最大深度
}
```

### InteractionEvent
`TrapTriggered`, `ItemFound`, `EnemyEncounter`, `StairsUp`, `StairsDown`, `BossEncounter`, `BossRoomEntered`, `HazardDamage`

### TileInteraction
`has_trap`, `has_item`, `is_stair`, `is_door`

## 测试与质量

- **测试文件**: 无专用测试文件
- 注意: 需要补充单元测试

## 常见问题

### 地牢生成
- 每隔 5 层生成 Boss 层（`depth % 5 == 0`）
- 使用 `rand_pcg` 确保可复现性
- 地牢通过 `seed` 完全确定

### 地牢与 ECS 集成
- 地牢通过 `DungeonComponent` 组件附加到 ECS 实体
- 使用 `get_dungeon_clone()` 和 `set_dungeon_instance()` 辅助函数

## 相关文件清单

| 文件 | 用途 |
|------|------|
| `src/dungeon/src/lib.rs` | 模块入口、Dungeon 结构体 |
| `src/dungeon/src/level.rs` | 层级生成 |
| `src/dungeon/src/level/rooms.rs` | 房间布局 |
| `src/dungeon/src/level/tiles.rs` | 瓦片类型 |
| `src/dungeon/src/boss_room.rs` | Boss 房间 |
| `src/dungeon/src/trap.rs` | 陷阱系统 |

## 变更记录 (Changelog)

### 2026-05-28
- 初始化模块文档
