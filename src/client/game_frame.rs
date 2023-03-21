use crate::array2d::*;
use crate::game::item::*;
use crate::game::tile::*;

pub struct GameFrame {
    // Debug printing.
    pub debug_text: String,

    // View data.
    pub view_x: usize,
    pub view_y: usize,
    pub view_w: usize,
    pub view_h: usize,

    // Item layer.
    pub items: Vec<(f32, f32, ItemId)>,

    // Humanoid layer.
    pub humanoid_positions: Vec<(f32, f32)>,

    // Tile layer.
    pub tiles_x: usize,
    pub tiles_y: usize,
    pub foreground_tiles: Array2D<Tile>,
    pub background_tiles: Array2D<Tile>,

    // Lighting layer.
    pub light_x: usize,
    pub light_y: usize,
    pub light_map_r: Array2D<u8>,
    pub light_map_g: Array2D<u8>,
    pub light_map_b: Array2D<u8>,
}
