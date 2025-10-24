// src/dungeon/src/boss_room.rs

use bincode::{Decode, Encode};
use rand::Rng;
use serde::{Deserialize, Serialize};

use combat::boss::{Boss, BossType};

/// Boss 房间类型
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct BossRoom {
    pub boss: Boss,
    pub is_locked: bool,
    pub is_completed: bool,
    pub entrance_x: i32,
    pub entrance_y: i32,
    pub arena_center: (i32, i32),
    pub arena_radius: u32,
    pub obstacles: Vec<(i32, i32)>, // 掩体位置
    pub hazards: Vec<Hazard>,       // 危险区域
}

/// 危险区域类型
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub enum Hazard {
    Lava { x: i32, y: i32, damage: u32 },
    Spike { x: i32, y: i32, damage: u32 },
    Poison { x: i32, y: i32, damage_per_turn: u32 },
}

impl BossRoom {
    /// 创建新的 Boss 房间
    pub fn new(boss_type: BossType, center_x: i32, center_y: i32, rng: &mut impl Rng) -> Self {
        let arena_radius = match boss_type {
            BossType::GiantOgre => 8,
            BossType::ShadowMage => 10,
            BossType::VenomLord => 9,
            BossType::MechanicalGuardian => 12,
            BossType::AbyssalLord => 15,
        };

        // Boss 生成在竞技场中心
        let boss = Boss::new(boss_type.clone(), center_x, center_y);

        // 生成掩体
        let obstacles = Self::generate_obstacles(center_x, center_y, arena_radius, rng);

        // 生成危险区域
        let hazards = Self::generate_hazards(center_x, center_y, arena_radius, &boss_type, rng);

        // 入口在竞技场北侧
        let entrance_x = center_x;
        let entrance_y = center_y - arena_radius as i32 - 2;

        Self {
            boss,
            is_locked: false,
            is_completed: false,
            entrance_x,
            entrance_y,
            arena_center: (center_x, center_y),
            arena_radius,
            obstacles,
            hazards,
        }
    }

    /// 生成掩体
    fn generate_obstacles(
        center_x: i32,
        center_y: i32,
        radius: u32,
        rng: &mut impl Rng,
    ) -> Vec<(i32, i32)> {
        let mut obstacles = Vec::new();
        let count = rng.random_range(3..8);

        for _ in 0..count {
            let angle = rng.random_range(0.0..(2.0 * std::f32::consts::PI));
            let distance = rng.random_range((radius / 2) as f32..radius as f32);
            
            let x = center_x + (distance * angle.cos()) as i32;
            let y = center_y + (distance * angle.sin()) as i32;
            
            obstacles.push((x, y));
        }

        obstacles
    }

    /// 生成危险区域
    fn generate_hazards(
        center_x: i32,
        center_y: i32,
        radius: u32,
        boss_type: &BossType,
        rng: &mut impl Rng,
    ) -> Vec<Hazard> {
        let mut hazards = Vec::new();

        let (hazard_type, count) = match boss_type {
            BossType::GiantOgre => (HazardType::Spike, rng.random_range(2..5)),
            BossType::ShadowMage => (HazardType::None, 0),
            BossType::VenomLord => (HazardType::Poison, rng.random_range(4..8)),
            BossType::MechanicalGuardian => (HazardType::Spike, rng.random_range(3..6)),
            BossType::AbyssalLord => (HazardType::Lava, rng.random_range(5..10)),
        };

        for _ in 0..count {
            let angle = rng.random_range(0.0..(2.0 * std::f32::consts::PI));
            let distance = rng.random_range((radius / 3) as f32..radius as f32);
            
            let x = center_x + (distance * angle.cos()) as i32;
            let y = center_y + (distance * angle.sin()) as i32;

            let hazard = match hazard_type {
                HazardType::Lava => Hazard::Lava { x, y, damage: rng.random_range(8..15) },
                HazardType::Spike => Hazard::Spike { x, y, damage: rng.random_range(5..10) },
                HazardType::Poison => Hazard::Poison { x, y, damage_per_turn: rng.random_range(2..5) },
                HazardType::None => continue,
            };

            hazards.push(hazard);
        }

        hazards
    }

