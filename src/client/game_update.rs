use array2d::{Array2D, FastArray2D};

use super::game_frame::*;
use super::input_event::*;
use crate::game::constants::*;
use crate::game::functions::*;
use crate::game::net_event::*;
use crate::game::tile::*;

pub struct GameUpdate {
    // Misc:
    timer: usize,
    exit: bool,

    // Input:
    cursor_x: f32,
    cursor_y: f32,
    cursor_left_queue: u8,
    cursor_right_queue: u8,
    up_queue: u8,
    down_queue: u8,
    left_queue: u8,
    right_queue: u8,

    // Client view:
    view: (f32, f32, f32, f32),

    // Network:
    outbound: Vec<NetEvent>,
    chunks: FastArray2D<(u16, u16)>,

    // Tiles:
    foreground_tiles: FastArray2D<Tile>,
    background_tiles: FastArray2D<Tile>,

    // Lighting:
    light_map: Array2D<u8>,
    fade_map: Array2D<u8>,
}

impl GameUpdate {
    pub fn new(view_w: f32, view_h: f32) -> Self {
        //
        let chunk_load_buffer_size_px = (CHUNK_LOAD_BUFFER_SIZE * TILE_SIZE) as f32;
        let chunk_size_px = (TILE_SIZE * CHUNK_SIZE) as f32;

        // Get number of chunks that will fit on screen.
        let chunks_v = ((view_w + 2. * chunk_load_buffer_size_px) / chunk_size_px).ceil() as usize;
        let chunks_h = ((view_h + 2. * chunk_load_buffer_size_px) / chunk_size_px).ceil() as usize;

        // Get smallest base2 that can fit chunks_v/chunks_h.
        let max_visible_chunks_v_base2 = (chunks_v as f32).log2().ceil() as usize;
        let max_visible_chunks_h_base2 = (chunks_h as f32).log2().ceil() as usize;

        // Create chunk array.
        let chunks = FastArray2D::from_closure(
            max_visible_chunks_v_base2,
            max_visible_chunks_h_base2,
            |_, _| (u16::max_value(), u16::max_value()),
        );

        // Create tile array (8 x 8) times larger than above array.
        let foreground_tiles = FastArray2D::from_closure(
            max_visible_chunks_v_base2 + CHUNK_SIZE_LOG2,
            max_visible_chunks_h_base2 + CHUNK_SIZE_LOG2,
            |_, _| Tile::None,
        );

        let background_tiles = FastArray2D::from_closure(
            max_visible_chunks_v_base2 + CHUNK_SIZE_LOG2,
            max_visible_chunks_h_base2 + CHUNK_SIZE_LOG2,
            |_, _| Tile::None,
        );

        // Light
        let icdiv = |n, d| (n + d - 1) / d; // ceiling idiv

        // most tiles that can be seen at once
        let max_vis_w = icdiv(view_w as usize - 1, TILE_SIZE) + 1;
        let max_vis_h = icdiv(view_h as usize - 1, TILE_SIZE) + 1;
        let light_map_w = max_vis_w + 2 * MAX_LIGHT_DISTANCE as usize;
        let light_map_h = max_vis_h + 2 * MAX_LIGHT_DISTANCE as usize;

        // Init light map
        let light_map = Array2D::from_closure(light_map_w, light_map_h, |_, _| MAX_BRIGHTNESS);
        let fade_map = Array2D::from_closure(light_map_w, light_map_h, |_, _| MAX_FADE);

        Self {
            timer: 0,
            exit: false,

            cursor_x: 0.,
            cursor_y: 0.,
            cursor_left_queue: 0,
            cursor_right_queue: 0,
            up_queue: 0,
            down_queue: 0,
            left_queue: 0,
            right_queue: 0,

            view: (32., 32., view_w, view_h),

            outbound: Vec::new(),
            chunks,

            foreground_tiles,
            background_tiles,

            light_map,
            fade_map,
        }
    }

