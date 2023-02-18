pub use crate::game::tile::Tile;
pub use array2d::{Array2D, FastArray2D};

pub fn clone_onscreen_tiles(
    (x, y, w, h): (f32, f32, f32, f32),
    tiles: &FastArray2D<Tile>,
) -> (usize, usize, Array2D<Tile>) {
    // Map view to tile space.
    let x1 = ((x - 32.) / 16.).floor() as usize;
    let y1 = ((y - 32.) / 16.).floor() as usize;
    let x2 = ((x + w + 32.) / 16.).ceil() as usize;
    let y2 = ((y + h + 32.) / 16.).ceil() as usize;

    (x1, y1, tiles.clone_sub_wrapping(x1..x2, y1..y2))
}
