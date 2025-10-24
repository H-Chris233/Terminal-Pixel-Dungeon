//src/hero/src/core/item.rs
use crate::{
    bag::BagError,
    effects::{Effect, EffectType},
};
use items::{
    Item, ItemCategory, ItemKind,
    herb::HerbKind,
    potion::PotionKind,
    scroll::ScrollKind,
    seed::{Seed, SeedKind},
};

use crate::Hero;

use crate::HeroError;
use crate::InventorySystem;

impl Hero {
    /// 使用物品
    pub fn use_item(&mut self, category: ItemCategory, index: usize) -> Result<(), HeroError> {
        match category {
            ItemCategory::Potion => {
                self.use_potion(index)?;
                Ok(())
            }
            ItemCategory::Scroll => {
                self.use_scroll(index)?;
                Ok(())
            }
            ItemCategory::Herb => {
                self.use_herb(index)?;
                Ok(())
            }
            ItemCategory::Throwable => {
                self.use_throwable(index)?;
                Ok(())
            }
            ItemCategory::Weapon => {
                let _ = self.equip_item(index);
                Ok(())
            }
            ItemCategory::Armor => {
                let _ = self.equip_item(index);
                Ok(())
            }
            ItemCategory::Ring => {
                let _ = self.equip_item(index);
                Ok(())
            }
            _ => Err(HeroError::BagFull(BagError::CannotUseItem)),
        }
    }

    /// 药水使用逻辑
    fn use_potion(&mut self, index: usize) -> Result<(), HeroError> {
        let mut item = self
            .bag
            .use_item(index)
            .map_err(|_| HeroError::ActionFailed)?;
        if let items::ItemKind::Potion(ref mut potion) = item.kind {
            if !potion.identified {
                self.notify("你喝下了未知的药水...");
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
                    // dungeon::affect_adjacent_enemies(self.x, self.y, |e| {
                    //     e.add_effect(Effect::new(EffectType::Poison, 10));
                    // });
                }
                PotionKind::Frost => {
                    // dungeon::affect_adjacent_enemies(self.x, self.y, |e| {
                    //     e.add_effect(Effect::new(EffectType::Frost, 5));
                    // });
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
                    self.bag
                        .upgrade_weapon()
                        .map_err(|_| HeroError::ActionFailed)?;
                }
                ScrollKind::RemoveCurse => {
                    self.bag.remove_curse_all();
                    self.notify("一股净化之力扫过你的装备");
                }
                ScrollKind::MagicMapping => {
                    // dungeon::reveal_current_level(self.x, self.y);
                    self.notify("你的脑海中浮现出这一层的地图");
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn use_herb(&mut self, index: usize) -> Result<(), HeroError> {
        let preview_item = self
            .bag
            .get_item_by_index(index)
            .map_err(|_| HeroError::ActionFailed)?;

        let (herb_kind, herb_name) = if let items::ItemKind::Herb(ref herb) = preview_item.kind {
            (herb.kind, herb.name())
        } else {
            return Err(HeroError::ActionFailed);
        };

        let recipe_seed = match herb_kind {
            HerbKind::Sungrass => Some(SeedKind::Earthroot),
            HerbKind::Moonleaf => Some(SeedKind::Fadeleaf),
            HerbKind::Nightshade => Some(SeedKind::Sorrowmoss),
            HerbKind::SpiritMoss => Some(SeedKind::Dreamfoil),
            HerbKind::Dragonthorn => Some(SeedKind::Stormvine),
            HerbKind::Glowcap => Some(SeedKind::Icecap),
        };

        if let Some(seed_kind) = recipe_seed {
            if let Ok(potion_item) = self.bag.combine_reagents(herb_kind, seed_kind) {
                if let ItemKind::Potion(ref potion) = potion_item.kind {
                    let seed_name = Seed::new(seed_kind).name();
                    self.notify(&format!(
                        "你将{}与{}炼制出了{}。",
                        herb_name,
                        seed_name,
                        potion.name()
                    ));
                }
                return Ok(());
            }
        }

        let item = self
            .bag
            .use_item(index)
            .map_err(|_| HeroError::ActionFailed)?;

        if let items::ItemKind::Herb(herb) = item.kind {
            let potency = herb.potency as u32;
            match herb.kind {
                HerbKind::Sungrass => {
                    self.heal(potency * 5);
                    self.notify("日光草的温暖治愈了你的伤势。");
                }
                HerbKind::Moonleaf => {
                    self.effects
                        .add(Effect::new(EffectType::Invisibility, 8 + potency));
                    self.notify("月影叶让你融入了阴影。");
                }
                HerbKind::Nightshade => {
                    self.effects
                        .add(Effect::new(EffectType::Poison, 4 + potency));
                    self.notify("夜影花的毒素在你的血液中流淌！");
                }
                HerbKind::SpiritMoss => {
                    self.effects.remove(EffectType::Poison);
                    self.effects.remove(EffectType::Burning);
                    self.heal(10 + potency * 2);
                    self.notify("灵魂苔净化了你的身体。");
                }
                HerbKind::Dragonthorn => {
                    self.effects
                        .add(Effect::new(EffectType::Haste, 6 + potency));
                    self.notify("龙棘草让你的动作变得更加迅捷。");
                }
                HerbKind::Glowcap => {
                    self.effects
                        .add(Effect::new(EffectType::Light, 12 + potency));
                    self.notify("萤帽菌的微光照亮了前方道路。");
                }
            }

            let satiety_gain = match herb.kind {
                HerbKind::Nightshade => 0,
                _ => herb.potency.min(3),
            };
            self.satiety = self.satiety.saturating_add(satiety_gain).min(10);
            Ok(())
        } else {
            Err(HeroError::ActionFailed)
        }
    }

    fn use_throwable(&mut self, index: usize) -> Result<(), HeroError> {
        let item = self
            .bag
            .use_item(index)
            .map_err(|_| HeroError::ActionFailed)?;

        if let items::ItemKind::Throwable(throwable) = item.kind {
            let haste_turns = (throwable.range as u32).max(2) + 2;
            self.effects
                .add(Effect::new(EffectType::Haste, haste_turns));
            self.satiety = self.satiety.saturating_sub(1);
            self.notify(&format!(
                "你熟练地练习投掷{}，身手更加敏捷。",
                throwable.name()
            ));
            Ok(())
        } else {
            Err(HeroError::ActionFailed)
        }
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