    /// 锁定房间（进入 Boss 战时）
    pub fn lock(&mut self) {
        self.is_locked = true;
    }

    /// 解锁房间（击败 Boss 后）
    pub fn unlock(&mut self) {
        self.is_locked = false;
        self.is_completed = true;
    }

    /// 检查位置是否在竞技场内
    pub fn is_in_arena(&self, x: i32, y: i32) -> bool {
        let dx = (x - self.arena_center.0) as f32;
        let dy = (y - self.arena_center.1) as f32;
        let distance = (dx * dx + dy * dy).sqrt();
        distance <= self.arena_radius as f32
    }

    /// 获取竞技场边界
    pub fn get_arena_bounds(&self) -> (i32, i32, i32, i32) {
        let r = self.arena_radius as i32;
        (
            self.arena_center.0 - r,
            self.arena_center.1 - r,
            self.arena_center.0 + r,
            self.arena_center.1 + r,
        )
    }

    /// 检查位置是否是掩体
    pub fn is_obstacle(&self, x: i32, y: i32) -> bool {
        self.obstacles.contains(&(x, y))
    }

    /// 检查位置是否是危险区域
    pub fn get_hazard_at(&self, x: i32, y: i32) -> Option<&Hazard> {
        self.hazards.iter().find(|h| match h {
            Hazard::Lava { x: hx, y: hy, .. } => *hx == x && *hy == y,
            Hazard::Spike { x: hx, y: hy, .. } => *hx == x && *hy == y,
            Hazard::Poison { x: hx, y: hy, .. } => *hx == x && *hy == y,
        })
    }

    /// 生成 Boss 房间的 ASCII 艺术边框描述
    pub fn get_entrance_message(&self) -> String {
        format!(
            r#"
╔══════════════════════════════════════╗
║                                      ║
║      ⚠️  WARNING: BOSS AHEAD  ⚠️      ║
║                                      ║
║         {}          ║
║                                      ║
╚══════════════════════════════════════╝
"#,
            self.boss.name()
        )
    }
}

/// 危险区域类型枚举（用于生成）
#[derive(Clone, Debug)]
enum HazardType {
    Lava,
    Spike,
    Poison,
    None,
}

impl Hazard {
    /// 获取危险区域的伤害
    pub fn damage(&self) -> u32 {
        match self {
            Self::Lava { damage, .. } => *damage,
            Self::Spike { damage, .. } => *damage,
            Self::Poison { damage_per_turn, .. } => *damage_per_turn,
        }
    }

    /// 获取危险区域的描述
    pub fn description(&self) -> &str {
        match self {
            Self::Lava { .. } => "熔岩",
            Self::Spike { .. } => "尖刺",
            Self::Poison { .. } => "毒雾",
        }
    }

    /// 获取位置
    pub fn position(&self) -> (i32, i32) {
        match self {
            Self::Lava { x, y, .. } => (*x, *y),
            Self::Spike { x, y, .. } => (*x, *y),
            Self::Poison { x, y, .. } => (*x, *y),
        }
    }
}

/// Boss 房间布局类型
#[derive(Clone, Debug)]
pub enum BossRoomLayout {
    /// 圆形竞技场
    CircularArena,
    /// 方形竞技场
    SquareArena,
    /// 迷宫式竞技场
    MazeArena,
}

impl BossRoomLayout {
    /// 根据 Boss 类型选择布局
    pub fn for_boss_type(boss_type: &BossType) -> Self {
        match boss_type {
            BossType::GiantOgre => Self::CircularArena,
            BossType::ShadowMage => Self::MazeArena,
            BossType::VenomLord => Self::CircularArena,
            BossType::MechanicalGuardian => Self::SquareArena,
            BossType::AbyssalLord => Self::CircularArena,
        }
    }
}
