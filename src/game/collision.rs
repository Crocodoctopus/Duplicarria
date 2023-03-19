pub use crate::array2d::*;
pub use crate::common::*;
pub use crate::game::humanoid::*;
pub use crate::game::tile::*;

/// Collects tiles colliding tiles that were not colliding previously, after an x movement occured.
/// Returns x positions of all tiles collected. Note: movements larger than TILE_SIZE will result
/// in invalid outputs.
pub fn collect_newly_colliding_tiles_x(
    old_x: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    tiles: &impl Index2d<usize, Output = Tile>,
    vec: &mut Vec<Tile>,
) -> usize {
    // Calculate y1 and y2.
    let x1_old = (old_x / TILE_SIZE as f32).floor() as usize;
    let x2_old = ((old_x + w) / TILE_SIZE as f32).ceil() as usize;
    let x1_new = (x / TILE_SIZE as f32).floor() as usize;
    let x2_new = ((x + w) / TILE_SIZE as f32).ceil() as usize;
    let (x1, x2) = if x > old_x {
        (x2_old, x2_new)
    } else {
        (x1_new, x1_old)
    };

    // Calculate x1 and x2.
    let y1 = (y / TILE_SIZE as f32).floor() as usize;
    let y2 = ((y + h) / TILE_SIZE as f32).ceil() as usize;

    // Make sure their values are reasonable.
    assert!(x2 - x1 < 6);
    assert!(y2 - y1 < 6);

    // Get the tiles.
    let (w, h) = tiles.size();
    for_each_sub_wrapping(w, h, x1..x2, y1..y2, |x, y, index| {
        let tile = tiles[index];
        if !matches!(tile, Tile::None) {
            vec.push(tile);
        }
    });

    x1
}

/// Collects tiles colliding tiles that were not colliding previously, after a y movement occured.
/// Returns y positions of all tiles collected. Note: movements larger than TILE_SIZE will result
/// in invalid outputs.
pub fn collect_newly_colliding_tiles_y(
    old_y: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    tiles: &impl Index2d<usize, Output = Tile>,
    vec: &mut Vec<Tile>,
) -> usize {
    // Calculate y1 and y2.
    let y1_old = (old_y / TILE_SIZE as f32).floor() as usize;
    let y2_old = ((old_y + h) / TILE_SIZE as f32).ceil() as usize;
    let y1_new = (y / TILE_SIZE as f32).floor() as usize;
    let y2_new = ((y + h) / TILE_SIZE as f32).ceil() as usize;
    let (y1, y2) = if y > old_y {
        (y2_old, y2_new)
    } else {
        (y1_new, y1_old)
    };

    // Calculate x1 and x2.
    let x1 = (x / TILE_SIZE as f32).floor() as usize;
    let x2 = ((x + w) / TILE_SIZE as f32).ceil() as usize;

    // Make sure their values are reasonable.
    assert!(x2 - x1 < 6);
    assert!(y2 - y1 < 6);

    // Get the tiles.
    let (w, h) = tiles.size();
    for_each_sub_wrapping(w, h, x1..x2, y1..y2, |x, y, index| {
        let tile = tiles[index];
        if !matches!(tile, Tile::None) {
            vec.push(tile);
        }
    });

    y1
}
