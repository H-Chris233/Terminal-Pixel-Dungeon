# Boss 战斗系统实现文档

## 概述

本文档描述了为 Terminal Pixel Dungeon 实现的完整 Boss 战斗系统。该系统包括 5 个独特的 Boss、特殊房间、Boss AI、技能系统、阶段转换机制和奖励系统。

## 功能特性

### 1. Boss 实体系统

#### Boss 类型
实现了 5 个独特的 Boss，每个出现在特定楼层：

| Boss | 楼层 | 类型 | 特点 |
|------|------|------|------|
| 巨型食人魔 (GiantOgre) | 第 5 层 | 近战暴力型 | 高血量、高攻击、范围攻击技能 |
| 暗影法师 (ShadowMage) | 第 10 层 | 远程魔法型 | 远程攻击、传送、致盲技能 |
| 毒液之王 (VenomLord) | 第 15 层 | 持续伤害型 | 毒液技能、范围攻击 |
| 机械守卫 (MechanicalGuardian) | 第 20 层 | 召唤小怪型 | 召唤小怪、自我修复、护盾 |
| 深渊领主 (AbyssalLord) | 第 25 层 | 终极 Boss | 多种技能、虚空裂隙、最强属性 |

#### Boss 属性
- **多阶段生命值系统**：根据血量百分比自动切换阶段
  - Phase1 (> 50% HP): 基础攻击模式
  - Phase2 (25-50% HP): 增强攻击 + 特殊技能
  - Enraged (< 25% HP): 狂暴状态，攻击力和速度大幅提升
  
- **护盾系统**：某些 Boss 可以生成护盾吸收伤害
- **免疫和抗性**：Boss 对特定状态效果免疫或有抗性
  - 例如：巨型食人魔免疫麻痹，50% 减速抗性
  - 机械守卫免疫中毒和流血（机械特性）

### 2. Boss 战斗机制

#### 阶段系统
```rust
pub enum BossPhase {
    Phase1,   // 第一阶段：基础攻击
    Phase2,   // 第二阶段：增加特殊技能
    Enraged,  // 狂暴阶段：低血量时攻击力和速度提升
}
```

阶段转换根据血量百分比自动触发：
- HP > 50%: Phase1
- 25% < HP <= 50%: Phase2
- HP <= 25%: Enraged

#### Boss 技能系统
实现了 11 种独特的 Boss 技能：

1. **AreaAttack**: 范围 AOE 攻击
2. **SummonMinions**: 召唤小怪协助战斗
3. **SelfHeal**: 自我治疗
4. **Shield**: 生成护盾
5. **ApplyStatus**: 施加状态效果
6. **Teleport**: 瞬间移动（逃离危险）
7. **Berserk**: 狂暴（提升攻击力和速度）
8. **ShadowBolt**: 暗影箭（远程魔法攻击）
9. **VenomSpit**: 毒液喷射（持续伤害）
10. **MechanicalRepair**: 机械修复（恢复生命值）
11. **VoidRift**: 虚空裂隙（地形改变 + 伤害）

每个技能都有独立的冷却时间机制：
```rust
pub struct SkillCooldowns {
    cooldowns: HashMap<String, u32>,
}
```

#### Boss AI 决策系统
Boss AI 根据以下因素选择行动：
- 与玩家的距离
- 当前血量百分比
- 技能冷却状态

示例逻辑：
```rust
pub fn choose_skill(&self, player_distance: f32, hp_percent: f32) -> Option<BossSkill> {
    // 低血量优先治疗或护盾
    if hp_percent < 0.3 {
        // 优先使用 SelfHeal 或 Shield
    }
    // 玩家距离较远时使用远程技能
    if player_distance > 5.0 {
        // 使用 ShadowBolt 等远程技能
    }
    // 玩家距离很近时使用 AOE
    if player_distance <= 3.0 {
        // 使用 AreaAttack
    }
}
```

### 3. Boss 房间设计

#### 房间生成
- 每 5 层生成一个 Boss 房间
- Boss 房间位于地图中心
- 竞技场式布局，半径根据 Boss 类型调整：
  - 巨型食人魔：半径 8
  - 暗影法师：半径 10
  - 毒液之王：半径 9
  - 机械守卫：半径 12
  - 深渊领主：半径 15

