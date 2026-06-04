[根目录](../../CLAUDE.md) > **hero**

# Hero 英雄模块

## 模块职责

英雄模块管理玩家角色相关的所有功能：
- 角色类别系统（战士、盗贼、法师、猎手）
- 职业技能与冷却
- 背包系统（库存和装备）
- 物品使用与装备逻辑
- 效果管理
- 战斗集成（实现 Combatant trait）

## 入口与启动

- **入口文件**: `src/hero/src/lib.rs`
- 主要导出: `Hero`, `Bag`, `HeroBehavior` trait, `EffectManager`
- 创建方式: `Hero::with_seed(class, seed)`

## 对外接口

### Hero 结构体
- `src/hero/src/core.rs` - 核心数据结构
- 属性: hp, max_hp, base_attack, base_defense, experience, level, strength, satiety, gold
- 子系统: bag (背包), effects (效果), rng (随机数), class_skills (技能)

### HeroBehavior trait
- `new(class)`, `with_seed(class, seed)`
- `on_turn()` - 每回合更新
- `move_to(dx, dy, dungeon)` - 移动交互
- `gain_exp(exp)` - 获取经验

### Class 系统
- `src/hero/src/class.rs` - 职业定义（Warrior, Mage, Rogue, Huntress）
- `src/hero/src/class/warrior.rs` - 战士技能
- `src/hero/src/class/mage.rs` - 法师技能
- `src/hero/src/class/rogue.rs` - 盗贼技能
- `src/hero/src/class/huntress.rs` - 猎手技能

### Bag 系统
- `src/hero/src/bag.rs` - 背包包装器
- `src/hero/src/bag/inventory.rs` - 库存管理
- `src/hero/src/bag/equipment.rs` - 装备管理

### Item 系统
- `src/hero/src/core/item.rs` - 物品操作
- `src/hero/src/core/item/equip.rs` - 装备操作
- `src/hero/src/core/item/use.rs` - 使用操作

### Event 系统
- `src/hero/src/core/events.rs` - `ActionResult`, `HeroEvent` 定义

## 关键依赖与配置

- **Cargo.toml**: `src/hero/Cargo.toml`
- 依赖: `combat`, `dungeon`, `items`, `rand`, `rand_pcg`, `serde`, `bincode`, `strum`, `thiserror`

## 数据模型

### Hero
```rust
pub struct Hero {
    pub class: Class,
    pub name: String,
    pub hp: u32, pub max_hp: u32,
    pub base_attack: u32, pub base_defense: u32,
    pub experience: u32, pub level: u32,
    pub strength: u8, pub satiety: u8,
    pub gold: u32, pub x: i32, pub y: i32,
    pub alive: bool, pub turns: u32,
    pub effects: EffectManager,
    pub rng: HeroRng,
    pub bag: Bag,
    pub class_skills: SkillState,
    pub entity_id: Option<u32>,
}
```

### SkillState
`unlocked_talents`, `active_skill`, `cooldowns`, `charges`

## 测试与质量

- **测试文件**: 无专用测试文件
- 注意: 需要在模块级别添加单元测试

## 常见问题

### 借用检查
背包系统和装备系统存在复杂的借用关系：
- 使用 `Bag` 包装 `Inventory` 和 `Equipment`
- 装备时需要分离可变借用和不可变借用
- 参考 `src/hero/src/core/item/equip.rs` 实现

### 适配器桥接
- `hero_adapter.rs` 将 `Hero` 结构转换为 ECS 组件
- ECS 中的 `Hunger`, `Wealth`, `PlayerProgress` 组件从 `Hero` 转换而来

## 相关文件清单

| 文件 | 用途 |
|------|------|
| `src/hero/src/lib.rs` | 模块入口、traits |
| `src/hero/src/core.rs` | Hero 结构体 |
| `src/hero/src/core/events.rs` | 事件定义 |
| `src/hero/src/core/item.rs` | 物品操作 |
| `src/hero/src/core/item/equip.rs` | 装备逻辑 |
| `src/hero/src/core/item/use.rs` | 使用逻辑 |
| `src/hero/src/class.rs` | 职业定义 |
| `src/hero/src/class/warrior.rs` | 战士 |
| `src/hero/src/class/mage.rs` | 法师 |
| `src/hero/src/class/rogue.rs` | 盗贼 |
| `src/hero/src/class/huntress.rs` | 猎手 |
| `src/hero/src/bag.rs` | 背包 |
| `src/hero/src/bag/inventory.rs` | 库存 |
| `src/hero/src/bag/equipment.rs` | 装备 |
| `src/hero/src/combat.rs` | 战斗集成 |
| `src/hero/src/effects.rs` | 效果管理 |
| `src/hero/src/abilities.rs` | 技能 |
| `src/hero/src/rng.rs` | 随机数 |

## 变更记录 (Changelog)

### 2026-05-28
- 初始化模块文档
