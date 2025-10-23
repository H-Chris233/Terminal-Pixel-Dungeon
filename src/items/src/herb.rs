use bincode::serde::encode_to_vec;
use bincode::{Decode, Encode};
use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hasher;

use crate::BINCODE_CONFIG;
use crate::Item;
use crate::ItemCategory;
use crate::ItemKind;
use crate::ItemRarity;
use crate::ItemTrait;

/// 药草系统，支持调配与炼金
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Herb {
    pub kind: HerbKind,
    pub potency: u8,
    pub quantity: u32,
    pub identified: bool,
}

#[derive(Copy, Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum HerbKind {
    Sungrass,
    Moonleaf,
    Nightshade,
    SpiritMoss,
    Dragonthorn,
    Glowcap,
}

impl Herb {
    pub fn new(kind: HerbKind) -> Self {
        let potency = match kind {
            HerbKind::Sungrass => 2,
            HerbKind::Moonleaf => 3,
            HerbKind::Nightshade => 4,
            HerbKind::SpiritMoss => 1,
            HerbKind::Dragonthorn => 5,
            HerbKind::Glowcap => 2,
        };

        Self {
            kind,
            potency,
            quantity: 1,
            identified: false,
        }
    }

    pub fn random_new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let kinds = [
            HerbKind::Sungrass,
            HerbKind::Moonleaf,
            HerbKind::Nightshade,
            HerbKind::SpiritMoss,
            HerbKind::Dragonthorn,
            HerbKind::Glowcap,
        ];
        let mut herb = Self::new(kinds[rng.random_range(0..kinds.len())]);
        if rng.random_bool(0.35) {
            herb.identified = true;
        }
        if rng.random_bool(0.2) {
            herb.quantity = rng.random_range(2..=4);
        }
        herb
    }

    pub fn name(&self) -> String {
        let base = match self.kind {
            HerbKind::Sungrass => "日光草",
            HerbKind::Moonleaf => "月影叶",
            HerbKind::Nightshade => "夜影花",
            HerbKind::SpiritMoss => "灵魂苔",
            HerbKind::Dragonthorn => "龙棘草",
            HerbKind::Glowcap => "萤帽菌",
        };

        if self.quantity > 1 {
            format!("{} ×{}", base, self.quantity)
        } else {
            base.to_string()
        }
    }

    pub fn rarity_level(&self) -> ItemRarity {
        match self.kind {
            HerbKind::Sungrass | HerbKind::Moonleaf | HerbKind::Glowcap => ItemRarity::Common,
            HerbKind::SpiritMoss => ItemRarity::Rare,
            HerbKind::Nightshade => ItemRarity::Epic,
            HerbKind::Dragonthorn => ItemRarity::Legendary,
        }
    }

    pub fn value(&self) -> u32 {
        let base = match self.kind {
            HerbKind::Sungrass => 35,
            HerbKind::Moonleaf => 45,
            HerbKind::Nightshade => 120,
            HerbKind::SpiritMoss => 80,
            HerbKind::Dragonthorn => 200,
            HerbKind::Glowcap => 60,
        };

        let identification_factor = if self.identified { 1.2 } else { 0.8 };
        (base as f32 * identification_factor * self.quantity as f32) as u32
    }

    pub fn to_potion_effect(&self) -> &'static str {
        match self.kind {
            HerbKind::Sungrass => "治疗药剂基底",
            HerbKind::Moonleaf => "隐匿药剂催化",
            HerbKind::Nightshade => "毒素增强剂",
            HerbKind::SpiritMoss => "精神恢复剂",
            HerbKind::Dragonthorn => "武器涂抹剂",
            HerbKind::Glowcap => "夜视药剂",
        }
    }

    pub fn stackable(&self) -> bool {
        true
    }

    pub fn stack_limit(&self) -> u32 {
        match self.kind {
            HerbKind::Dragonthorn => 6,
            HerbKind::Nightshade => 8,
            HerbKind::SpiritMoss => 10,
            HerbKind::Sungrass | HerbKind::Moonleaf | HerbKind::Glowcap => 16,
        }
    }

    fn stacking_key(&self) -> (HerbKind, u8, bool) {
        (self.kind, self.potency, self.identified)
    }
}

impl Default for Herb {
    fn default() -> Self {
        Self::new(HerbKind::Sungrass)
    }
}

impl fmt::Display for Herb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (效力 {} · {})",
            self.name(),
            self.potency,
            self.to_potion_effect()
        )
    }
}

impl ItemTrait for Herb {
    fn stacking_id(&self) -> u64 {
        let mut hasher = SeaHasher::new();
        let bytes = encode_to_vec(self.stacking_key(), BINCODE_CONFIG).unwrap();
        hasher.write(&bytes);
        hasher.finish()
    }

    fn is_stackable(&self) -> bool {
        self.stackable()
    }

    fn max_stack(&self) -> u32 {
        self.stack_limit()
    }

    fn display_name(&self) -> String {
        self.name()
    }

    fn category(&self) -> ItemCategory {
        ItemCategory::Herb
    }

    fn rarity(&self) -> ItemRarity {
        self.rarity_level()
    }

    fn sort_value(&self) -> u32 {
        match self.kind {
            HerbKind::Dragonthorn => 130,
            HerbKind::Nightshade => 120,
            HerbKind::SpiritMoss => 90,
            HerbKind::Glowcap => 80,
            HerbKind::Moonleaf => 70,
            HerbKind::Sungrass => 60,
        }
    }
}

impl From<Herb> for Item {
    fn from(herb: Herb) -> Self {
        Item {
            name: herb.name(),
            kind: ItemKind::Herb(herb),
            description: "...".to_string(),
            quantity: 1,
            x: -1,
            y: -1,
        }
    }
}
