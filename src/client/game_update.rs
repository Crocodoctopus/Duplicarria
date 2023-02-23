use array2d::{Array2D, FastArray2D};

use super::functions::*;
use super::game_frame::*;
use super::input_event::*;
use crate::common::*;

use crate::game::lighting::*;
use crate::game::net::*;
use crate::game::tile::*;

pub struct GameUpdate {
    // Misc:
    timer: usize,
    exit: bool,

    // Input:
    cursor_x: usize,
    cursor_y: usize,
    cursor_left_queue: u8,
    cursor_right_queue: u8,
    up_queue: u8,
    down_queue: u8,
    left_queue: u8,
    right_queue: u8,

    // Client view:
    view_pos: (usize, usize),
    view_size: (usize, usize),

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
        let view_w = view_w as usize;
        let view_h = view_h as usize;

        //
        let chunk_load_buffer_size_px = CHUNK_LOAD_BUFFER_SIZE * TILE_SIZE;
        let chunk_size_px = TILE_SIZE * CHUNK_SIZE;

        // Get number of chunks that will fit on screen.
        let chunks_v = icdiv(view_w + 2 * chunk_load_buffer_size_px, chunk_size_px);
        let chunks_h = icdiv(view_h + 2 * chunk_load_buffer_size_px, chunk_size_px);

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
        // most tiles that can be seen at once
        let max_vis_w = icdiv(view_w - 1, TILE_SIZE) + 1;
        let max_vis_h = icdiv(view_h - 1, TILE_SIZE) + 1;
        let light_map_w = max_vis_w + 2 * MAX_LIGHT_DISTANCE;
        let light_map_h = max_vis_h + 2 * MAX_LIGHT_DISTANCE;

        // Init light map
        let light_map = Array2D::from_closure(light_map_w, light_map_h, |_, _| MAX_BRIGHTNESS);
        let fade_map = Array2D::from_closure(light_map_w, light_map_h, |_, _| MAX_FADE);

        Self {
            timer: 0,
            exit: false,

            cursor_x: 0,
            cursor_y: 0,
            cursor_left_queue: 0,
            cursor_right_queue: 0,
            up_queue: 0,
            down_queue: 0,
            left_queue: 0,
            right_queue: 0,

            view_pos: (32, 32),
            view_size: (view_w, view_h),

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

                InputEvent::CursorMove(x, y) => (self.cursor_x, self.cursor_y) = (x as _, y as _),
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
            let x = (self.view_pos.0 + self.cursor_x) / 16;
            let y = (self.view_pos.1 + self.cursor_y) / 16;
            *self.foreground_tiles.get_wrapping_mut(x, y) = Tile::None;
            self.outbound
                .push(NetEvent::BreakForeground(x as _, y as _));
        }

        // On right click
        if self.cursor_right_queue & 0b1 == 1 && self.cursor_right_queue & 0b10 == 0 {
            let x = (self.view_pos.0 + self.cursor_x) / 16;
            let y = (self.view_pos.1 + self.cursor_y) / 16;
            *self.background_tiles.get_wrapping_mut(x, y) = Tile::None;
            self.outbound
                .push(NetEvent::BreakBackground(x as _, y as _));
        }
    }

    pub fn step(&mut self, _timestamp: u64, frametime: u64) {
        let _dt = frametime as f32 / 1_000_000.;

        // Temporary camera movement
        if self.up_queue & 1 > 0 {
            self.view_pos.1 -= 3;
        }
        if self.down_queue & 1 > 0 {
            self.view_pos.1 += 3;
        }
        if self.left_queue & 1 > 0 {
            self.view_pos.0 -= 3;
        }
        if self.right_queue & 1 > 0 {
            self.view_pos.0 += 3;
        }

        // Ensure the view is always inbounds.
        self.view_pos.0 = self.view_pos.0.max(16);
        self.view_pos.1 = self.view_pos.1.max(16);

        // Request from the server any chunks that may now be onscreen (Should client be the one to ask this?).
        request_chunks_from_server(
            self.view_pos,
            self.view_size,
            &mut self.chunks,
            &mut self.outbound,
        );

        update_lighting(
            self.view_pos,
            self.view_size,
            &self.foreground_tiles,
            &self.background_tiles,
            &mut self.light_map,
            &mut self.fade_map,
        );
    }

    pub fn postframe(
        &mut self,
        _timestamp: u64,
    ) -> (Option<GameFrame>, impl IntoIterator<Item = NetEvent>) {
        // Clone the visible tiles.
        const VISIBLE_TILE_BUFFER: usize = 4;
        let x1 = ifdiv(self.view_pos.0 - VISIBLE_TILE_BUFFER, TILE_SIZE).saturating_sub(1);
        let x2 = icdiv(
            self.view_pos.0 + self.view_size.0 + VISIBLE_TILE_BUFFER,
            TILE_SIZE,
        ) + 1;
        let y1 = ifdiv(self.view_pos.1 - VISIBLE_TILE_BUFFER, TILE_SIZE).saturating_sub(1);
        let y2 = icdiv(
            self.view_pos.1 + self.view_size.1 + VISIBLE_TILE_BUFFER,
            TILE_SIZE,
        ) + 1;
        let foreground_tiles = self.foreground_tiles.clone_sub_wrapping(x1..x2, y1..y2);
        let background_tiles = self.background_tiles.clone_sub_wrapping(x1..x2, y1..y2);
        let (tiles_x, tiles_y) = (x1, y1);

        // Clone the innermost square of the light map
        let camx1 = ifdiv(self.view_pos.0, TILE_SIZE);
        let camx2 = icdiv(self.view_pos.0 + self.view_size.0, TILE_SIZE);
        let camy1 = ifdiv(self.view_pos.1, TILE_SIZE);
        let camy2 = icdiv(self.view_pos.1 + self.view_size.1, TILE_SIZE);
        let lmx = ifdiv(self.view_pos.0, TILE_SIZE).saturating_sub(MAX_LIGHT_DISTANCE);
        let lmy = ifdiv(self.view_pos.1, TILE_SIZE).saturating_sub(MAX_LIGHT_DISTANCE);
        let light_map = self
            .light_map
            .clone_sub(camx1 - lmx..camx2 - lmx, camy1 - lmy..camy2 - lmy)
            .unwrap();

        // Construct frame.
        let frame = (!self.exit).then(|| GameFrame {
            view_x: self.view_pos.0,
            view_y: self.view_pos.1,
            view_w: self.view_size.0,
            view_h: self.view_size.1,

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
