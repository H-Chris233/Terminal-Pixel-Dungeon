use dungeon::Dungeon;
use hecs::Entity;
use hero::Hero;

/// 游戏状态管理器
pub struct GameState {
    /// 当前地牢
    pub dungeon: Dungeon,
    /// 玩家角色
    pub hero: Hero,
    /// 玩家实体
    pub player_entity: Option<Entity>,
}

impl GameState {
    pub fn new() -> Self {
        // 生成默认地牢和英雄
        let dungeon = Dungeon::generate(1, 12345).unwrap(); // 使用默认深度和种子
        let hero = Hero::new(hero::class::Class::Warrior); // 默认使用战士职业

        Self {
            dungeon,
            hero,
            player_entity: None,
        }
    }

    /// 获取玩家实体
    pub fn get_player_entity(&self) -> Option<Entity> {
        self.player_entity
    }

    /// 设置玩家实体
    pub fn set_player_entity(&mut self, entity: Entity) {
        self.player_entity = Some(entity);
    }
}
