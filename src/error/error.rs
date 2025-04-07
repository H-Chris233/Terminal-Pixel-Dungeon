use bincode::error::DecodeError; // 对于反序列化错误
use bincode::error::EncodeError;
use thiserror::Error; // 对于序列化错误

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Save system error: {0}")]
    SaveError(#[from] anyhow::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid save slot")]
    InvalidSlot,
    // 其他游戏特定错误...
}

impl From<bincode::error::DecodeError> for GameError {
    fn from(err: bincode::error::DecodeError) -> Self {
        GameError::SerializationError(err.to_string())
    }
}
