use crate::array2d::*;
use crate::common::*;
use crate::game::tile::*;

// Some lighting constants
pub const MAX_BRIGHTNESS: u8 = 16; // Value representing maxmimal brightness.
pub const MIN_BRIGHTNESS: u8 = 0; // Value representing maximal darkness.
pub const MAX_FADE: u8 = MAX_BRIGHTNESS;
pub const MIN_FADE: u8 = 1; // Least amount of fade that can occur.
pub const TRANSPARENT_FADE: u8 = MIN_FADE; // Fade of free space.
pub const OPAQUE_FADE: u8 = TRANSPARENT_FADE * 3; // Fade of solid blocks.
pub const MAX_LIGHT_DISTANCE: usize = (MAX_BRIGHTNESS / MIN_FADE) as usize; // The furthest a light source can reach (in tiles).

#[inline(always)]
pub fn gen_fade_map(
    (view_x, view_y): (usize, usize),
    foreground_tiles: &FastArray2D<Tile>,
    background_tiles: &FastArray2D<Tile>,
    mut fade_map: &mut Array2D<u8>,
    mut light_map_r: &mut Array2D<u8>,
    mut light_map_g: &mut Array2D<u8>,
    mut light_map_b: &mut Array2D<u8>,
) -> impl Iterator<Item = usize> + Clone {
    let (w, h) = light_map_r.size();

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
                    light_map_r[light_index] = MAX_BRIGHTNESS;
                    light_map_g[light_index] = MAX_BRIGHTNESS;
                    light_map_b[light_index] = MAX_BRIGHTNESS;
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

/// Generates a lightmap from points
#[inline(always)]
pub fn propogate_light_map_unbounded(
    mut light_map: impl Index2dMut<usize, Output = u8>,
    fade_map: impl Index2d<usize, Output = u8>,
    lights: impl IntoIterator<Item = usize>,
) {
    let stride = light_map.stride();

    let mut queue = std::collections::VecDeque::from_iter(lights);

    while let Some(index) = queue.pop_front() {
        let brightness = light_map[index];
        let fade = fade_map[index];
        let new_brightness = brightness.saturating_sub(fade);

        // Left
        let next_index = index - 1;
        if light_map[next_index] < new_brightness {
            light_map[next_index] = new_brightness;
            queue.push_back(next_index);
        }

        // Right
        let next_index = index + 1;
        if light_map[next_index] < new_brightness {
            light_map[next_index] = new_brightness;
            queue.push_back(next_index);
        }

        // Top
        let next_index = index - stride;
        if light_map[next_index] < new_brightness {
            light_map[next_index] = new_brightness;
            queue.push_back(next_index);
        }

        // Bottom
        let next_index = index + stride;
        if light_map[next_index] < new_brightness {
            light_map[next_index] = new_brightness;
            queue.push_back(next_index);
        }
    }
}
