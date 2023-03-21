use crate::game::lighting::*;

// Tile:
pub const TILE_SIZE: usize = 16; // Tile size (in pixels).

// Chunk:
pub const CHUNK_SIZE: usize = 8; // Chunk size (in tiles) (must be power of two).
pub const CHUNK_SIZE_LOG2: usize = 3; // Round will probably break this sometimes (?)
pub const CHUNK_LOAD_BUFFER_SIZE: usize = MAX_LIGHT_DISTANCE as usize; // (in tiles).

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[repr(u8)]
pub enum Tile {
    None = 0,
    Dirt = 1,
    Stone = 2,
}
