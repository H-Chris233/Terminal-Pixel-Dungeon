use crate::{
    bag::Bag,
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
            ItemCategory::Potion => self.use_potion(index),
            ItemCategory::Scroll => self.use_scroll(index),
            ItemCategory::Weapon => self.equip_weapon(index),
            ItemCategory::Armor => self.equip_armor(index),
            ItemCategory::Ring => self.equip_ring(index),
            _ => Err(HeroError::UnusableItem),
        }
    }

    /// 药水使用逻辑
    fn use_potion(&mut self, index: usize) -> Result<(), HeroError> {
        let potion = self
            .bag
            .potions()
            .get(index)
            .ok_or(HeroError::InvalidIndex)?;

        if !potion.identified {
            self.notify("你喝下了未知的药水...".into());
            if self.rng.gen_bool(0.5) {
                return Err(HeroError::IdentifyFailed);
            }
        }

        match potion.kind {
            PotionKind::Healing => self.heal(self.max_hp / 3),
            PotionKind::Strength => self.strength += 1,
            PotionKind::MindVision => self.effects.add(Effect::new(
                EffectType::MindVision,
                20,
                "获得灵视效果".into(),
            )),
            PotionKind::ToxicGas => {
                // 对周围敌人造成中毒效果
                dungeon::affect_adjacent_enemies(self.x, self.y, |e| {
                    e.add_effect(Effect::poison(2, 10));
                });
            }
            PotionKind::Frost => {
                // 冰冻周围敌人
                dungeon::affect_adjacent_enemies(self.x, self.y, |e| {
                    e.add_effect(Effect::new(EffectType::Frozen, 5, "被冰冻".into()));
                });
            }
        }

        self.bag.remove_potion(index)?;
        Ok(())
    }

    /// 使用卷轴
    fn use_scroll(&mut self, index: usize) -> Result<(), HeroError> {
        let scroll = self
            .bag
            .scrolls()
            .get(index)
            .ok_or(HeroError::InvalidIndex)?;

        if !scroll.identified {
            self.notify("你阅读了未知的卷轴...".into());
            if self.rng.gen_bool(0.5) {
                scroll.identify(&mut self.rng);
            } else {
                return Err(HeroError::IdentifyFailed);
            }
        }

        match scroll.kind {
            ScrollKind::Upgrade => {
                if let Some(weapon) = self.bag.equipment().weapon.as_mut() {
                    weapon.upgrade();
                    self.notify(format!("你的{}变得更锋利了！", weapon.name));
                } else {
                    return Err(HeroError::UnusableItem);
                }
            }
            ScrollKind::RemoveCurse => {
                self.bag.remove_cursed_items();
                self.notify("一股净化之力扫过你的装备".into());
            }
            ScrollKind::MagicMapping => {
                dungeon::reveal_current_level(self.x, self.y);
                self.notify("你的脑海中浮现出这一层的地图".into());
            }
        }

        self.bag.remove_scroll(index)?;
        Ok(())
    }

    /// 装备武器
    fn equip_weapon(&mut self, index: usize) -> Result<(), HeroError> {
        let weapon = self
            .bag
            .weapons()
            .get(index)
            .ok_or(HeroError::InvalidIndex)?;

        if weapon.str_requirement > self.strength {
            return Err(HeroError::Underpowered);
        }

        let old_weapon = self.bag.equip_weapon(index)?;
        if let Some(w) = old_weapon {
            self.bag
                .add_weapon(w)
                .map_err(|_| HeroError::InventoryFull)?;
        }

        Ok(())
    }

    /// 装备护甲
    fn equip_armor(&mut self, index: usize) -> Result<(), HeroError> {
        let armor = self
            .bag
            .armors()
            .get(index)
            .ok_or(HeroError::InvalidIndex)?;

        if armor.str_requirement > self.strength {
            return Err(HeroError::Underpowered);
        }

        let old_armor = self.bag.equip_armor(index)?;
        if let Some(a) = old_armor {
            self.bag
                .add_armor(a)
                .map_err(|_| HeroError::InventoryFull)?;
        }

        Ok(())
    }

    /// 装备戒指
    fn equip_ring(&mut self, index: usize) -> Result<(), HeroError> {
        let ring = self.bag.rings().get(index).ok_or(HeroError::InvalidIndex)?;

        if self
            .bag
            .equipment()
            .rings
            .iter()
            .filter(|r| r.is_some())
            .count()
            >= 2
        {
            return Err(HeroError::InventoryFull); // 戒指槽已满
        }

        let old_ring = self.bag.equip_ring(index)?;
        if let Some(r) = old_ring {
            self.bag.add_ring(r).map_err(|_| HeroError::InventoryFull)?;
        }

        Ok(())
    }
}

impl InventorySystem for Hero {
    fn add_item(&mut self, item: Item) -> Result<(), BagError> {
        self.bag.add_item(item)
    }

    fn remove_item(&mut self, index: usize) -> Result<(), BagError> {
        self.bag.remove_item(index)
    }

    fn equip_item(&mut self, index: usize, strength: u8) -> Result<Option<Item>, BagError> {
        self.bag.equip_item(index, strength)
    }

    fn use_item(&mut self, index: usize) -> Result<Item, BagError> {
        self.bag.use_item(index)
    }
}
