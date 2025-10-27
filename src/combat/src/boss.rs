// src/combat/src/boss.rs

use bincode::{Decode, Encode};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::combatant::Combatant;
use crate::effect::{Effect, EffectType};
use items::Weapon;

/// Boss 类型，每个 Boss 出现在特定楼层
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize, PartialEq)]
pub enum BossType {
    /// 第 5 层：巨型食人魔（近战暴力型）
    GiantOgre,
    /// 第 10 层：暗影法师（远程魔法型）
    ShadowMage,
    /// 第 15 层：毒液之王（持续伤害型）
    VenomLord,
    /// 第 20 层：机械守卫（召唤小怪型）
    MechanicalGuardian,
    /// 第 25 层：深渊领主（终极 Boss，多机制）
    AbyssalLord,
}

impl BossType {
    /// 根据层数获取对应的 Boss 类型
    pub fn for_depth(depth: usize) -> Option<Self> {
        match depth {
            5 => Some(Self::GiantOgre),
            10 => Some(Self::ShadowMage),
            15 => Some(Self::VenomLord),
            20 => Some(Self::MechanicalGuardian),
            25 => Some(Self::AbyssalLord),
            _ => None,
        }
    }

    /// 获取 Boss 名称
    pub fn name(&self) -> &str {
        match self {
            Self::GiantOgre => "巨型食人魔",
            Self::ShadowMage => "暗影法师",
            Self::VenomLord => "毒液之王",
            Self::MechanicalGuardian => "机械守卫",
            Self::AbyssalLord => "深渊领主",
        }
    }

    /// 获取 Boss 符号
    pub fn symbol(&self) -> char {
        match self {
            Self::GiantOgre => 'Ø',
            Self::ShadowMage => 'Ψ',
            Self::VenomLord => 'Ω',
            Self::MechanicalGuardian => '¤',
            Self::AbyssalLord => '☠',
        }
    }

    /// 获取 Boss 颜色 (RGB)
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Self::GiantOgre => (255, 100, 50),           // 橙红色
            Self::ShadowMage => (100, 50, 200),          // 紫色
            Self::VenomLord => (50, 255, 100),           // 毒绿色
            Self::MechanicalGuardian => (200, 200, 200), // 银灰色
            Self::AbyssalLord => (150, 0, 0),            // 暗红色
        }
    }
}

/// Boss 阶段，用于控制 Boss 的行为变化
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize, PartialEq)]
pub enum BossPhase {
    /// 第一阶段：基础攻击模式
    Phase1,
    /// 第二阶段：增强攻击 + 特殊技能
    Phase2,
    /// 狂暴阶段：低血量时的疯狂状态
    Enraged,
}

impl BossPhase {
    /// 根据血量百分比确定当前阶段
    pub fn from_health_percent(hp_percent: f32) -> Self {
        if hp_percent <= 0.25 {
            Self::Enraged
        } else if hp_percent <= 0.5 {
            Self::Phase2
        } else {
            Self::Phase1
        }
    }
}

/// Boss 技能定义
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize, PartialEq)]
pub enum BossSkill {
    /// AOE 范围攻击（半径）
    AreaAttack { radius: u32, damage_multiplier: f32 },
    /// 召唤小怪（数量）
    SummonMinions { count: u32 },
    /// 自我治疗（治疗量百分比）
    SelfHeal { percent: f32 },
    /// 生成护盾（吸收伤害值）
    Shield { amount: u32 },
    /// 施加状态效果
    ApplyStatus { status: EffectType, duration: u32 },
    /// 传送（逃离危险）
    Teleport,
    /// 狂暴（提升攻击力和速度）
    Berserk { attack_boost: f32, speed_boost: f32 },
    /// 暗影箭（远程魔法攻击）
    ShadowBolt { damage_multiplier: f32 },
    /// 毒液喷射（持续伤害）
    VenomSpit { damage_per_turn: u32, duration: u32 },
    /// 机械修复（恢复生命值）
    MechanicalRepair { heal_amount: u32 },
    /// 虚空裂隙（地形改变 + 伤害）
    VoidRift { damage: u32, duration: u32 },
}

impl BossSkill {
    /// 获取技能名称
    pub fn name(&self) -> &str {
        match self {
            Self::AreaAttack { .. } => "范围攻击",
            Self::SummonMinions { .. } => "召唤小怪",
            Self::SelfHeal { .. } => "自我治疗",
            Self::Shield { .. } => "能量护盾",
            Self::ApplyStatus { .. } => "施加负面状态",
            Self::Teleport => "瞬间移动",
            Self::Berserk { .. } => "狂暴",
            Self::ShadowBolt { .. } => "暗影箭",
            Self::VenomSpit { .. } => "毒液喷射",
            Self::MechanicalRepair { .. } => "机械修复",
            Self::VoidRift { .. } => "虚空裂隙",
        }
    }

