//src/dungeon/level/rooms/rooms.rs
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::level::tiles::tiles::Tile;

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Tile>,
}
