use thiserror::Error;

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

impl From<bincode::Error> for GameError {
    fn from(err: bincode::Error) -> Self {
        GameError::SerializationError(err.to_string())
    }
}
