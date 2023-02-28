use crate::array2d::*;
use crate::common::*;
use crate::game::lighting::*;
use crate::game::tile::*;

#[inline(always)]
pub fn gen_fade_map(
    (view_x, view_y): (usize, usize),
    foreground_tiles: &FastArray2D<Tile>,
    background_tiles: &FastArray2D<Tile>,
    mut fade_map: &mut Array2D<u8>,
    mut light_map: &mut Array2D<u8>,
) -> impl Iterator<Item = usize> {
    let (w, h) = light_map.size();

    // Record some view stuff
    let x = ifdiv(view_x, TILE_SIZE).saturating_sub(MAX_LIGHT_DISTANCE);
    let y = ifdiv(view_y, TILE_SIZE).saturating_sub(MAX_LIGHT_DISTANCE);

    // Set up the fade_map from tile data
    let mut light_queue = Vec::with_capacity(1024);
    let (tw, th) = foreground_tiles.size();
    let m = x + y * w;
    crate::array2d::for_each_sub_wrapping(
        tw,
        th,
        x + 1..x + w - 1,
        y + 1..y + h - 1,
        |x, y, index| {
            let tile_index = index;
            let light_index = x + y * w - m;

            // get tile at this (x, y)
            let fg_tile = foreground_tiles[tile_index];
            let bg_tile = background_tiles[tile_index];

            match (fg_tile, bg_tile) {
                // For (air, air), update the light map and push a light probe
                (Tile::None, Tile::None) => {
                    light_map[light_index] = MAX_BRIGHTNESS;
                    fade_map[light_index] = MIN_FADE;
                    light_queue.push(light_index);
                }
                // For (air, anything), make transparent fade
                (Tile::None, _) => fade_map[light_index] = TRANSPARENT_FADE,
                // Anything else, make solid fade
                (_, _) => fade_map[light_index] = OPAQUE_FADE,
            }
        },
    );

    light_queue.into_iter()
}
