use crate::ecs::{Inventory, Stats};
use hero::{Bag, Hero};

pub trait HeroAdapter {
    fn to_stats(&self) -> Stats;
}

pub trait StatsAdapter {
    fn to_hero(&self) -> Hero;
}

impl HeroAdapter for Hero {
    fn to_stats(&self) -> Stats {
        Stats {
            hp: self.hp,
            max_hp: self.max_hp,
            attack: self.base_attack,
            defense: self.base_defense,
            accuracy: 80,
            evasion: 20,
            level: self.level,
            experience: self.experience,
        }
    }
}

impl StatsAdapter for Stats {
    fn to_hero(&self) -> Hero {
        let mut hero = Hero::with_seed(hero::class::Class::Warrior, 12345);
        hero.hp = self.hp;
        hero.max_hp = self.max_hp;
        hero.base_attack = self.attack;
        hero.base_defense = self.defense;
        hero.level = self.level;
        hero.experience = self.experience;
        hero
    }
}

pub trait InventoryAdapter {
    fn to_bag(&self) -> Bag;
}

pub trait BagAdapter {
    fn to_inventory(&self) -> Inventory;
}

impl InventoryAdapter for Inventory {
    fn to_bag(&self) -> Bag {
        hero::Bag::from(self)
    }
}

impl BagAdapter for Bag {
    fn to_inventory(&self) -> Inventory {
        Inventory::from(self)
    }
}
