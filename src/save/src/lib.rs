// src/save.rs

use anyhow::{Context, Result};
use bincode::{Decode, Encode, config};
use error::GameError;
use hero::class::{Class, SkillState};
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
    pub hero_class: Class,
    pub play_time: f64, // 游戏时长(秒)
}

/// Turn scheduler state for save/load
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct TurnStateData {
    /// Current turn phase (PlayerTurn, AITurn, etc.)
    pub current_phase: TurnPhase,
    /// Whether player has taken an action this turn
    pub player_action_taken: bool,
}

impl Default for TurnStateData {
    fn default() -> Self {
        Self {
            current_phase: TurnPhase::PlayerTurn,
            player_action_taken: false,
        }
    }
}

/// Turn phase for serialization
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq)]
pub enum TurnPhase {
    PlayerTurn,
    ProcessingPlayerAction,
    AITurn,
    ProcessingAIActions,
}

impl Default for TurnPhase {
    fn default() -> Self {
        TurnPhase::PlayerTurn
    }
}

/// Entity state for non-player entities (enemies, etc.)
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct EntityStateData {
    pub position: (i32, i32, i32), // x, y, z
    pub name: String,
    pub hp: u32,
    pub max_hp: u32,
    pub energy_current: u32,
    pub energy_max: u32,
    pub energy_regen: u32,
    pub active_effects: Vec<StatusEffectData>,
}

/// Serializable status effect data
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct StatusEffectData {
    pub effect_type: String,
    pub duration: u32,
    pub intensity: u32,
}

/// Game clock state
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct ClockStateData {
    pub turn_count: u32,
    pub elapsed_time_secs: f64,
}

impl Default for ClockStateData {
    fn default() -> Self {
        Self {
            turn_count: 0,
            elapsed_time_secs: 0.0,
        }
    }
}

/// 存档数据(包含游戏完整状态)
#[derive(Debug, Encode, Decode, Serialize, Deserialize)]
pub struct SaveData {
    /// Version for backward compatibility
    #[serde(default = "default_version")]
    pub version: u32,
    
    pub metadata: SaveMetadata,
    
    #[serde(default)]
    pub hero_skill_state: SkillState,
    
    pub hero: hero::Hero,
    pub dungeon: dungeon::Dungeon,
    pub game_seed: u64,
    
    /// Turn system state (v2+)
    #[serde(default)]
    pub turn_state: TurnStateData,
    
    /// Game clock state (v2+)
    #[serde(default)]
    pub clock_state: ClockStateData,
    
    /// Player energy state (v2+)
    #[serde(default = "default_player_energy")]
    pub player_energy: u32,
    
    /// Player hunger last turn (v2+)
    #[serde(default)]
    pub player_hunger_last_turn: u32,
    
    /// Non-player entity states (v2+)
    #[serde(default)]
    pub entities: Vec<EntityStateData>,
}

fn default_player_energy() -> u32 {
    100 // Default to full energy for legacy saves
}

/// Current save format version
pub const SAVE_VERSION: u32 = 2;

fn default_version() -> u32 {
    1 // Legacy saves default to version 1
}

impl SaveData {
    /// Migrate legacy save data to current version
    pub fn migrate(&mut self) {
        if self.version < SAVE_VERSION {
            match self.version {
                1 => {
                    // Migrate from v1 to v2: Initialize turn state and clock state
                    // These fields already have defaults, but we ensure they're set
                    if self.turn_state.current_phase == TurnPhase::PlayerTurn 
                        && !self.turn_state.player_action_taken {
                        // Already at default, no action needed
                    }
                    
                    if self.clock_state.turn_count == 0 {
                        // Initialize from hero turns if available
                        self.clock_state.turn_count = self.hero.turns;
                    }
                    
                    self.version = 2;
                }
                _ => {
                    // Unknown version, skip migration
                }
            }
        }
    }
    
    /// Validate save data integrity
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.metadata.dungeon_depth == 0 {
            return Err(anyhow::anyhow!("Invalid dungeon depth"));
        }
        
        if self.hero.max_hp == 0 {
            return Err(anyhow::anyhow!("Invalid hero HP"));
        }
        
        Ok(())
    }
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
        let mut data: SaveData = bincode::decode_from_std_read(&mut file, config)
            .context("Failed to deserialize save data")?;

        // Migrate legacy saves to current version
        data.migrate();
        
        // Validate save data integrity
        data.validate().context("Save data validation failed")?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use bincode::config;
    use hero::class::Class;
    use hero::Hero;

    #[test]
    fn save_data_roundtrip_preserves_class_and_skills() {
        let mut hero = Hero::with_seed(Class::Mage, 99);
        hero.name = "Archivist".to_string();
        hero.class_skills
            .unlocked_talents
            .push("arcane_focus".to_string());
        hero.class_skills.active_skill = Some("fireball".to_string());

        let metadata = SaveMetadata {
            timestamp: SystemTime::now(),
            dungeon_depth: 3,
            hero_name: hero.name.clone(),
            hero_class: hero.class.clone(),
            play_time: 128.5,
        };

        let hero_skill_state = hero.class_skills.clone();
        let dungeon = dungeon::Dungeon::generate(1, 4242).expect("generate dungeon");

        let save_data = SaveData {
            version: SAVE_VERSION,
            metadata,
            hero_skill_state: hero_skill_state.clone(),
            hero,
            dungeon,
            game_seed: 4242,
            turn_state: TurnStateData::default(),
            clock_state: ClockStateData {
                turn_count: 42,
                elapsed_time_secs: 128.5,
            },
            player_energy: 75,
            player_hunger_last_turn: 20,
            entities: vec![],
        };

        let cfg = config::standard();
        let encoded = bincode::encode_to_vec(&save_data, cfg).expect("serialize save data");
        let (decoded, _) = bincode::decode_from_slice::<SaveData, _>(&encoded, cfg)
            .expect("deserialize save data");

        assert_eq!(decoded.metadata.hero_class, Class::Mage);
        assert_eq!(decoded.hero.class, Class::Mage);
        assert_eq!(decoded.hero_skill_state, hero_skill_state);
        assert_eq!(decoded.hero.class_skills, hero_skill_state);
        assert_eq!(decoded.version, SAVE_VERSION);
        assert_eq!(decoded.clock_state.turn_count, 42);
        assert_eq!(decoded.clock_state.elapsed_time_secs, 128.5);
        assert_eq!(decoded.turn_state.current_phase, TurnPhase::PlayerTurn);
        assert_eq!(decoded.player_energy, 75);
    }
}
