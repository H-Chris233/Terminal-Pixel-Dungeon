[根目录](../../CLAUDE.md) > **render**

# Render 渲染模块

## 模块职责

渲染模块负责游戏的所有终端 UI 渲染，基于 `ratatui` 库实现：
- 地牢地图渲染（使用 ECS 的 FOV 数据）
- 玩家状态 HUD 渲染
- 物品栏渲染
- 菜单界面渲染（主菜单、暂停、选项）
- Boss 战斗 UI 渲染
- 职业选择界面渲染
- 游戏结束界面渲染

## 入口与启动

- **入口文件**: `src/render/mod.rs`
- 属于主 crate 的一部分（无独立 Cargo.toml）
- 所有渲染器直接操作 ECS World 和 Resources

## 对外接口

### DungeonRenderer
- `src/render/dungeon.rs` - 地牢地图的渲染逻辑
- 使用 ECS 中的 Position, Tile, Renderable, Viewshed 组件

### HudRenderer
- `src/render/hud.rs` - 玩家 HUD（生命、饱食度、层数等）

### InventoryRenderer
- `src/render/inventory.rs` - 物品栏界面

### MenuRenderer
- `src/render/menu.rs` - 主菜单、暂停菜单、选项

### BossUI
- `src/render/boss.rs` - Boss 战斗 UI

### ClassSelectionRenderer
- `src/render/class_selection.rs` - 职业选择界面

### GameOverRenderer
- `src/render/game_over.rs` - 游戏结束界面

## 数据模型

- 使用 ECS 的 `Renderable` 组件（symbol, fg_color, bg_color, order）
- 使用 `GameState.game_state` 确定当前渲染模式
- 使用 `GameStatus` 枚举切换不同界面

## 测试与质量

- **测试文件**: 无专用测试文件
- 注意: 需要补充渲染模块的测试

## 常见问题

### 渲染架构
- 渲染是游戏循环的最后一个阶段（`SystemPhase::Render`）
- 在菜单/暂停状态下，只运行输入和渲染阶段
- 所有渲染器共享 ECS World 和 Resources

## 相关文件清单

| 文件 | 用途 |
|------|------|
| `src/render/mod.rs` | 模块入口 |
| `src/render/dungeon.rs` | 地牢渲染 |
| `src/render/hud.rs` | HUD 渲染 |
| `src/render/inventory.rs` | 物品栏渲染 |
| `src/render/menu.rs` | 菜单渲染 |
| `src/render/boss.rs` | Boss UI |
| `src/render/class_selection.rs` | 职业选择 |
| `src/render/game_over.rs` | 游戏结束 |

## 变更记录 (Changelog)

### 2026-05-28
- 初始化模块文档
