pub mod net_event;
pub mod tile;

// Tile:
pub const TILE_SIZE: usize = 16; // Tile size (in pixels).

// Lighting:
pub const MAX_BRIGHTNESS: u8 = 255; // Value representing maxmimal brightness.
pub const MIN_BRIGHTNESS: u8 = 0; // Value representing maximal darkness.
pub const MIN_FADE: u8 = 8; // Least amount of fade that can occur.
pub const TRANSPARENT_FADE: u8 = MIN_FADE; // Fade of free space.
pub const OPAQUE_FADE: u8 = TRANSPARENT_FADE * 3; // Fade of solid blocks.
pub const MAX_LIGHT_DISTANCE: u8 = MAX_BRIGHTNESS / MIN_FADE; // The furthest a light source can reach (in tiles).

// Chunk:
pub const CHUNK_SIZE: usize = 8; // Chunk size (in tiles).
pub const CHUNK_LOAD_BUFFER_SIZE: usize = MAX_LIGHT_DISTANCE as _; // (in tiles).
