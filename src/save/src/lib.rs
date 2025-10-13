// src/save.rs

use error::GameError;
use anyhow::{Context, Result};
use bincode::{config, Decode, Encode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// 存档元数据
#[derive(Debug, Encode, Decode, Serialize, Deserialize)] // 添加Encode和Decode派生
pub struct SaveMetadata {
    pub timestamp: SystemTime,
    pub dungeon_depth: usize,
    pub hero_name: String,
    pub hero_class: String,
    pub play_time: f64, // 游戏时长(秒)
}

/// 存档数据(包含游戏完整状态)
#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct SaveData {
    pub metadata: SaveMetadata,
    pub hero: hero::Hero,
    pub dungeon: dungeon::Dungeon,
    pub game_seed: u64,
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
            fs::create_dir_all(save_dir).context("Failed to create save directory")?;
        }

        Ok(Self {
            save_dir: save_dir.to_path_buf(),
            max_slots,
        })
    }

    /// 获取所有存档列表(按时间倒序)
    pub fn list_saves(&self) -> Result<Vec<SaveMetadata>, GameError> {
        let mut saves = Vec::new();

        // 读取存档目录
        let entries = fs::read_dir(&self.save_dir).context("Failed to read save directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // 检查是否是.sav文件
            if path.is_file() && path.extension().map_or(false, |ext| ext == "sav") {
                // 打开文件
                let mut file = fs::File::open(&path)
                    .context(format!("Failed to open save file: {:?}", path))?;

                // 反序列化数据
                let config = bincode::config::standard();
                let data: SaveData = bincode::decode_from_std_read(&mut file, config)
                    .context(format!("Failed to deserialize save file: {:?}", path))?;

                // 添加元数据到列表
                saves.push(data.metadata);
            }
        }

        // 按时间戳排序(最新的在前)
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(saves)
    }

    /// 保存游戏状态
    pub fn save_game(&self, slot: usize, data: &SaveData) -> Result<()> {
        if slot >= self.max_slots {
            return Err(anyhow::anyhow!("Invalid save slot"));
        }

        let filename = format!("save_{}.sav", slot);
        let path = self.save_dir.join(filename);

        // 创建临时文件
        let temp_path = path.with_extension("tmp");
        let mut file =
            fs::File::create(&temp_path).context("Failed to create temporary save file")?;

        // 序列化数据
        let config = config::standard();
        bincode::encode_into_std_write(data, &mut file, config)
            .context("Failed to serialize save data")?;

        // 确保数据写入磁盘
        file.flush().context("Failed to flush save data")?;

        // 原子性重命名
        fs::rename(temp_path, path).context("Failed to commit save file")?;

        Ok(())
    }

    /// 加载游戏状态
    pub fn load_game(&self, slot: usize) -> Result<SaveData> {
        if slot >= self.max_slots {
            return Err(anyhow::anyhow!("Invalid save slot"));
        }

        let filename = format!("save_{}.sav", slot);
        let path = self.save_dir.join(filename);

        let mut file = fs::File::open(&path).context(format!("Save file not found: {:?}", path))?;

        let config = config::standard();
        let data: SaveData = bincode::decode_from_std_read(&mut file, config)
            .context("Failed to deserialize save data")?;

        Ok(data)
    }

    /// 删除存档
    pub fn delete_save(&self, slot: usize) -> Result<()> {
        if slot >= self.max_slots {
            return Err(anyhow::anyhow!("Invalid save slot"));
        }

        let filename = format!("save_{}.sav", slot);
        let path = self.save_dir.join(filename);

        if path.exists() {
            fs::remove_file(path).context("Failed to delete save file")?;
        }

        Ok(())
    }

    /// 获取存档目录路径
    pub fn save_dir(&self) -> &Path {
        &self.save_dir
    }
    pub fn max_slots(&self) -> usize {
        self.max_slots
    }

    /// 检查指定槽位是否有存档
    pub fn has_save(&self, slot: usize) -> bool {
        if slot >= self.max_slots {
            return false;
        }
        let filename = format!("save_{}.sav", slot);
        self.save_dir.join(filename).exists()
    }

    /// 获取存档文件路径
    pub fn save_path(&self, slot: usize) -> Option<PathBuf> {
        if slot >= self.max_slots {
            return None;
        }
        let filename = format!("save_{}.sav", slot);
        Some(self.save_dir.join(filename))
    }
}

/// 自动保存功能
pub struct AutoSave {
    pub save_system: SaveSystem,
    pub interval: std::time::Duration,
    pub last_save: Option<SystemTime>,
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
    pub fn try_save(&mut self, save_data: &SaveData) -> Result<()> {
        self.check_auto_save(save_data)?;
        Ok(())
    }

    /// 强制立即保存（忽略自动保存间隔）
    pub fn force_save(&mut self, save_data: &SaveData) -> Result<()> {
        self.save_system.save_game(0, save_data)?;
        self.last_save = Some(SystemTime::now());
        Ok(())
    }

    /// 获取上次保存时间
    pub fn last_save_time(&self) -> Option<SystemTime> {
        self.last_save
    }

    /// 获取自动保存间隔
    pub fn save_interval(&self) -> Duration {
        self.interval
    }

    /// 设置新的自动保存间隔
    pub fn set_save_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }
}
