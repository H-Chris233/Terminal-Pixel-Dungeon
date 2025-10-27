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

/// 投掷武器系统
#[derive(Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Throwable {
    pub kind: ThrowableKind,
    pub damage: (u32, u32),
    pub range: u8,
    pub quantity: u32,
    pub identified: bool,
    pub base_value: u32,
}

#[derive(Copy, Eq, Hash, PartialEq, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub enum ThrowableKind {
    Dart,
    Shuriken,
    Javelin,
    Chakram,
    Bomb,
    Boomerang,
}

impl Throwable {
    pub fn new(kind: ThrowableKind) -> Self {
        let (damage, range, quantity, base_value) = match kind {
            ThrowableKind::Dart => ((2, 4), 4, 6, 20),
            ThrowableKind::Shuriken => ((3, 6), 4, 5, 35),
            ThrowableKind::Javelin => ((5, 10), 5, 3, 75),
            ThrowableKind::Chakram => ((4, 9), 5, 1, 120),
            ThrowableKind::Bomb => ((8, 14), 3, 2, 160),
            ThrowableKind::Boomerang => ((3, 8), 6, 1, 180),
        };

        Self {
            kind,
            damage,
            range,
            quantity,
            identified: true,
            base_value,
        }
    }

    pub fn random_new() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let kinds = [
            ThrowableKind::Dart,
            ThrowableKind::Shuriken,
            ThrowableKind::Javelin,
            ThrowableKind::Chakram,
            ThrowableKind::Bomb,
            ThrowableKind::Boomerang,
        ];
        let kind = kinds[rng.random_range(0..kinds.len())];
        let mut item = Self::new(kind);

        if item.stackable() {
            let max = item.stack_limit();
            if max > 1 {
                item.quantity = rng.random_range(1..=max.min(6));
            }
        }

        item
    }

    pub fn name(&self) -> String {
        let base = match self.kind {
            ThrowableKind::Dart => "飞镖",
            ThrowableKind::Shuriken => "手里剑",
            ThrowableKind::Javelin => "标枪",
            ThrowableKind::Chakram => "回旋刃",
            ThrowableKind::Bomb => "震撼炸弹",
            ThrowableKind::Boomerang => "回力镖",
        };

        if self.is_stackable() && self.quantity > 1 {
            format!("{} ×{}", base, self.quantity)
        } else {
            base.to_string()
        }
    }

    pub fn rarity_level(&self) -> ItemRarity {
        match self.kind {
            ThrowableKind::Dart | ThrowableKind::Shuriken => ItemRarity::Common,
            ThrowableKind::Javelin | ThrowableKind::Boomerang => ItemRarity::Rare,
            ThrowableKind::Chakram => ItemRarity::Epic,
            ThrowableKind::Bomb => ItemRarity::Legendary,
        }
    }

    pub fn value(&self) -> u32 {
        if self.stackable() {
            self.base_value * self.quantity
        } else {
            self.base_value
        }
    }

    pub fn stackable(&self) -> bool {
        matches!(
            self.kind,
            ThrowableKind::Dart
                | ThrowableKind::Shuriken
                | ThrowableKind::Javelin
                | ThrowableKind::Bomb
        )
    }

    pub fn stack_limit(&self) -> u32 {
        match self.kind {
            ThrowableKind::Dart => 24,
            ThrowableKind::Shuriken => 18,
            ThrowableKind::Javelin => 12,
            ThrowableKind::Bomb => 6,
            ThrowableKind::Chakram | ThrowableKind::Boomerang => 1,
        }
    }

    fn stacking_key(&self) -> (ThrowableKind, u32, u8) {
        (self.kind, 0, self.range)
    }
}

impl Default for Throwable {
    fn default() -> Self {
        Self::new(ThrowableKind::Dart)
    }
}

impl fmt::Display for Throwable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (伤害 {}-{}, 射程 {})",
            self.name(),
            self.damage.0,
            self.damage.1,
            self.range
        )
    }
}

impl ItemTrait for Throwable {
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
        ItemCategory::Throwable
    }

    fn rarity(&self) -> ItemRarity {
        self.rarity_level()
    }

    fn sort_value(&self) -> u32 {
        match self.kind {
            ThrowableKind::Bomb => 140,
            ThrowableKind::Chakram => 120,
            ThrowableKind::Boomerang => 110,
            ThrowableKind::Javelin => 100,
            ThrowableKind::Shuriken => 80,
            ThrowableKind::Dart => 60,
        }
    }
}

impl From<Throwable> for Item {
    fn from(throwable: Throwable) -> Self {
        let qty = throwable.quantity;
        Item {
            name: throwable.name(),
            kind: ItemKind::Throwable(throwable),
            description: "...".to_string(),
            quantity: qty.max(1),
            x: -1,
            y: -1,
        }
    }
}
