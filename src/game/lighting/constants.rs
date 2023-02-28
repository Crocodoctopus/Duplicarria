// Some lighting constants
pub const MAX_BRIGHTNESS: u8 = 16; // Value representing maxmimal brightness.
pub const MIN_BRIGHTNESS: u8 = 0; // Value representing maximal darkness.
pub const MAX_FADE: u8 = MAX_BRIGHTNESS;
pub const MIN_FADE: u8 = 1; // Least amount of fade that can occur.
pub const TRANSPARENT_FADE: u8 = MIN_FADE; // Fade of free space.
pub const OPAQUE_FADE: u8 = TRANSPARENT_FADE * 3; // Fade of solid blocks.
pub const MAX_LIGHT_DISTANCE: usize = (MAX_BRIGHTNESS / MIN_FADE) as usize; // The furthest a light source can reach (in tiles).
