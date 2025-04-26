// src/dungeon/src/trap.rs
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 表示地牢中的一个陷阱
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Trap {
    kind: TrapKind,   // 陷阱类型
    visible: bool,    // 是否对玩家可见
    active: bool,     // 是否处于激活状态(可触发)
    triggered: bool,  // 是否已被触发过
    discovered: bool, // 是否已被发现(通过搜索或技能)
    pos: (i32, i32),  // 在地牢中的位置坐标
    detection_dc: u8, // 发现难度等级(0-255)
}

impl Trap {
    /// 创建一个新的隐藏陷阱
    pub fn new(kind: TrapKind, pos: (i32, i32), detection_dc: u8) -> Self {
        Trap {
            kind,
            visible: false,
            active: true,
            triggered: false,
            discovered: false,
            pos,
            detection_dc,
        }
    }

    /// 使陷阱可见(对玩家显示)
    pub fn reveal(&mut self) {
        self.visible = true;
    }

    /// 解除陷阱(使其失效)
    pub fn disarm(&mut self) {
        self.active = false;
    }

    /// 检查陷阱是否对玩家可见
    pub fn is_visible(&self) -> bool {
        self.visible || self.discovered
    }

    /// 检查陷阱是否处于激活状态(可触发)
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// 检查陷阱是否已被触发过
    pub fn is_triggered(&self) -> bool {
        self.triggered
    }

    pub fn kind(&self) -> &TrapKind {
        &self.kind
    }

    /// 检查陷阱是否已被发现
    pub fn is_discovered(&self) -> bool {
        self.discovered
    }

    /// 尝试发现陷阱(基于玩家感知值)
    pub fn try_discover(&mut self, perception: u8) -> bool {
        if !self.discovered && perception >= self.detection_dc {
            self.discovered = true;
            true
        } else {
            false
        }
    }

    /// 强制发现陷阱(无视难度)
    pub fn force_discover(&mut self) {
        self.discovered = true;
    }

    /// 触发陷阱并返回其效果
    /// 触发后陷阱会被标记为已触发状态并变为可见
    pub fn force_trigger(&mut self) -> TrapEffect {
        if !self.visible {
            self.reveal();
        }
        self.triggered = true;
        self.kind.effect()
    }

    /// 获取陷阱效果
    pub fn effect(&self) -> TrapEffect {
        self.kind.effect()
    }

    /// 尝试触发陷阱(仅当未触发过且处于激活状态时)
    pub fn trigger(&mut self) -> Option<TrapEffect> {
        if !self.triggered && self.active {
            Some(self.force_trigger())
        } else {
            None
        }
    }

    /// 重置陷阱状态(可再次触发)
    pub fn reset(&mut self) {
        self.triggered = false;
    }

    /// 获取陷阱在地图中的位置
    pub fn position(&self) -> (i32, i32) {
        self.pos
    }

    /// 获取陷阱的发现难度
    pub fn detection_difficulty(&self) -> u8 {
        self.detection_dc
    }
}

/// 不同类型的陷阱及其特定行为
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum TrapKind {
    /// 飞镖陷阱 - 触发时造成伤害
    Dart {
        damage: u32, // 伤害值
    },
    /// 毒镖陷阱 - 触发时造成中毒效果
    Poison {
        damage: u32,   // 每回合伤害
        duration: u32, // 持续时间(回合数)
    },
    /// 警报陷阱 - 触发时警告附近敌人
    Alarm,
    /// 传送陷阱 - 触发时随机传送玩家
    Teleport,
    /// 麻痹陷阱 - 触发时使玩家麻痹
    Paralyze {
        duration: u32, // 麻痹持续时间(回合数)
    },
    /// 召唤陷阱 - 触发时在周围召唤敌人
    Summon,
    /// 火焰陷阱 - 触发时点燃玩家
    Fire {
        damage: u32, // 火焰伤害
    },
    /// 陷坑陷阱 - 触发时将玩家掉落至下层
    Pitfall,
    /// 束缚陷阱 - 触发时将玩家固定在原地
    Gripping {
        duration: u32, // 束缚持续时间(回合数)
    },
    /// 解除陷阱 - 触发时解除本层其他陷阱
    Disarming,
}

impl TrapKind {
    /// 返回触发此陷阱的效果
    pub fn effect(&self) -> TrapEffect {
        match self {
            TrapKind::Dart { damage } => TrapEffect::Damage(*damage),
            TrapKind::Poison { damage, duration } => TrapEffect::Poison(*damage, *duration),
            TrapKind::Alarm => TrapEffect::Alarm,
            TrapKind::Teleport => TrapEffect::Teleport,
            TrapKind::Paralyze { duration } => TrapEffect::Paralyze(*duration),
            TrapKind::Summon => TrapEffect::Summon,
            TrapKind::Fire { damage } => TrapEffect::Fire(*damage),
            TrapKind::Pitfall => TrapEffect::Pitfall,
            TrapKind::Gripping { duration } => TrapEffect::Grip(*duration),
            TrapKind::Disarming => TrapEffect::DisarmOtherTraps,
        }
    }
}

/// 陷阱触发时产生的效果
#[derive(Debug, Clone, Encode, Decode)]
pub enum TrapEffect {
    /// 直接伤害效果
    Damage(u32),
    /// 中毒效果(持续伤害)
    Poison(u32, u32), // 每回合伤害, 持续时间
    /// 警报效果(吸引敌人)
    Alarm,
    /// 传送效果
    Teleport,
    /// 麻痹效果(无法移动)
    Paralyze(u32), // 持续时间
    /// 召唤敌人效果
    Summon,
    /// 火焰伤害效果
    Fire(u32), // 初始伤害
    /// 掉落至下层效果
    Pitfall,
    /// 束缚效果(无法移动)
    Grip(u32), // 持续时间
    /// 解除其他陷阱效果
    DisarmOtherTraps,
}

impl fmt::Display for TrapKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrapKind::Dart { .. } => write!(f, "飞镖陷阱"),
            TrapKind::Poison { .. } => write!(f, "毒镖陷阱"),
            TrapKind::Alarm => write!(f, "警报陷阱"),
            TrapKind::Teleport => write!(f, "传送陷阱"),
            TrapKind::Paralyze { .. } => write!(f, "麻痹陷阱"),
            TrapKind::Summon => write!(f, "召唤陷阱"),
            TrapKind::Fire { .. } => write!(f, "火焰陷阱"),
            TrapKind::Pitfall => write!(f, "陷坑陷阱"),
            TrapKind::Gripping { .. } => write!(f, "束缚陷阱"),
            TrapKind::Disarming => write!(f, "解除陷阱"),
        }
    }
}

impl From<TrapKind> for Trap {
    fn from(kind: TrapKind) -> Self {
        Trap {
            kind,
            visible: false,
            active: true,
            triggered: false,
            discovered: false,
            pos: (0, 0),      // 默认位置，使用时需要手动设置
            detection_dc: 15, // 默认发现难度
        }
    }
}
