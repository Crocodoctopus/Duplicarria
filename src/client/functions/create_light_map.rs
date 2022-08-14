use crate::shared::tile::*;
use crate::shared::*;
use array2d::*;
use std::collections::VecDeque;

pub fn create_light_map(
    (x, y, w, h): (f32, f32, f32, f32),
    fg_tiles: &FastArray2D<Tile>,
    bg_tiles: &FastArray2D<Tile>,
    _timer: &mut usize,
) -> (usize, usize, Array2D<u8>) {
    // Generate a rectangle (in tiles) representing the light map.
    let x1 = (x / TILE_SIZE as f32 - MAX_LIGHT_DISTANCE as f32).floor() as usize;
    let y1 = (y / TILE_SIZE as f32 - MAX_LIGHT_DISTANCE as f32).floor() as usize;
    let x2 =
        (x / TILE_SIZE as f32 + w / TILE_SIZE as f32 + MAX_LIGHT_DISTANCE as f32).ceil() as usize;
    let y2 =
        (y / TILE_SIZE as f32 + h / TILE_SIZE as f32 + MAX_LIGHT_DISTANCE as f32).ceil() as usize;

    // Light map dimensions.
    let light_map_w: usize = x2 - x1;
    let light_map_h: usize = y2 - y1;

    // Generate light and fade map.
    let mut fade_map: Box<[u8]> = vec![OPAQUE_FADE; light_map_w * light_map_h].into_boxed_slice();
    let mut light_map: Box<[u8]> =
        vec![MIN_BRIGHTNESS; light_map_w * light_map_h].into_boxed_slice();

    // Make edges of light map fully lit, to prevent OOB during light calculation.
    let to_light_map_index = |x: usize, y: usize| x + y * light_map_w;
    for x in 0..light_map_w {
        light_map[to_light_map_index(x, 0)] = MAX_BRIGHTNESS;
        light_map[to_light_map_index(x, light_map_h - 1)] = MAX_BRIGHTNESS;
    }
    for y in 0..light_map_h {
        light_map[to_light_map_index(0, y)] = MAX_BRIGHTNESS;
        light_map[to_light_map_index(light_map_w - 1, y)] = MAX_BRIGHTNESS;
    }

    // Generate light source queue.
    let mut queue = VecDeque::with_capacity(1024);

    //
    let xr = x1 + 1..x2 - 1;
    let yr = y1 + 1..y2 - 1;
    let (w, h) = fg_tiles.size();
    assert_eq!((w, h), bg_tiles.size());
    for_each_sub_wrapping(w, h, xr, yr, |x, y, index| {
        let fg_tile = fg_tiles[index];
        let bg_tile = bg_tiles[index];

        // New lightmap index
        let index = x - x1 + (y - y1) * light_map_w;
        match (fg_tile, bg_tile) {
            // Sky tile
            (Tile::None, Tile::None) => {
                light_map[index] = MAX_BRIGHTNESS;
                queue.push_back(index);
            }
            (Tile::None, _) => fade_map[index] = TRANSPARENT_FADE,
            _ => {}
        }
    });

    // Push misc light sources into queue.
    /*for &light_source in &light_sources {
        queue.push_back(light_source);
        light_map[light_source as usize] = 0;
    }*/

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
        let next_index = index - light_map_w;
        if light_map[next_index] < new_brightness {
            light_map[next_index] = new_brightness;
            queue.push_back(next_index);
        }

        // Bottom
        let next_index = index + light_map_w;
        if light_map[next_index] < new_brightness {
            light_map[next_index] = new_brightness;
            queue.push_back(next_index);
        }
    }

    (
        x1,
        y1,
        Array2D::from_box(light_map_w, light_map_h, light_map),
    )
}
