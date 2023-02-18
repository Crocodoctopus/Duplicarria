use crate::array2d::Array2D;
use crate::game::tile::Tile;

pub struct GameFrame {
    // View data.
    pub view_x: usize,
    pub view_y: usize,
    pub view_w: usize,
    pub view_h: usize,

    // Tile layer.
    pub tiles_x: usize,
    pub tiles_y: usize,
    pub foreground_tiles: Array2D<Tile>,
    pub background_tiles: Array2D<Tile>,

    // Lighting layer.
    pub light_x: usize,
    pub light_y: usize,
    pub light_map: Array2D<u8>,
}
