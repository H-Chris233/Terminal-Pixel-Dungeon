//src/hero/bag/mod.rs
mod bag;
mod equipment;
mod inventory;

pub use bag::Bag;
pub use equipment::{EquipError, Equipment};
pub use super::inventory::{Inventory, ItemSlot};