    pub fn preframe(
        &mut self,
        _timestamp: u64,
        input_events: impl Iterator<Item = InputEvent>,
        net_events: impl Iterator<Item = NetEvent>,
    ) {
        // Net loop.
        for net in net_events {
            match net {
                NetEvent::Accept => {}
                NetEvent::UpdateForegroundChunk(x, y, tiles) => {
                    // Verify the incoming chunk exists in the world still, update tiles.
                    if &(x, y) == self.chunks.get_wrapping(x as usize, y as usize) {
                        self.foreground_tiles.splice_wrapping(
                            CHUNK_SIZE * x as usize..CHUNK_SIZE * (x as usize + 1),
                            CHUNK_SIZE * y as usize..CHUNK_SIZE * (y as usize + 1),
                            tiles.clone(),
                        );
                    }
                }
                NetEvent::UpdateBackgroundChunk(x, y, tiles) => {
                    // Verify the incoming chunk exists in the world still, update tiles.
                    if &(x, y) == self.chunks.get_wrapping(x as usize, y as usize) {
                        self.background_tiles.splice_wrapping(
                            CHUNK_SIZE * x as usize..CHUNK_SIZE * (x as usize + 1),
                            CHUNK_SIZE * y as usize..CHUNK_SIZE * (y as usize + 1),
                            tiles.clone(),
                        );
                    }
                }
                _ => {}
            }
        }

        // Shift left, cloning right most bit.
        self.cursor_left_queue = self.cursor_left_queue & 1 | self.cursor_left_queue << 1;
        self.cursor_right_queue = self.cursor_right_queue & 1 | self.cursor_right_queue << 1;
        self.up_queue = self.up_queue & 1 | self.up_queue << 1;
        self.down_queue = self.down_queue & 1 | self.down_queue << 1;
        self.left_queue = self.left_queue & 1 | self.left_queue << 1;
        self.right_queue = self.right_queue & 1 | self.right_queue << 1;

        // Input loop.
        for input in input_events {
            match input {
                InputEvent::KeyEvent(KeyState::Down, InputKey::W) => self.up_queue |= 1,
                InputEvent::KeyEvent(KeyState::Down, InputKey::A) => self.left_queue |= 1,
                InputEvent::KeyEvent(KeyState::Down, InputKey::S) => self.down_queue |= 1,
                InputEvent::KeyEvent(KeyState::Down, InputKey::D) => self.right_queue |= 1,
                InputEvent::KeyEvent(KeyState::Up, InputKey::W) => self.up_queue &= !1,
                InputEvent::KeyEvent(KeyState::Up, InputKey::A) => self.left_queue &= !1,
                InputEvent::KeyEvent(KeyState::Up, InputKey::S) => self.down_queue &= !1,
                InputEvent::KeyEvent(KeyState::Up, InputKey::D) => self.right_queue &= !1,

                InputEvent::CursorMove(x, y) => (self.cursor_x, self.cursor_y) = (x, y),
                InputEvent::KeyEvent(KeyState::Down, InputKey::LeftClick) => {
                    self.cursor_left_queue |= 1
                }
                InputEvent::KeyEvent(KeyState::Down, InputKey::RightClick) => {
                    self.cursor_right_queue |= 1
                }
                InputEvent::KeyEvent(KeyState::Up, InputKey::LeftClick) => {
                    self.cursor_left_queue &= !1
                }
                InputEvent::KeyEvent(KeyState::Up, InputKey::RightClick) => {
                    self.cursor_right_queue &= !1
                }
                _ => continue,
            };
        }

        // On left click
        if self.cursor_left_queue & 0b1 == 1 && self.cursor_left_queue & 0b10 == 0 {
            let x = ((self.view.0 + self.cursor_x) / 16.) as usize;
            let y = ((self.view.1 + self.cursor_y) / 16.) as usize;
            *self.foreground_tiles.get_wrapping_mut(x, y) = Tile::None;
            self.outbound
                .push(NetEvent::BreakForeground(x as _, y as _));
        }

        // On right click
        if self.cursor_right_queue & 0b1 == 1 && self.cursor_right_queue & 0b10 == 0 {
            let x = ((self.view.0 + self.cursor_x) / 16.) as usize;
            let y = ((self.view.1 + self.cursor_y) / 16.) as usize;
            *self.background_tiles.get_wrapping_mut(x, y) = Tile::None;
            self.outbound
                .push(NetEvent::BreakBackground(x as _, y as _));
        }
    }

