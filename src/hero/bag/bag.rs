use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::hero::hero::*;
use crate::items::armor::Armor;
use crate::items::weapon::Weapon;

pub struct Bag {
    weapon: Weapon,
    armor: Armor,
}
