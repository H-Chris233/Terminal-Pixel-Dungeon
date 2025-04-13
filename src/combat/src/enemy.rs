
// src/combat/src/enemy/enemy.rs

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use rand::Rng;


/// 敌人实体，包含战斗属性和位置信息
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub exp_value: i32,
    pub x: i32,
    pub y: i32,
    pub state: EnemyState,
    pub attack_range: i32,
    pub detection_range: i32,
    pub symbol: char,       // 敌人显示字符
    pub color: (u8, u8, u8), // 敌人显示颜色(RGB)
}

/// 敌人种类，影响基础属性和行为
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub enum EnemyKind {
    Rat,
    Snake,
    Gnoll,
    Crab,
    Bat,
    Scorpion,
    Guard,
    Warlock,
    Golem,
}

// 为 EnemyKind 实现 Default
impl Default for EnemyKind {
    fn default() -> Self {
        EnemyKind::Rat // 或其他合理的默认值
    }
}

/// 敌人当前状态
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize, PartialEq)]
pub enum EnemyState {
    Idle,
    Alert,
    Hostile,
    Fleeing,
    Sleeping,
    Passive,
}

/// 掉落物品类型(简化版)
#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub enum DropItem {
    Gold(i32),
    HealthPotion,
    Weapon,
    Armor,
    Scroll,
    // 其他掉落物...
}

impl Enemy {
    /// 创建新敌人实例，属性根据破碎的像素地牢原作平衡
    pub fn new(kind: EnemyKind, x: i32, y: i32) -> Self {
        let (hp, attack, defense, exp_value, range, detection, symbol, color) = match kind {
            EnemyKind::Rat => (10, 4, 2, 2, 1, 5, 'r', (255, 150, 150)),
            EnemyKind::Snake => (12, 6, 3, 4, 1, 6, 's', (150, 255, 150)),
            EnemyKind::Gnoll => (20, 8, 5, 6, 1, 6, 'g', (200, 200, 100)),
            EnemyKind::Crab => (25, 5, 10, 5, 1, 4, 'c', (255, 100, 100)),
            EnemyKind::Bat => (15, 10, 4, 3, 1, 8, 'b', (200, 150, 255)),
            EnemyKind::Scorpion => (22, 12, 8, 8, 1, 5, 'S', (255, 100, 0)),
            EnemyKind::Guard => (30, 12, 10, 10, 1, 7, 'G', (100, 100, 255)),
            EnemyKind::Warlock => (18, 15, 5, 12, 3, 8, 'W', (255, 0, 255)),
            EnemyKind::Golem => (50, 18, 15, 15, 1, 4, 'M', (150, 150, 150)),
        };
        
        Self {
            kind,
            hp,
            max_hp: hp,
            attack,
            defense,
            exp_value,
            x,
            y,
            state: EnemyState::Idle,
            attack_range: range,
            detection_range: detection,
            symbol,
            color,
        }
    }
    
    /// 受到伤害
    pub fn take_damage(&mut self, amount: i32) {
        self.hp = (self.hp - amount).max(0);
    }
    
    /// 是否存活
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }
    
    /// 重置敌人状态
    pub fn reset(&mut self) {
        self.state = EnemyState::Idle;
    }
    
    /// 状态转换方法
    pub fn set_state(&mut self, new_state: EnemyState) {
        self.state = new_state;
    }
    
    /// 转换为敌对状态
    pub fn make_hostile(&mut self) {
        self.state = EnemyState::Hostile;
    }
    
    /// 转换为逃跑状态
    pub fn start_fleeing(&mut self) {
        self.state = EnemyState::Fleeing;
    }
    
    /// 转换为警戒状态
    pub fn alert(&mut self) {
        self.state = EnemyState::Alert;
    }
    
    /// 使用A*寻路算法计算下一步移动方向
    pub fn calculate_move(&self, target_x: i32, target_y: i32, obstacles: &[(i32, i32)]) -> Option<(i32, i32)> {
        // 简单实现：优先减少x或y距离
        let dx = (target_x - self.x).signum();
        let dy = (target_y - self.y).signum();
        
        // 检查移动是否有效(不穿过障碍物)
        let new_x = self.x + dx;
        let new_y = self.y + dy;
        
        if !obstacles.contains(&(new_x, new_y)) {
            Some((dx, dy))
        } else {
            // 尝试只移动x或y方向
            if !obstacles.contains(&(self.x + dx, self.y)) {
                Some((dx, 0))
            } else if !obstacles.contains(&(self.x, self.y + dy)) {
                Some((0, dy))
            } else {
                None
            }
        }
    }
    
    /// 执行移动
    pub fn perform_move(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }
    
    /// 敌人死亡时掉落物品
    pub fn drop_items(&self) -> Vec<DropItem> {
        let mut drops = Vec::new();
        
        // 基础掉落：金币
        let gold_amount = match self.kind {
            EnemyKind::Rat => 1..5,
            EnemyKind::Snake => 2..6,
            EnemyKind::Gnoll => 3..8,
            EnemyKind::Crab => 2..7,
            EnemyKind::Bat => 1..4,
            EnemyKind::Scorpion => 4..10,
            EnemyKind::Guard => 5..12,
            EnemyKind::Warlock => 8..15,
            EnemyKind::Golem => 10..20,
        };
        
        drops.push(DropItem::Gold(rand::rng().random_range(gold_amount)));
        
        // 稀有掉落
        let rare_drop_chance = match self.kind {
            EnemyKind::Rat => 0.05,
            EnemyKind::Snake => 0.08,
            EnemyKind::Gnoll => 0.1,
            EnemyKind::Crab => 0.07,
            EnemyKind::Bat => 0.15,  // 蝙蝠有更高几率掉血瓶
            EnemyKind::Scorpion => 0.12,
            EnemyKind::Guard => 0.2,
            EnemyKind::Warlock => 0.25,
            EnemyKind::Golem => 0.3,
        };
        
        if rand::random::<f32>() < rare_drop_chance {
            drops.push(match rand::random::<u8>() % 5 {
                0 => DropItem::HealthPotion,
                1 => DropItem::Weapon,
                2 => DropItem::Armor,
                3 => DropItem::Scroll,
                _ => DropItem::Gold(5), // 额外金币
            });
        }
        
        drops
    }
}

// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enemy_creation() {
        let rat = Enemy::new(EnemyKind::Rat, 0, 0);
        assert_eq!(rat.hp, 10);
        assert_eq!(rat.attack, 4);
        assert_eq!(rat.symbol, 'r');
        
        let golem = Enemy::new(EnemyKind::Golem, 0, 0);
        assert_eq!(golem.hp, 50);
        assert_eq!(golem.defense, 15);
    }
    
    #[test]
    fn test_state_transitions() {
        let mut enemy = Enemy::new(EnemyKind::Gnoll, 0, 0);
        assert_eq!(enemy.state, EnemyState::Idle);
        
        enemy.make_hostile();
        assert_eq!(enemy.state, EnemyState::Hostile);
        
        enemy.start_fleeing();
        assert_eq!(enemy.state, EnemyState::Fleeing);
    }
}
