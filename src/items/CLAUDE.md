[根目录](../../CLAUDE.md) > **items**

# Items 物品模块

## 模块职责

物品模块实现了游戏中的所有物品类型和机制，完全模仿 Shattered Pixel Dungeon 的物品系统：
- 近战武器（Tier 1-5，含特殊武器）
- 护甲（含护甲 Glyph）
- 药水（12 种）
- 卷轴（10 种）
- 法杖（8 种）
- 戒指（10 种）
- 种子（8 种）
- 魔法石（6 种）
- 食物（3 种）
- 投掷武器
- 药草
- 杂项物品

## 入口与启动

- **入口文件**: `src/items/src/lib.rs`
- 核心结构体: `Item`, `ItemKind`
- 核心 trait: `ItemTrait` - 所有物品需实现的接口

## 对外接口

### Item 结构体
- `Item::new(kind)` - 创建物品
- `Item::name()` - 获取显示名称
- `Item::is_consumable()` - 是否为消耗品
- `Item::needs_identify()` - 是否需要鉴定
- `Item::value()` - 物品价值

### ItemKind 枚举
- `Weapon(Weapon)`, `Armor(Armor)`, `Potion(Potion)`, `Scroll(Scroll)`
- `Food(Food)`, `Wand(Wand)`, `Ring(Ring)`, `Seed(Seed)`
- `Stone(Stone)`, `Misc(MiscItem)`, `Throwable(Throwable)`, `Herb(Herb)`

### Weapon 系统
- `src/items/src/weapon.rs` - 武器基类
- `src/items/src/weapon/kind.rs` - 武器种类
- `src/items/src/weapon/tier.rs` - 武器等级
- `src/items/src/weapon/tier/one.rs` ~ `five.rs` - 各等级武器
- `src/items/src/weapon/tier/two/dagger.rs` - 匕首
- `src/items/src/weapon/tier/two/spear.rs` - 长矛
- `src/items/src/weapon/tier/five/greataxe.rs` - 巨斧
- `src/items/src/weapon/tier/five/greatsword.rs` - 巨剑

## 关键依赖与配置

- **Cargo.toml**: `src/items/Cargo.toml`
- 依赖: `rand`, `serde`, `bincode`, `ratatui`, `strum`, `seahash`

## 数据模型

### Item
```rust
pub struct Item {
    pub kind: ItemKind,
    pub name: String,
    pub description: String,
    pub quantity: u32,   // 堆叠数量
    pub x: i32, pub y: i32,
}
```

### ItemCategory
`Weapon`, `Armor`, `Potion`, `Scroll`, `Wand`, `Ring`, `Seed`, `Stone`, `Throwable`, `Herb`, `Food`, `Misc`

### ItemRarity
`Common`, `Rare`, `Epic`, `Legendary`

## 测试与质量

- **测试文件**: 无专用测试文件
- 注意: 需要补充物品系统的单元测试

## 常见问题

### ECS 桥接
物品系统同时支持两种表示：
- `items::Item` - 物品模块的原生格式
- `ECSItem` - ECS 组件格式（在 `src/ecs.rs` 中定义）
- 通过 `ECSItem::from_items_item()` 和 `to_items_item()` 互相转换

### 序列化
所有物品类型都实现了 `serde` 和 `bincode` 序列化，用于存档

## 相关文件清单

| 文件 | 用途 |
|------|------|
| `src/items/src/lib.rs` | 模块入口、Item 定义、ItemTrait |
| `src/items/src/weapon.rs` | 武器系统 |
| `src/items/src/weapon/kind.rs` | 武器种类 |
| `src/items/src/weapon/tier.rs` | 武器等级 |
| `src/items/src/armor.rs` | 护甲 |
| `src/items/src/potion.rs` | 药水 |
| `src/items/src/scroll.rs` | 卷轴 |
| `src/items/src/wand.rs` | 法杖 |
| `src/items/src/ring.rs` | 戒指 |
| `src/items/src/seed.rs` | 种子 |
| `src/items/src/stone.rs` | 魔法石 |
| `src/items/src/food.rs` | 食物 |
| `src/items/src/throwable.rs` | 投掷武器 |
| `src/items/src/herb.rs` | 药草 |
| `src/items/src/misc.rs` | 杂项 |

## 变更记录 (Changelog)

### 2026-05-28
- 初始化模块文档
