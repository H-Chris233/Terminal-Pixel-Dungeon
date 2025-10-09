//src/hero/src/core/item.rs
use crate::{
    bag::{Bag, BagError},
    class::Class,
    effects::{Effect, EffectManager, EffectType},
    rng::HeroRng,
};
use combat::Combatant;
use dungeon::trap::Trap;
use dungeon::trap::TrapEffect;
use dungeon::Dungeon;
use items::scroll::ScrollKind;
use combat::enemy::Enemy;
use items::{potion::PotionKind, Item, ItemCategory};
use thiserror::Error;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::Hero;
use crate::HeroBehavior;
use crate::HeroError;
use crate::InventorySystem;

impl Hero {
    /// 使用物品
    pub fn use_item(&mut self, category: ItemCategory, index: usize) -> Result<(), HeroError> {
        match category {
            ItemCategory::Potion => {
                self.use_potion(index);
                Ok(())
            }
            ItemCategory::Scroll => {
                self.use_scroll(index);
                Ok(())
            }
            ItemCategory::Weapon => {
                self.equip_item(index);
                Ok(())
            }
            ItemCategory::Armor => {
                self.equip_item(index);
                Ok(())
            }
            ItemCategory::Ring => {
                self.equip_item(index);
                Ok(())
            }
            _ => Err(HeroError::BagFull(BagError::CannotUseItem)),
        }
    }

    /// 药水使用逻辑
    fn use_potion(&mut self, index: usize) -> Result<(), HeroError> {
        let mut item = self.bag.use_item(index).map_err(|_| HeroError::ActionFailed)?;
        if let items::ItemKind::Potion(ref mut potion) = item.kind {
            if !potion.identified {
                self.notify("你喝下了未知的药水...".into());
                potion.identify();
                self.notify(&format!("这是一瓶...{}!", potion.name()));
            }

            match potion.kind {
            PotionKind::Healing => self.heal(self.max_hp / 3),
            PotionKind::Strength => self.strength += 1,
            PotionKind::MindVision => {
                self.effects.add(Effect::new(EffectType::MindVision, 20));
            }
            PotionKind::ToxicGas => {
                dungeon::affect_adjacent_enemies(self.x, self.y, |e| {
                    e.add_effect(Effect::new(EffectType::Poison, 10));
                });
            }
            PotionKind::Frost => {
                dungeon::affect_adjacent_enemies(self.x, self.y, |e| {
                    e.add_effect(Effect::new(EffectType::Frost, 5));
                });
            }
            _ => {}
        }

        }
        Ok(())
    }

    /// 使用卷轴
    fn use_scroll(&mut self, index: usize) -> Result<(), HeroError> {
        let mut item = self.bag.use_item(index)?;
        if let items::ItemKind::Scroll(ref mut scroll) = item.kind {
            if !scroll.identified {
                self.notify("你阅读了未知的卷轴...");
                scroll.identify();
                self.notify(&format!("这是一张...{}!", scroll.name()));
            }

            match scroll.kind {
                ScrollKind::Upgrade => {
                    self.bag.upgrade_weapon().map_err(|_| HeroError::ActionFailed)?;
                }
                ScrollKind::RemoveCurse => {
                    self.bag.remove_curse_all();
                    self.notify("一股净化之力扫过你的装备".into());
                }
                ScrollKind::MagicMapping => {
                    dungeon::reveal_current_level(self.x, self.y);
                    self.notify("你的脑海中浮现出这一层的地图".into());
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl InventorySystem for Hero {
    fn add_item(&mut self, item: Item) -> Result<(), HeroError> {
        self.bag.add_item(item).map_err(|e| e.into())
    }

    fn remove_item(&mut self, index: usize) -> Result<(), HeroError> {
        self.bag.remove_item(index).map_err(|e| e.into()) 
    }

    fn equip_item(&mut self, index: usize) -> Result<Option<Item>, HeroError> {
        self.bag
            .equip_item(index, self.strength)
            .map_err(|e| e.into())
    }

    fn use_item(&mut self, index: usize) -> Result<Item, HeroError> {
        self.bag.use_item(index).map_err(|e| e.into())
    }
}
