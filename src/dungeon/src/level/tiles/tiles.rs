//src/dungeon/src/level/tiles/tiles.rs
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub info: TileInfo,
}

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct TileInfo {
    pub passable: bool,
    pub blocks_sight: bool, // 是否阻挡视线
    pub terrain_type: TerrainType,
    pub has_item: bool,
    pub has_enemy: bool,
    pub is_visible: bool,
}

#[derive(Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub enum TerrainType {
    Floor,
    Wall,
    Water,
    Grass,
    Door,
    // 其他地形类型...
}
