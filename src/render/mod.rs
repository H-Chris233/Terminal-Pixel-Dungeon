//! 模块化渲染系统
//!
//! 将渲染逻辑分解为独立的、可测试的组件：
//! - `dungeon` - 地牢地图渲染（使用 ECS 的 FOV 数据）
//! - `hud` - 玩家状态 HUD 渲染
//! - `inventory` - 物品栏渲染
//! - `menu` - 菜单和界面渲染
//! - `game_over` - 游戏结束界面渲染
//!
//! 所有渲染器直接操作 ECS World 和 Resources，确保架构统一。

pub mod dungeon;
pub mod game_over;
pub mod hud;
pub mod inventory;
pub mod menu;

pub use dungeon::DungeonRenderer;
pub use game_over::GameOverRenderer;
pub use hud::HudRenderer;
pub use inventory::InventoryRenderer;
pub use menu::MenuRenderer;
