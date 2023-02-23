use array2d::*;

/// Generates a lightmap from points
#[inline(always)]
pub fn propogate_light_map_unbounded(
    mut light_map: impl Index2dMut<usize, Output = u8>,
    fade_map: impl Index2d<usize, Output = u8>,
    lights: impl IntoIterator<Item = usize>,
) {
    let stride = light_map.stride();

    use std::iter::FromIterator;
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
