use crate::hero::hero::Hero;
use crate::dungeon::dungeon::Enemy;

// src/combat.rs
pub struct Combat;

impl Combat {
    pub fn engage(hero: &mut Hero, enemy: &mut Enemy) {
        // 英雄攻击敌人
        let hero_damage = hero.attack - enemy.defense / 2;
        enemy.hp -= hero_damage.max(1);
        
        if enemy.hp > 0 {
            // 敌人反击
            let enemy_damage = enemy.attack - hero.defense / 2;
            hero.hp -= enemy_damage.max(1);
        } else {
            hero.gain_exp(enemy.exp_value);
        }
    }
}
