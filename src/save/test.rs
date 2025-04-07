// src/save/tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let save_system = SaveSystem::new(temp_dir.path(), 3).unwrap();
        
        let mut hero = Hero::default();
        hero.name = "TestHero".to_string();
        
        let dungeon = Dungeon::generate(1).unwrap();
        let seed = 12345;
        
        let save_data = SaveData {
            metadata: SaveMetadata {
                timestamp: SystemTime::now(),
                dungeon_depth: 1,
                hero_name: hero.name.clone(),
                hero_class: "Warrior".to_string(),
                play_time: 0.0,
            },
            hero,
            dungeon,
            game_seed: seed,
        };
        
        // 测试保存和加载
        save_system.save_game(0, &save_data).unwrap();
        let loaded = save_system.load_game(0).unwrap();
        
        assert_eq!(loaded.metadata.hero_name, "TestHero");
        assert_eq!(loaded.game_seed, seed);
        
        // 测试列表和删除
        let saves = save_system.list_saves().unwrap();
        assert_eq!(saves.len(), 1);
        
        save_system.delete_save(0).unwrap();
        assert!(save_system.load_game(0).is_err());
    }

    #[test]
    fn test_auto_save() {
        let temp_dir = tempdir().unwrap();
        let save_system = SaveSystem::new(temp_dir.path(), 3).unwrap();
        
        let mut auto_save = AutoSave::new(
            save_system,
            std::time::Duration::from_secs(1) // 1秒间隔
        );
        
        let save_data = SaveData {
            // ...测试数据...
        };
        
        // 第一次应该保存
        assert!(auto_save.check_auto_save(&save_data).unwrap());
        
        // 立即再次检查不应该保存
        assert!(!auto_save.check_auto_save(&save_data).unwrap());
        
        // 等待1秒后应该再次保存
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert!(auto_save.check_auto_save(&save_data).unwrap());
    }
    #[test]
    fn test_item_serialization() {
        let item = Item::new_potion();
        let serialized = bincode::serialize(&item).unwrap();
        let deserialized: Item = bincode::deserialize(&serialized).unwrap();
        assert_eq!(item, deserialized);
    }
}