#### 环境元素
- **掩体**：3-8 个随机分布的障碍物，玩家可以利用躲避攻击
- **危险区域**：
  - 熔岩：造成 8-15 点伤害
  - 尖刺：造成 5-10 点伤害
  - 毒雾：每回合造成 2-5 点持续伤害

#### 房间特性
- 圆形竞技场布局，边缘有墙壁
- 北侧入口，连接走廊
- 进入时显示 Boss 警告信息
- 战斗胜利后解锁房间

### 4. Boss 战斗 UI

#### Boss 血条显示
使用 `src/render/boss.rs` 模块实现：
- 屏幕顶部显示 Boss 名称和血条
- 阶段指示器（第一阶段/第二阶段/狂暴状态）
- 护盾值显示（如果有）
- 根据血量百分比改变颜色：
  - > 66%: 绿色
  - 33-66%: 黄色
  - < 33%: 红色

#### Boss 房间入口警告
```
╔══════════════════════════════════════╗
║                                      ║
║      ⚠️  WARNING: BOSS AHEAD  ⚠️      ║
║                                      ║
║         {Boss 名称}                  ║
║                                      ║
╚══════════════════════════════════════╝
```

#### 技能使用提示
显示 Boss 使用的技能名称和描述

#### 战斗胜利奖励显示
显示获得的金币、物品数量和首杀奖励（如果适用）

### 5. 奖励系统

#### Boss 掉落
每个 Boss 击败后掉落：
- **金币**：
  - 巨型食人魔：100-200
  - 暗影法师：150-250
  - 毒液之王：200-350
  - 机械守卫：250-400
  - 深渊领主：500-800

- **装备**：保证掉落 1-6 件装备（根据 Boss 等级）
- **独特物品**：每个 Boss 都有专属物品掉落
- **消耗品**：2-5 件（药水、卷轴等）

#### 首杀奖励
- 首次击败 Boss 获得特殊标记
- 额外奖励和成就解锁
- 记录在玩家的 `BossDefeatRecord` 组件中

### 6. 事件系统集成

新增的 Boss 相关事件（在 `event_bus.rs` 中定义）：

```rust
pub enum GameEvent {
    // Boss 事件
    BossEncountered { boss_type: String, boss_entity: u32 },
    BossRoomEntered { boss_type: String },
    BossPhaseChanged { 
        boss_entity: u32, 
        old_phase: String, 
        new_phase: String 
    },
    BossSkillUsed { 
        boss_entity: u32, 
        skill_name: String 
    },
    BossDefeated { 
        boss_entity: u32, 
        boss_type: String, 
        is_first_kill: bool 
    },
    BossSummonedMinions { 
        boss_entity: u32, 
        minion_count: u32 
    },
    // ... 其他事件
}
```

## 技术实现

### 模块结构

```
src/
├── combat/
│   └── src/
│       ├── boss.rs          # Boss 实体和技能系统
│       ├── combatant.rs     # Combatant trait（Boss 实现此 trait）
│       └── ...
├── dungeon/
│   └── src/
│       ├── boss_room.rs     # Boss 房间生成和管理
│       ├── level.rs         # 集成 Boss 房间到地牢生成
│       └── ...
├── render/
│   ├── boss.rs              # Boss UI 渲染
│   └── ...
├── ecs.rs                   # Boss 相关 ECS 组件
├── systems.rs               # BossSystem 实现
└── event_bus.rs             # Boss 事件定义
```

### ECS 组件

添加了 3 个新组件：

1. **BossComponent**：标记 Boss 实体
```rust
pub struct BossComponent {
    pub boss_type: combat::boss::BossType,
    pub current_phase: combat::boss::BossPhase,
    pub shield: u32,
}
```

2. **BossSkillComponent**：管理 Boss 技能冷却
```rust
pub struct BossSkillComponent {
    pub cooldowns: combat::boss::SkillCooldowns,
    pub available_skills: Vec<combat::boss::BossSkill>,
}
```