    pub fn step(&mut self, _timestamp: u64, frametime: u64) {
        let dt = frametime as f32 / 1_000_000.;

        if self.up_queue & 1 > 0 {
            self.view.1 -= 160. * dt;
        }
        if self.down_queue & 1 > 0 {
            self.view.1 += 160. * dt;
        }
        if self.left_queue & 1 > 0 {
            self.view.0 -= 160. * dt;
        }
        if self.right_queue & 1 > 0 {
            self.view.0 += 160. * dt;
        }

        // Ensure the view is always inbounds.
        if self.view.0 < 16. {
            self.view.0 = 16.;
        }
        if self.view.1 < 16. {
            self.view.1 = 16.;
        }

        // Record some view stuff
        let ifdiv = |n, d| n / d; // floor idiv
        let icdiv = |n, d| ifdiv(n + d - 1, d); // ceiling idiv

        let camx1 =
            ifdiv(self.view.0 as usize, TILE_SIZE).saturating_sub(MAX_LIGHT_DISTANCE as usize);
        let camx2 =
            icdiv((self.view.0 + self.view.2) as usize, TILE_SIZE) + MAX_LIGHT_DISTANCE as usize;
        let camy1 =
            ifdiv(self.view.1 as usize, TILE_SIZE).saturating_sub(MAX_LIGHT_DISTANCE as usize);
        let camy2 =
            icdiv((self.view.1 + self.view.3) as usize, TILE_SIZE) + MAX_LIGHT_DISTANCE as usize;

        let lmx =
            ifdiv(self.view.0 as usize, TILE_SIZE).saturating_sub(MAX_LIGHT_DISTANCE as usize);
        let lmy =
            ifdiv(self.view.1 as usize, TILE_SIZE).saturating_sub(MAX_LIGHT_DISTANCE as usize);

        // Wipe light_map and fade_map
        let (w, h) = self.light_map.size();
        self.light_map
            .for_each_sub_wrapping_mut(1..w - 1, 1..h - 1, |_, _, t| *t = MIN_BRIGHTNESS);
        self.fade_map
            .for_each_sub_wrapping_mut(0..w, 0..h, |_, _, t| *t = MAX_FADE);

        let xr = camx1 - lmx + 1..camx2 - lmx;
        let yr = camy1 - lmy + 1..camy2 - lmy;

        // Set up light_map and fade_map from tile data
        let mut light_queue = vec![];
        let (tw, th) = self.foreground_tiles.size();
        let m = lmy * w - lmx;
        crate::array2d::for_each_sub_wrapping(
            tw,
            th,
            camx1 + 1..camx2 - 1,
            camy1 + 1..camy2 - 1,
            |x, y, index| {
                let tile_index = index;
                let light_index = x + y * w - m;

                // get tile at this (x, y)
                let fg_tile = self.foreground_tiles[tile_index];
                let bg_tile = self.background_tiles[tile_index];

                match (fg_tile, bg_tile) {
                    // For (air, air), update the light map and push a light probe
                    (Tile::None, Tile::None) => {
                        self.light_map[light_index] = MAX_BRIGHTNESS;
                        self.fade_map[light_index] = MIN_FADE;
                        light_queue.push(light_index);
                    }
                    // For (air, anything), make transparent fade
                    (Tile::None, _) => self.fade_map[light_index] = TRANSPARENT_FADE,
                    // Anything else, make solid fade
                    (_, _) => self.fade_map[light_index] = OPAQUE_FADE,
                }
            },
        );

        // Add misc light sources

        // propogate light map
        propogate_light_map_unbounded(&mut self.light_map, &self.fade_map, light_queue);
    }

    pub fn postframe(
        &mut self,
        _timestamp: u64,
    ) -> (Option<GameFrame>, impl IntoIterator<Item = NetEvent>) {
        // Request from the server any chunks that may now be onscreen (Should client be the one to ask this?).
        super::functions::request_chunks_from_server(
            self.view,
            &mut self.chunks,
            &mut self.outbound,
        );

        // Clone the visible tiles.
        let (tiles_x, tiles_y, foreground_tiles) =
            super::functions::clone_onscreen_tiles(self.view, &self.foreground_tiles);
        let (_, _, background_tiles) =
            super::functions::clone_onscreen_tiles(self.view, &self.background_tiles);

        // Clone the innermost square of the light map
        // Record some view stuff
        let ifdiv = |n, d| n / d; // floor idiv
        let icdiv = |n, d| ifdiv(n + d - 1, d); // ceiling idiv
        let camx1 = ifdiv(self.view.0 as usize, TILE_SIZE);
        let camx2 = icdiv((self.view.0 + self.view.2) as usize, TILE_SIZE);
        let camy1 = ifdiv(self.view.1 as usize, TILE_SIZE);
        let camy2 = icdiv((self.view.1 + self.view.3) as usize, TILE_SIZE);
        let lmx =
            ifdiv(self.view.0 as usize, TILE_SIZE).saturating_sub(MAX_LIGHT_DISTANCE as usize);
        let lmy =
            ifdiv(self.view.1 as usize, TILE_SIZE).saturating_sub(MAX_LIGHT_DISTANCE as usize);
        let light_map = self
            .light_map
            .clone_sub(camx1 - lmx..camx2 - lmx, camy1 - lmy..camy2 - lmy)
            .unwrap();

        // Construct frame.
        let frame = (!self.exit).then(|| GameFrame {
            view_x: self.view.0 as usize,
            view_y: self.view.1 as usize,
            view_w: self.view.2 as usize,
            view_h: self.view.3 as usize,

            tiles_x,
            tiles_y,
            foreground_tiles,
            background_tiles,

            light_x: camx1, // TEMP
            light_y: camy1, // TEMP
            light_map,
        });

        // Return.
        (frame, std::mem::take(&mut self.outbound))
    }
}
