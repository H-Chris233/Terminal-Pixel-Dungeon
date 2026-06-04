[根目录](../../CLAUDE.md) > **save**

# Save 保存模块

## 模块职责

保存模块负责游戏存档的序列化、读写和管理：
- 使用 `bincode` 二进制序列化
- 存档版本管理（当前版本 2）
- 自动保存功能（默认 5 分钟间隔）
- 多槽位存档支持（最多 10 个）

## 入口与启动

- **入口文件**: `src/save/src/lib.rs`
- 核心结构体: `SaveSystem`, `AutoSave`, `SaveData`

## 对外接口

### SaveSystem
- `new(save_dir, max_slots)` - 初始化存档系统
- `save_game(slot, data)` - 保存游戏到指定槽位
- `load_game(slot)` - 从指定槽位加载游戏
- `delete_save(slot)` - 删除存档
- `list_saves()` - 列出所有存档
- `has_save(slot)` - 检查槽位是否有存档

### AutoSave
- `new(save_system, interval)` - 自动保存系统
- `check_auto_save(game_data)` - 检查并触发自动保存
- `force_save(save_data)` - 强制立即保存
- `set_save_interval(interval)` - 设置保存间隔

### SaveData
- `migrate()` - 迁移旧版本存档到当前版本
- `validate()` - 验证存档数据完整性

## 数据模型

### SaveData
```rust
pub struct SaveData {
    pub version: u32,           // 存档版本
    pub metadata: SaveMetadata, // 元数据
    pub hero_skill_state: SkillState,
    pub hero: hero::Hero,       // 英雄数据
    pub dungeon: dungeon::Dungeon, // 地牢数据
    pub game_seed: u64,         // 随机种子
    pub turn_state: TurnStateData,  // 回合状态
    pub clock_state: ClockStateData, // 时钟状态
    pub player_energy: u32,     // 玩家能量
    pub player_hunger_last_turn: u32, // 饥饿状态
    pub entities: Vec<EntityStateData>, // 实体状态
}
```

### SaveMetadata
`timestamp`, `dungeon_depth`, `hero_name`, `hero_class`, `play_time`

## 测试与质量

- **测试文件**: `src/save/src/lib.rs` 中的内联测试
- 测试内容: 存档序列化/反序列化往返、职业和技能状态保持

## 常见问题

### 版本迁移
- 当前版本: 2
- v1 到 v2: 初始化回合状态和时钟状态

### 序列化
- 使用 `bincode` 的 `config::standard()` 配置
- 文件扩展名: `.sav`
- 写入使用原子性重命名（临时文件 + rename）

## 相关文件清单

| 文件 | 用途 |
|------|------|
| `src/save/src/lib.rs` | 模块入口、所有存档逻辑 |

## 变更记录 (Changelog)

### 2026-05-28
- 初始化模块文档