3. **BossDefeatRecord**：记录玩家击败的 Boss
```rust
pub struct BossDefeatRecord {
    pub defeated_bosses: Vec<combat::boss::BossType>,
    pub first_kill_rewards_claimed: Vec<combat::boss::BossType>,
}
```

### BossSystem

在 `systems.rs` 中实现了 `BossSystem`，负责：
- 检查 Boss 阶段转换
- 更新 Boss 技能冷却
- 执行 Boss AI 逻辑
- 发布 Boss 相关事件

## 使用示例

### 创建 Boss 实体

```rust
use combat::boss::{Boss, BossType};

// 创建一个 Boss
let boss = Boss::new(BossType::GiantOgre, 50, 50);

// 转换为 ECS 组件
let boss_component = BossComponent {
    boss_type: boss.boss_type.clone(),
    current_phase: boss.phase.clone(),
    shield: boss.shield,
};

world.spawn((
    boss_component,
    Position::new(50, 50, 5),
    Stats { ... },
    Renderable { ... },
    // ... 其他组件
));
```

### 检查 Boss 房间

```rust
// 检查当前层是否有 Boss
if dungeon.has_boss() {
    if let Some(boss_room) = dungeon.get_boss_room() {
        println!("Boss 房间位置: {:?}", boss_room.arena_center);
        println!("Boss 类型: {}", boss_room.boss.name());
    }
}
```

### 渲染 Boss UI

```rust
use crate::render::BossUI;

// 在 Boss 战斗中渲染 Boss 血条
BossUI::render(
    frame,
    boss_ui_area,
    "巨型食人魔",
    &BossType::GiantOgre,
    boss_hp,
    boss_max_hp,
    &boss_phase,
    boss_shield,
);
```

## 平衡性设计

### Boss 难度曲线
- 第 5 层 Boss：适合等级 3-5 的玩家
- 第 10 层 Boss：适合等级 6-8 的玩家
- 第 15 层 Boss：适合等级 9-12 的玩家
- 第 20 层 Boss：适合等级 13-16 的玩家
- 第 25 层 Boss：适合等级 17-20 的玩家（终极挑战）

### 属性平衡
Boss 属性设计遵循以下原则：
- HP：比同层敌人高 5-10 倍
- 攻击力：比同层敌人高 2-3 倍
- 防御力：比同层敌人高 1.5-2 倍
- 经验值：100-500（递增）

### 技能冷却
- 基础攻击技能：2-4 回合
- 强力技能：6-8 回合
- 终极技能：10 回合

## 测试建议

### 单元测试
- 测试 Boss 阶段转换逻辑
- 测试技能冷却机制
- 测试免疫和抗性计算

### 集成测试
- 测试 Boss 房间生成
- 测试 Boss AI 决策
- 测试战斗流程

### 平衡性测试
- 测试不同等级玩家与 Boss 战斗
- 调整 Boss 属性和技能伤害
- 验证奖励掉落率

## 未来扩展

### 可能的增强功能
1. **Boss 变种**：同一 Boss 的不同难度版本
2. **Boss 组合**：多个 Boss 同时出现
3. **Boss 成就系统**：特殊击杀条件奖励
4. **Boss 动画**：更丰富的战斗动画效果
5. **Boss 对话**：战斗前后的剧情对话
6. **Boss 音乐**：专属 Boss 战斗音乐（如果添加音频支持）

## 参考文档

- `CLAUDE.md`：项目架构和开发指南
- `EVENT_BUS_GUIDE.md`：事件总线使用指南
- `src/combat/src/boss.rs`：Boss 实现源码
- `src/dungeon/src/boss_room.rs`：Boss 房间实现源码
- `src/render/boss.rs`：Boss UI 实现源码

## 作者说明

本 Boss 战斗系统实现了 Shattered Pixel Dungeon 风格的 Boss 战斗体验，包括：
- 多阶段战斗
- 丰富的技能系统
- 智能的 Boss AI
- 特殊的竞技场设计
- 完整的奖励机制

系统与现有的 ECS 架构完全集成，遵循项目的代码风格和设计模式。
