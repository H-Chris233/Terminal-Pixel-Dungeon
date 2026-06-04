[根目录](../../CLAUDE.md) > **error**

# Error 错误处理模块

## 模块职责

错误处理模块定义了游戏中所有可能的错误类型和错误处理工具：
- `GameError` 枚举（基于 `thiserror`）
- 错误到用户友好消息的转换
- `BagError` 背包系统错误

## 入口与启动

- **入口文件**: `src/error/src/lib.rs`
- 主要导出: `GameError`, `BagError`, `handle_error()`

## 对外接口

### GameError 枚举
- `SaveError(anyhow::Error)` - 存档系统错误
- `IoError(std::io::Error)` - IO 操作错误
- `SerializationError(String)` / `DeserializationError(String)` - 序列化错误
- `InvalidSlot` - 无效存档槽位
- `CorruptedSave` - 存档数据损坏
- `VersionMismatch(String)` - 版本不兼容
- `InvalidHeroData` / `InvalidLevelData` / `InvalidItemData` / `InvalidMobData` - 数据验证错误
- `InvalidGameState` - 游戏状态无效
- `InputError(String)` - 用户输入错误

### BagError
- `Full` - 背包已满
- `ItemNotFound` - 物品未找到
- `EquipmentConflict` - 装备冲突

### 辅助函数
- `handle_error(error: &GameError) -> String` - 转换为用户友好消息

## 关键依赖与配置

- **Cargo.toml**: `src/error/Cargo.toml`
- 依赖: `anyhow`, `bincode`, `thiserror`

## 测试与质量

- **测试文件**: 无专用测试文件
- 注意: 需要补充单元测试

## 变更记录 (Changelog)

### 2026-05-28
- 初始化模块文档