    /// 获取技能冷却时间（回合数）
    pub fn cooldown(&self) -> u32 {
        match self {
            Self::AreaAttack { .. } => 4,
            Self::SummonMinions { .. } => 6,
            Self::SelfHeal { .. } => 8,
            Self::Shield { .. } => 5,
            Self::ApplyStatus { .. } => 3,
            Self::Teleport => 7,
            Self::Berserk { .. } => 10,
            Self::ShadowBolt { .. } => 2,
            Self::VenomSpit { .. } => 3,
            Self::MechanicalRepair { .. } => 6,
            Self::VoidRift { .. } => 8,
        }
    }
}

/// Boss 技能冷却状态
#[derive(Clone, Debug, Default, Encode, Decode, Serialize, Deserialize)]
pub struct SkillCooldowns {
    cooldowns: HashMap<String, u32>,
}

impl SkillCooldowns {
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查技能是否可用
    pub fn is_available(&self, skill: &BossSkill) -> bool {
        let key = format!("{:?}", skill);
        !self.cooldowns.contains_key(&key) || self.cooldowns[&key] == 0
    }

    /// 使用技能（设置冷却）
    pub fn use_skill(&mut self, skill: &BossSkill) {
        let key = format!("{:?}", skill);
        self.cooldowns.insert(key, skill.cooldown());
    }

    /// 更新冷却（每回合调用）
    pub fn tick(&mut self) {
        self.cooldowns.retain(|_, cd| {
            *cd = cd.saturating_sub(1);
            *cd > 0
        });
    }

    /// 重置所有冷却
    pub fn reset(&mut self) {
        self.cooldowns.clear();
    }
}

/// Boss 实体
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Boss {
    pub boss_type: BossType,
    pub hp: u32,
    pub max_hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub exp_value: u32,
    pub x: i32,
    pub y: i32,
    pub phase: BossPhase,
    pub skills: Vec<BossSkill>,
    pub cooldowns: SkillCooldowns,
    pub effects: Vec<Effect>,
    pub shield: u32,
    pub immunities: Vec<EffectType>,
    pub resistances: HashMap<EffectType, f32>, // 抗性百分比
    pub entity_id: Option<u32>,
    pub first_kill_bonus: bool, // 是否已获得首杀奖励
    pub summon_count: u32,      // 已召唤的小怪数量
}

impl Boss {
    /// 创建新的 Boss 实例
    pub fn new(boss_type: BossType, x: i32, y: i32) -> Self {
        let (hp, attack, defense, exp_value, skills, immunities, resistances) =
            Self::get_boss_stats(&boss_type);

        Self {
            boss_type,
            hp,
            max_hp: hp,
            attack,
            defense,
            exp_value,
            x,
            y,
            phase: BossPhase::Phase1,
            skills,
            cooldowns: SkillCooldowns::new(),
            effects: Vec::new(),
            shield: 0,
            immunities,
            resistances,
            entity_id: None,
            first_kill_bonus: true,
            summon_count: 0,
        }
    }

