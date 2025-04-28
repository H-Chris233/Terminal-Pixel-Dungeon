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
use items::{potion::PotionKind, Item, ItemCategory};
use thiserror::Error;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::BagError;
use crate::EffectSystem;
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
        let potion = self
            .bag
            .potions()
            .get(index)
            .ok_or(BagError::InvalidIndex)?;

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
                // 对周围敌人造成中毒效果
                dungeon::affect_adjacent_enemies(self.x, self.y, |e| {
                    e.add_effect(Effect::new(EffectType::Poison, 10));
                });
            }
            PotionKind::Frost => {
                // 冰冻周围敌人
                dungeon::affect_adjacent_enemies(self.x, self.y, |e| {
                    e.add_effect(Effect::new(EffectType::Frost, 5));
                });
            }
        }

        self.bag.remove_item(index)?;
        Ok(())
    }

    /// 使用卷轴
    fn use_scroll(&mut self, index: usize) -> Result<(), HeroError> {
        let scroll = self
            .bag
            .scrolls()
            .get(index)
            .ok_or(BagError::InvalidIndex)?;

        if !scroll.identified {
            self.notify("你阅读了未知的卷轴...");
            scroll.identify();
            self.notify(&format!("这是一张...{}!", scroll.name()));
        }

        match scroll.kind {
            ScrollKind::Upgrade => {
                if let Some(weapon) = self.bag.equipment().weapon.as_mut() {
                    weapon.upgrade();
                    self.notify(&format!("你的{}变得更锋利了！", weapon.name));
                } else {
                    return Err(HeroError::ActionFailed);
                }
            }
            ScrollKind::RemoveCurse => {
                self.bag.remove_curse_all();
                self.notify("一股净化之力扫过你的装备".into());
            }
            ScrollKind::MagicMapping => {
                dungeon::reveal_current_level(self.x, self.y);
                self.notify("你的脑海中浮现出这一层的地图".into());
            }
        }

        self.bag.remove_item(index)?;
        Ok(())
    }
}

impl InventorySystem for Hero {
    pub fn add_item(&mut self, item: Item) -> Result<(), BagError> {
        self.bag.add_item(item)
    }

    pub fn remove_item(&mut self, index: usize) -> Result<(), BagError> {
        self.bag.remove_item(index)
    }

    pub fn equip_item(&mut self, index: usize) -> Result<Option<Item>, HeroError> {
        self.bag
            .equip_item(index, self.strength)
            .map_err(|e| e.into())
    }

    pub fn use_item(&mut self, index: usize) -> Result<Item, BagError> {
        self.bag.use_item(index)
    }
}
