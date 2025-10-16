//! 游戏错误处理模块
//!
//! 处理游戏运行过程中可能出现的各种错误，包括存档系统、序列化、IO等错误。

use bincode::error::{DecodeError, EncodeError};
use thiserror::Error;

/// 游戏运行过程中可能出现的错误类型
#[derive(Debug, Error)]
pub enum GameError {
    /// 存档系统错误
    #[error("Save system error: {0}")]
    SaveError(#[from] anyhow::Error),

    /// IO操作错误
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// 反序列化错误
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// 无效的存档槽位
    #[error("Invalid save slot")]
    InvalidSlot,

    /// 存档数据损坏
    #[error("Corrupted save data")]
    CorruptedSave,

    /// 游戏版本不兼容
    #[error("Incompatible game version: {0}")]
    VersionMismatch(String),

    /// 英雄数据无效
    #[error("Invalid hero data")]
    InvalidHeroData,

    /// 地图数据无效
    #[error("Invalid level data")]
    InvalidLevelData,

    /// 物品数据无效
    #[error("Invalid item data")]
    InvalidItemData,

    /// 怪物数据无效
    #[error("Invalid mob data")]
    InvalidMobData,

    /// 游戏状态无效
    #[error("Invalid game state")]
    InvalidGameState,

    /// 用户输入错误
    #[error("Input error: {0}")]
    InputError(String),
}

impl From<DecodeError> for GameError {
    fn from(err: DecodeError) -> Self {
        // 破碎的像素地牢中，反序列化错误通常意味着存档损坏
        if err.to_string().contains("invalid utf-8 sequence") {
            GameError::CorruptedSave
        } else {
            GameError::DeserializationError(err.to_string())
        }
    }
}

impl From<EncodeError> for GameError {
    fn from(err: EncodeError) -> Self {
        GameError::SerializationError(err.to_string())
    }
}

/// 处理游戏错误并转换为用户友好的消息
pub fn handle_error(error: &GameError) -> String {
    match error {
        GameError::CorruptedSave => "存档数据已损坏，无法加载".to_string(),
        GameError::InvalidSlot => "无效的存档槽位".to_string(),
        GameError::VersionMismatch(v) => format!("存档版本不兼容: {}", v),
        GameError::IoError(e) => match e.kind() {
            std::io::ErrorKind::NotFound => "存档文件不存在".to_string(),
            std::io::ErrorKind::PermissionDenied => "没有权限访问存档文件".to_string(),
            _ => format!("IO错误: {}", e),
        },
        _ => error.to_string(),
    }
}

/// 背包系统错误类型
#[derive(Debug)]
pub enum BagError {
    Full,
    ItemNotFound,
    EquipmentConflict,
}
