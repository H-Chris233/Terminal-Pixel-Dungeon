
// src/dungeon/src/level/tiles/tiles.rs
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 表示游戏中的一个地图格子
/// 使用#[repr(C)]优化内存布局，字段按从大到小排列减少padding
#[repr(C)]
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Tile {
    /// 格子的属性信息
    pub info: TileInfo,
    /// 格子的x坐标(地图坐标系)
    pub x: i32,
    /// 格子的y坐标(地图坐标系)
    pub y: i32,
}

impl Tile {
    /// 创建一个新的Tile实例
    pub fn new(x: i32, y: i32, info: TileInfo) -> Self {
        Self { x, y, info }
    }

    /// 检查格子是否可通行(有敌人的格子不可通行)
    pub fn is_passable(&self) -> bool {
        self.info.passable && !self.info.has_enemy
    }

    /// 检查格子是否阻挡视线
    pub fn blocks_sight(&self) -> bool {
        self.info.blocks_sight
    }

    /// 检查格子是否可见
    pub fn is_visible(&self) -> bool {
        self.info.is_visible
    }

    /// 重置格子的可见状态(用于探索和记忆系统)
    pub fn reset_visibility(&mut self) {
        self.info.is_visible = false;
    }

    /// 设置格子的可见状态
    pub fn set_visible(&mut self, visible: bool) {
        self.info.is_visible = visible;
    }

    /// 检查是否是门
    pub fn is_door(&self) -> bool {
        matches!(self.info.terrain_type, TerrainType::Door(_))
    }

    /// 检查是否是陷阱
    pub fn is_trap(&self) -> bool {
    matches!(self.info.terrain_type, TerrainType::Trap(..))
}

    /// 尝试开门(返回操作是否成功)
    pub fn try_open_door(&mut self) -> bool {
        if let TerrainType::Door(state) = &mut self.info.terrain_type {
            match state {
                DoorState::Closed => {
                    *state = DoorState::Open;
                    self.info.passable = true;
                    self.info.blocks_sight = false;
                    true
                }
                DoorState::Locked => false,
                DoorState::Open => true,
            }
        } else {
            false
        }
    }

    /// 触发陷阱(返回陷阱类型)
    pub fn trigger_trap(&mut self) -> Option<TrapType> {
        if let TerrainType::Trap(state, trap_type) = &mut self.info.terrain_type {
            *state = TrapState::Triggered;
            self.info.passable = true; // 触发后的陷阱通常可通行
            Some(*trap_type)
        } else {
            None
        }
    }
}

/// 格子的属性信息
/// 使用#[repr(C)]优化内存布局，将布尔值打包在一起
#[repr(C)]
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct TileInfo {
    /// 地形类型
    pub terrain_type: TerrainType,
    /// 基础通行性(不考虑敌人/物品)
    pub passable: bool,
    /// 是否阻挡视线(影响FOV计算)
    pub blocks_sight: bool,
    /// 是否有物品
    pub has_item: bool,
    /// 是否有敌人
    pub has_enemy: bool,
    /// 当前是否可见(用于FOV计算)
    pub is_visible: bool,
    /// 是否已被探索过(用于记忆系统)
    pub explored: bool,
}

impl TileInfo {
    /// 创建一个新的TileInfo实例
    pub fn new(
        passable: bool,
        blocks_sight: bool,
        terrain_type: TerrainType,
    ) -> Self {
        Self {
            terrain_type,
            passable,
            blocks_sight,
            has_item: false,
            has_enemy: false,
            is_visible: false,
            explored: false,
        }
    }
}

/// 地形类型枚举
#[derive(Copy, Clone, Debug, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerrainType {
    /// 普通地板
    Floor,
    /// 墙壁
    Wall,
    /// 水域(可能减速或伤害)
    Water,
    /// 草地(可能隐藏物品)
    Grass,
    /// 门(可开关)
    Door(DoorState),
    /// 陷阱(隐藏或可见)
    Trap(TrapState, TrapType),
    /// 楼梯(上下层)
    Stair(StairDirection),
    /// 特殊地形(如祭坛等)
    Special,
}

/// 门的状态
#[derive(Copy, Clone, Debug, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
pub enum DoorState {
    /// 关闭状态
    Closed,
    /// 打开状态
    Open,
    /// 锁定状态
    Locked,
}

/// 陷阱的状态
#[derive(Copy, Clone, Debug, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrapState {
    /// 隐藏的陷阱
    Hidden,
    /// 可见的陷阱
    Visible,
    /// 已触发的陷阱
    Triggered,
}

/// 陷阱类型
#[derive(Copy, Clone, Debug, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrapType {
    /// 伤害陷阱
    Damage(u8),
    /// 减速陷阱
    Slow(u8),
    /// 毒陷阱
    Poison(u8),
    /// 警报陷阱
    Alarm,
    /// 传送陷阱
    Teleport,
}

/// 楼梯方向
#[derive(Copy, Clone, Debug, Encode, Decode, Serialize, Deserialize, PartialEq, Eq)]
pub enum StairDirection {
    /// 上楼
    Up,
    /// 下楼
    Down,
}

// 为Tile实现常用的trait
impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Tile {}

impl std::hash::Hash for Tile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}
