
// src/save.rs
use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};
use std::{
    fs,
    path::{Path, PathBuf},
    io::{self, Write},
    time::SystemTime
};
use crate::error::error::GameError;
use crate::hero::hero::Hero;
use crate::dungeon::dungeon::Dungeon;


/// 存档元数据
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub timestamp: SystemTime,
    pub dungeon_depth: usize,
    pub hero_name: String,
    pub hero_class: String,
    pub play_time: f64, // 游戏时长(秒)
}

/// 存档数据(包含游戏完整状态)
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub metadata: SaveMetadata,
    pub hero: crate::hero::Hero,
    pub dungeon: crate::dungeon::Dungeon,
    pub game_seed: u64, // 用于重现随机地牢
}

/// 存档系统
pub struct SaveSystem {
    save_dir: PathBuf,
    max_slots: usize,
}

impl SaveSystem {
    /// 初始化存档系统
    pub fn new(save_dir: impl AsRef<Path>, max_slots: usize) -> Result<Self, GameError> {
        let save_dir = save_dir.as_ref();
        
        // 创建存档目录(如果不存在)
        if !save_dir.exists() {
            fs::create_dir_all(save_dir)
                .context("Failed to create save directory")?;
        }

        Ok(Self {
            save_dir: save_dir.to_path_buf(),
            max_slots,
        })
    }

    /// 获取所有存档列表(按时间倒序)
    pub fn list_saves(&self) -> Result<Vec<SaveMetadata>, GameError> {
        let mut saves = Vec::new();
        
        for entry in fs::read_dir(&self.save_dir)
            .context("Failed to read save directory")? 
        {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "sav") {
                let file = fs::File::open(&path)
                    .context(format!("Failed to open save file: {:?}", path))?;
                
                let metadata: SaveMetadata = bincode::deserialize_from(file)
                    .context(format!("Failed to parse save metadata: {:?}", path))?;
                
                saves.push(metadata);
            }
        }
        
        // 按时间戳排序(最新的在前)
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(saves)
    }

    /// 保存游戏状态
    pub fn save_game(&self, slot: usize, data: &SaveData) -> Result<(), GameError> {
        if slot >= self.max_slots {
            return Err(anyhow::anyhow!("Invalid save slot"));
        }
        
        let filename = format!("save_{}.sav", slot);
        let path = self.save_dir.join(filename);
        
        // 创建临时文件
        let temp_path = path.with_extension("tmp");
        let mut file = fs::File::create(&temp_path)
            .context("Failed to create temporary save file")?;
        
        // 序列化数据
        bincode::serialize_into(&mut file, data)
            .context("Failed to serialize save data")?;
        
        // 确保数据写入磁盘
        file.flush()
            .context("Failed to flush save data")?;
        
        // 原子性重命名
        fs::rename(temp_path, path)
            .context("Failed to commit save file")?;
        
        Ok(())
    }

    /// 加载游戏状态
    pub fn load_game(&self, slot: usize) -> Result<SaveData, GameError> {
        if slot >= self.max_slots {
            return Err(anyhow::anyhow!("Invalid save slot"));
        }
        
        let filename = format!("save_{}.sav", slot);
        let path = self.save_dir.join(filename);
        
        let file = fs::File::open(&path)
            .context(format!("Save file not found: {:?}", path))?;
        
        let data: SaveData = bincode::deserialize_from(file)
            .context("Failed to deserialize save data")?;
        
        Ok(data)
    }

    /// 删除存档
    pub fn delete_save(&self, slot: usize) -> Result<(), GameError> {
        if slot >= self.max_slots {
            return Err(anyhow::anyhow!("Invalid save slot"));
        }
        
        let filename = format!("save_{}.sav", slot);
        let path = self.save_dir.join(filename);
        
        if path.exists() {
            fs::remove_file(path)
                .context("Failed to delete save file")?;
        }
        
        Ok(())
    }

    /// 获取存档目录路径
    pub fn save_dir(&self) -> &Path {
        &self.save_dir
    }
}

/// 自动保存功能
pub struct AutoSave {
    save_system: SaveSystem,
    interval: std::time::Duration,
    last_save: Option<SystemTime>,
}

impl AutoSave {
    pub fn new(save_system: SaveSystem, interval: std::time::Duration) -> Self {
        Self {
            save_system,
            interval,
            last_save: None,
        }
    }

    /// 检查是否需要自动保存
    pub fn check_auto_save(&mut self, game_data: &SaveData) -> Result<bool> {
        let now = SystemTime::now();
        let should_save = match self.last_save {
            Some(last) => now.duration_since(last)? >= self.interval,
            None => true,
        };

        if should_save {
            self.save_system.save_game(0, game_data)?;
            self.last_save = Some(now);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