    /// 获取 Boss 基础属性和技能
    fn get_boss_stats(
        boss_type: &BossType,
    ) -> (
        u32,
        u32,
        u32,
        u32,
        Vec<BossSkill>,
        Vec<EffectType>,
        HashMap<EffectType, f32>,
    ) {
        match boss_type {
            BossType::GiantOgre => (
                200, // HP
                30,  // 攻击
                15,  // 防御
                100, // 经验值
                vec![
                    BossSkill::AreaAttack {
                        radius: 2,
                        damage_multiplier: 1.5,
                    },
                    BossSkill::Berserk {
                        attack_boost: 1.5,
                        speed_boost: 1.3,
                    },
                ],
                vec![EffectType::Paralysis], // 免疫眩晕
                [(EffectType::Slow, 0.5)].iter().cloned().collect(), // 50% 减速抗性
            ),
            BossType::ShadowMage => (
                150,
                35,
                10,
                150,
                vec![
                    BossSkill::ShadowBolt {
                        damage_multiplier: 1.8,
                    },
                    BossSkill::Teleport,
                    BossSkill::ApplyStatus {
                        status: EffectType::Darkness,
                        duration: 3,
                    },
                ],
                vec![EffectType::Darkness], // 免疫致盲
                [(EffectType::Burning, 0.5)].iter().cloned().collect(),
            ),
            BossType::VenomLord => (
                180,
                25,
                12,
                200,
                vec![
                    BossSkill::VenomSpit {
                        damage_per_turn: 5,
                        duration: 5,
                    },
                    BossSkill::AreaAttack {
                        radius: 3,
                        damage_multiplier: 1.2,
                    },
                    BossSkill::ApplyStatus {
                        status: EffectType::Poison,
                        duration: 8,
                    },
                ],
                vec![EffectType::Poison], // 免疫中毒
                [(EffectType::Bleeding, 0.3)].iter().cloned().collect(),
            ),
            BossType::MechanicalGuardian => (
                250,
                28,
                20,
                250,
                vec![
                    BossSkill::SummonMinions { count: 3 },
                    BossSkill::MechanicalRepair { heal_amount: 50 },
                    BossSkill::Shield { amount: 80 },
                ],
                vec![EffectType::Poison, EffectType::Bleeding], // 免疫生物状态
                [(EffectType::Burning, 0.7), (EffectType::Frost, 0.5)]
                    .iter()
                    .cloned()
                    .collect(),
            ),
            BossType::AbyssalLord => (
                300,
                40,
                18,
                500,
                vec![
                    BossSkill::VoidRift {
                        damage: 25,
                        duration: 4,
                    },
                    BossSkill::SummonMinions { count: 5 },
                    BossSkill::SelfHeal { percent: 0.15 },
                    BossSkill::AreaAttack {
                        radius: 4,
                        damage_multiplier: 2.0,
                    },
                    BossSkill::Berserk {
                        attack_boost: 2.0,
                        speed_boost: 1.5,
                    },
                ],
                vec![EffectType::Paralysis, EffectType::Rooted],
                [
                    (EffectType::Burning, 0.5),
                    (EffectType::Frost, 0.5),
                    (EffectType::Poison, 0.5),
                    (EffectType::Bleeding, 0.5),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
        }
    }

    /// 获取当前血量百分比
    pub fn health_percent(&self) -> f32 {
        self.hp as f32 / self.max_hp as f32
    }

    /// 更新阶段
    pub fn update_phase(&mut self) -> Option<BossPhase> {
        let new_phase = BossPhase::from_health_percent(self.health_percent());
        if new_phase != self.phase {
            self.phase = new_phase.clone();
            Some(new_phase)
        } else {
            None
        }
    }

    /// 受到伤害（考虑护盾）
    pub fn take_damage_with_shield(&mut self, amount: u32) -> u32 {
        if self.shield > 0 {
            if self.shield >= amount {
                self.shield -= amount;
                return 0; // 护盾完全吸收
            } else {
                let remaining = amount - self.shield;
                self.shield = 0;
                self.hp = self.hp.saturating_sub(remaining);
                return remaining;
            }
        }
        self.hp = self.hp.saturating_sub(amount);
        amount
    }

    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    /// 治疗
    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// 添加护盾
    pub fn add_shield(&mut self, amount: u32) {
        self.shield += amount;
    }

    /// 选择要使用的技能（AI 决策）
    pub fn choose_skill(&self, player_distance: f32, hp_percent: f32) -> Option<BossSkill> {
        let available_skills: Vec<BossSkill> = self
            .skills
            .iter()
            .filter(|skill| self.cooldowns.is_available(skill))
            .cloned()
            .collect();

        if available_skills.is_empty() {
            return None;
        }

        // 根据情况选择技能
        let mut rng = rand::rng();

        // 低血量优先治疗或护盾
        if hp_percent < 0.3 {
            if let Some(skill) = available_skills
                .iter()
                .find(|s| matches!(s, BossSkill::SelfHeal { .. }))
            {
                return Some(skill.clone());
            }
            if let Some(skill) = available_skills
                .iter()
                .find(|s| matches!(s, BossSkill::Shield { .. }))
            {
                return Some(skill.clone());
            }
        }

        // 玩家距离较远时使用远程技能或传送
        if player_distance > 5.0 {
            if let Some(skill) = available_skills
                .iter()
                .find(|s| matches!(s, BossSkill::ShadowBolt { .. }))
            {
                return Some(skill.clone());
            }
        }

        // 玩家距离很近时使用 AOE
        if player_distance <= 3.0 {
            if let Some(skill) = available_skills
                .iter()
                .find(|s| matches!(s, BossSkill::AreaAttack { .. }))
            {
                if rng.random_bool(0.6) {
                    return Some(skill.clone());
                }
            }
        }

        // 随机选择一个可用技能
        if !available_skills.is_empty() {
            Some(available_skills[rng.random_range(0..available_skills.len())].clone())
        } else {
            None
        }
    }

    /// 使用技能
    pub fn use_skill(&mut self, skill: &BossSkill) {
        self.cooldowns.use_skill(skill);
    }

    /// 更新冷却时间（每回合）
    pub fn tick_cooldowns(&mut self) {
        self.cooldowns.tick();
    }

    /// 检查是否免疫某个状态效果
    pub fn is_immune(&self, status: &EffectType) -> bool {
        self.immunities.contains(status)
    }

    /// 获取对某个状态的抗性（0.0-1.0）
    pub fn get_resistance(&self, status: &EffectType) -> f32 {
        *self.resistances.get(status).unwrap_or(&0.0)
    }

    /// Boss 名称
    pub fn name(&self) -> &str {
        self.boss_type.name()
    }

    /// Boss 符号
    pub fn symbol(&self) -> char {
        self.boss_type.symbol()
    }

    /// Boss 颜色
    pub fn color(&self) -> (u8, u8, u8) {
        self.boss_type.color()
    }

    /// 生成 Boss 掉落物品
    pub fn generate_loot(&self) -> BossLoot {
        let mut rng = rand::rng();

        // 基础金币掉落
        let gold = match self.boss_type {
            BossType::GiantOgre => rng.random_range(100..200),
            BossType::ShadowMage => rng.random_range(150..250),
            BossType::VenomLord => rng.random_range(200..350),
            BossType::MechanicalGuardian => rng.random_range(250..400),
            BossType::AbyssalLord => rng.random_range(500..800),
        };

        // 保证掉落的装备数量
        let equipment_count = match self.boss_type {
            BossType::GiantOgre => rng.random_range(1..3),
            BossType::ShadowMage => rng.random_range(2..4),
            BossType::VenomLord => rng.random_range(2..4),
            BossType::MechanicalGuardian => rng.random_range(3..5),
            BossType::AbyssalLord => rng.random_range(4..7),
        };

        BossLoot {
            gold,
            equipment_count,
            has_unique_item: true,
            consumables_count: rng.random_range(2..5),
        }
    }
}

/// Boss 掉落物品信息
#[derive(Clone, Debug)]
pub struct BossLoot {
    pub gold: u32,
    pub equipment_count: u32,
    pub has_unique_item: bool,
    pub consumables_count: u32,
}

/// 实现 Combatant trait 以便与现有战斗系统集成
impl Combatant for Boss {
    fn id(&self) -> u32 {
        self.entity_id.unwrap_or(0)
    }

    fn hp(&self) -> u32 {
        self.hp
    }

    fn max_hp(&self) -> u32 {
        self.max_hp
    }

    fn name(&self) -> &str {
        self.boss_type.name()
    }

    fn attack_power(&self) -> u32 {
        // 根据阶段调整攻击力
        let multiplier = match self.phase {
            BossPhase::Phase1 => 1.0,
            BossPhase::Phase2 => 1.3,
            BossPhase::Enraged => 1.6,
        };
        (self.attack as f32 * multiplier) as u32
    }

    fn defense(&self) -> u32 {
        self.defense
    }

    fn accuracy(&self) -> u32 {
        // Boss 有较高的基础命中
        let base = 20;
        match self.phase {
            BossPhase::Phase1 => base,
            BossPhase::Phase2 => base + 5,
            BossPhase::Enraged => base + 10,
        }
    }

    fn evasion(&self) -> u32 {
        // Boss 闪避相对较低，但在某些阶段会提升
        match self.phase {
            BossPhase::Phase1 => 8,
            BossPhase::Phase2 => 12,
            BossPhase::Enraged => 16,
        }
    }

    fn crit_bonus(&self) -> f32 {
        match self.phase {
            BossPhase::Phase1 => 0.15,
            BossPhase::Phase2 => 0.25,
            BossPhase::Enraged => 0.35,
        }
    }

    fn weapon(&self) -> Option<&Weapon> {
        None // Boss 不使用普通武器
    }

    fn attack_distance(&self) -> u32 {
        // Boss 根据类型有不同的攻击距离
        match self.boss_type {
            BossType::GiantOgre => 1,
            BossType::ShadowMage => 8,
            BossType::VenomLord => 5,
            BossType::MechanicalGuardian => 3,
            BossType::AbyssalLord => 6,
        }
    }

    fn take_damage(&mut self, amount: u32) -> bool {
        self.take_damage_with_shield(amount);
        self.update_phase();
        self.is_alive()
    }

    fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    fn is_alive(&self) -> bool {
        self.hp > 0
    }

    fn exp_value(&self) -> u32 {
        self.exp_value
    }
}
