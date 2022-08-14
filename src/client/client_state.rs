use array2d::FastArray2D;

use super::input_event::*;
use super::render_frame::RenderFrame;
use crate::shared::net_event::*;
use crate::shared::tile::*;
use crate::shared::*;

pub struct ClientState {
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
    view: (f32, f32, f32, f32), // x, y, w, h

    // Network:
    chunks: FastArray2D<(u16, u16)>,

    // Tiles:
    foreground_tiles: FastArray2D<Tile>,
    background_tiles: FastArray2D<Tile>,
}

impl ClientState {
    pub fn new(view_w: f32, view_h: f32) -> Self {
        //
        let chunk_load_buffer_size_px = (CHUNK_LOAD_BUFFER_SIZE * TILE_SIZE) as f32;
        let chunk_size_px = (TILE_SIZE * CHUNK_SIZE) as f32;

        // Get number of chunks that will fit on screen (plus 2 chunk border on all sides);
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
            max_visible_chunks_v_base2 + 3,
            max_visible_chunks_h_base2 + 3,
            |_, _| Tile::None,
        );

        let background_tiles = FastArray2D::from_closure(
            max_visible_chunks_v_base2 + 3,
            max_visible_chunks_h_base2 + 3,
            |_, _| Tile::None,
        );

        println!(
            "Chunks: {:?}, Tiles: {:?}",
            chunks.size(),
            foreground_tiles.size()
        );

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

            chunks,

            foreground_tiles,
            background_tiles,
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
                NetEvent::UpdateChunk(x, y, tiles) => {
                    // Verify the incoming chunk exists in the world still, update tiles.
                    if &(x, y) == self.chunks.get_wrapping(x as usize, y as usize) {
                        self.foreground_tiles.splice_wrapping(
                            8 * x as usize..8 * x as usize + 8,
                            8 * y as usize..8 * y as usize + 8,
                            tiles.clone(),
                        );
                        self.background_tiles.splice_wrapping(
                            8 * x as usize..8 * x as usize + 8,
                            8 * y as usize..8 * y as usize + 8,
                            tiles,
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
        }

        // On right click
        if self.cursor_right_queue & 0b1 == 1 && self.cursor_right_queue & 0b10 == 0 {
            let x = ((self.view.0 + self.cursor_x) / 16.) as usize;
            let y = ((self.view.1 + self.cursor_y) / 16.) as usize;
            *self.background_tiles.get_wrapping_mut(x, y) = Tile::None;
        }
    }

    pub fn step(&mut self, _timestamp: u64, frametime: u64) {
        let dt = frametime as f32 / 1_000_000.;

        if self.up_queue & 1 > 0 {
            self.view.1 -= 60. * dt;
        }
        if self.down_queue & 1 > 0 {
            self.view.1 += 60. * dt;
        }
        if self.left_queue & 1 > 0 {
            self.view.0 -= 60. * dt;
        }
        if self.right_queue & 1 > 0 {
            self.view.0 += 60. * dt;
        }

        // Ensure the view is always inbounds.
        if self.view.0 < 16. {
            self.view.0 = 16.;
        }
        if self.view.1 < 16. {
            self.view.1 = 16.;
        }
    }

    pub fn postframe(
        &mut self,
        _timestamp: u64,
    ) -> (Option<RenderFrame>, impl Iterator<Item = NetEvent>) {
        // Outbound net events.
        let mut net_events = vec![];

        // Request from the server any chunks that may now be onscreen (Should client be the one to ask this?).
        super::functions::request_chunks_from_server(self.view, &mut self.chunks, &mut net_events);

        // Clone the visible tiles.
        let (tiles_x, tiles_y, foreground_tiles) =
            super::functions::clone_onscreen_tiles(self.view, &self.foreground_tiles);
        let (_, _, background_tiles) =
            super::functions::clone_onscreen_tiles(self.view, &self.background_tiles);

        // Generate light map
        let (light_x, light_y, light_map) = super::functions::create_light_map(
            self.view,
            &self.foreground_tiles,
            &self.background_tiles,
            &mut self.timer,
        );

        // Construct frame.
        let frame = self.exit.then(|| RenderFrame {
            view_x: self.view.0 as usize,
            view_y: self.view.1 as usize,
            view_w: self.view.2 as usize,
            view_h: self.view.3 as usize,

            tiles_x,
            tiles_y,
            foreground_tiles,
            background_tiles,

            light_x,
            light_y,
            light_map,
        });

        // Return.
        (frame, net_events.into_iter())
    }
}
