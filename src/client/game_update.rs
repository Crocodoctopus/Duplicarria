use array2d::{Array2D, FastArray2D};
use std::collections::BTreeMap;

use super::game_frame::*;
use super::input_event::*;
use crate::common::*;

use crate::game::collision::*;
use crate::game::humanoid::*;
use crate::game::lighting::*;
use crate::game::net::*;
use crate::game::tile::*;

pub struct GameUpdate {
    // Misc:
    timer: usize,
    exit: bool,

    // Network:
    outbound: Vec<NetEvent>,
    chunks: FastArray2D<(u16, u16)>,

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

    // Humanoid
    player_id: u64,
    humanoids: BTreeMap<u64, Humanoid>,

    // Tiles:
    world_w: usize,
    world_h: usize,
    foreground_tiles: FastArray2D<Tile>,
    background_tiles: FastArray2D<Tile>,

    // Lighting:
    light_map_r: Array2D<u8>,
    light_map_g: Array2D<u8>,
    light_map_b: Array2D<u8>,
    fade_map: Array2D<u8>,
}

impl GameUpdate {
    pub fn new(view_w: f32, view_h: f32, world_w: u16, world_h: u16, player_id: u64) -> Self {
        let view_w = view_w as usize;
        let view_h = view_h as usize;
        let world_w = world_w as usize;
        let world_h = world_h as usize;

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
        let light_map_r = Array2D::from_closure(light_map_w, light_map_h, |_, _| MAX_BRIGHTNESS);
        let light_map_g = Array2D::from_closure(light_map_w, light_map_h, |_, _| MAX_BRIGHTNESS);
        let light_map_b = Array2D::from_closure(light_map_w, light_map_h, |_, _| MAX_BRIGHTNESS);
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

            view_pos: (0, 0),
            view_size: (view_w, view_h),

            outbound: Vec::new(),
            chunks,

            player_id,
            humanoids: BTreeMap::default(),

            world_w,
            world_h,
            foreground_tiles,
            background_tiles,

            light_map_r,
            light_map_g,
            light_map_b,
            fade_map,
        }
    }

    pub fn preframe(
        &mut self,
        timestamp_us: u64,
        input_events: impl Iterator<Item = InputEvent>,
        net_events: impl IntoIterator<Item = NetEvent>,
    ) {
        let timestamp_ms = timestamp_us / 1_000;
        let _timestamp_s = timestamp_us / 1_000_000;

        // Clear all outbound messages from last frame.
        self.outbound.clear();

        // Net loop.
        for net in net_events {
            match net {
                NetEvent::Ping => {
                    self.outbound.push(NetEvent::Ping);
                }
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
                NetEvent::UpdateForegroundTile(x, y, tile) => {
                    let (x, y) = (x as usize, y as usize);
                    let (chunk_x, chunk_y) = (x / CHUNK_SIZE, y / CHUNK_SIZE);
                    let verify = &(chunk_x as u16, chunk_y as u16)
                        == self.chunks.get_wrapping(chunk_x, chunk_y);
                    if verify {
                        *self.foreground_tiles.get_wrapping_mut(x, y) = tile;
                    }
                }
                NetEvent::UpdateBackgroundTile(x, y, tile) => {
                    let (x, y) = (x as usize, y as usize);
                    let (chunk_x, chunk_y) = (x / CHUNK_SIZE, y / CHUNK_SIZE);
                    let verify = &(chunk_x as u16, chunk_y as u16)
                        == self.chunks.get_wrapping(chunk_x, chunk_y);
                    if verify {
                        *self.background_tiles.get_wrapping_mut(x, y) = tile;
                    }
                }
                NetEvent::HumanoidUpdate(id, x, y) => {
                    // Create a new humanoid if one doesn't exist.
                    let humanoid = self.humanoids.entry(id).or_insert_with(|| Humanoid {
                        state: HumanoidState {
                            action_state: HumanoidActionState::Idle,
                            direction: HumanoidDirection::Right,
                            timestamp_ms: timestamp_ms as u16,
                        },
                        physics: HumanoidPhysics {
                            x,
                            y,
                            dx: 0.,
                            dy: 0.,
                            ddx: 0.,
                            ddy: 0.,
                            grounded: false,
                        },
                    });

                    // If the humanoid is the player, ignore the state update.
                    if id == self.player_id {
                        continue;
                    }

                    // Update.
                    (humanoid.physics.x, humanoid.physics.y) = (x, y);
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

    #[inline(always)]
    pub fn step(&mut self, timestamp_us: u64, frametime: u64) {
        let _dt = frametime as f32 / 1_000_000.;
        let _timestamp_ms = timestamp_us / 1_000;
        let _timestamp_s = timestamp_us / 1_000_000;

        // Center camera around humanoids (if it exists).
        if let Some(player) = self.humanoids.get(&self.player_id) {
            self.view_pos.0 = (player.physics.x as usize).saturating_sub(self.view_size.0 / 2);
            self.view_pos.1 = (player.physics.y as usize).saturating_sub(self.view_size.1 / 2);
        };

        // Currect the view if it is out of bounds.
        self.view_pos.0 = (self.view_pos.0 + self.view_size.0)
            .min((self.world_w * 8).saturating_sub(16))
            .saturating_sub(self.view_size.0)
            .max(16);
        self.view_pos.1 = (self.view_pos.1 + self.view_size.1)
            .min((self.world_h * 8).saturating_sub(16))
            .saturating_sub(self.view_size.1)
            .max(16);

        // Update player state.
        if let Some(player) = self.humanoids.get_mut(&self.player_id) {
            let left_cmd = self.left_queue & 0b1 == 0b1;
            let right_cmd = self.right_queue & 0b1 == 0b1;
            let jump_cmd = self.up_queue & 0b11 == 0b01;
            let grounded = player.physics.grounded;
            let _nodx = player.physics.dx.round() == 0.0;

            // Cancel all acceleration.
            player.physics.ddx = 0.;
            player.physics.ddy = 0.;

            // Move player right
            if right_cmd && !left_cmd {
                player.physics.ddx = 0.18;
            }
            // Move player left
            if left_cmd && !right_cmd {
                player.physics.ddx = -0.18;
            }
            // Else friction?
            if !left_cmd && !right_cmd {
                player.physics.ddx = -player.physics.dx * 0.1;
            }
            // Jump
            if jump_cmd && grounded {
                player.physics.dy = -5.0;
            }
        }

        // Player physics [TODO: make this neater]
        let mut tiles = Vec::with_capacity(6); // generic vec
        if let Some(player) = self.humanoids.get_mut(&self.player_id) {
            let player_state = &mut player.state;
            let player_physics = &mut player.physics;

            // Player Physics Y
            {
                // Upldate player physics
                let last_y = player_physics.y;
                update_humanoid_physics_y(player_state, player_physics);
                let dy = player_physics.y - last_y;
                let going_down = dy > 0.;

                // Calculate tiles that are now colliding with the player.
                let ty = collect_newly_colliding_tiles_y(
                    last_y,
                    player_physics.x,
                    player_physics.y,
                    HUMANOID_WIDTH as f32,
                    HUMANOID_HEIGHT as f32,
                    &self.foreground_tiles,
                    &mut tiles,
                );

                // Resolve colliding tiles
                {
                    player_physics.grounded = false; // assume player isn't grounded
                    for _ in &tiles {
                        // apply tile affect

                        // correct position
                        if going_down {
                            player_physics.y = (ty * TILE_SIZE - HUMANOID_HEIGHT) as f32;
                            player_physics.grounded = true;
                        } else {
                            player_physics.y = (ty * TILE_SIZE + TILE_SIZE) as f32;
                        }
                        player_physics.dy = 0.0;
                    }
                }
            }

            // Player Physics Y
            {
                // Upldate player physics
                let last_x = player_physics.x;
                update_humanoid_physics_x(player_state, player_physics);

                // Calculate tiles that are now colliding with the player.
                tiles.clear();
                let tx = collect_newly_colliding_tiles_x(
                    last_x,
                    player_physics.x,
                    player_physics.y,
                    HUMANOID_WIDTH as f32,
                    HUMANOID_HEIGHT as f32,
                    &self.foreground_tiles,
                    &mut tiles,
                );

                // Resolve colliding tiles
                {
                    let (mut corrected_x, mut corrected_dx) = (player_physics.x, player_physics.dx);
                    for _tile in tiles {
                        // apply tile affect

                        // correct position
                        if player_physics.dx > 0. {
                            corrected_x = (tx * TILE_SIZE - HUMANOID_WIDTH) as f32;
                        } else {
                            corrected_x = (tx * TILE_SIZE + TILE_SIZE) as f32;
                        }
                        corrected_dx = 0.0;
                    }
                    player_physics.x = corrected_x;
                    player_physics.dx = corrected_dx;
                }
            }
        }

        // Clear light map.
        let (w, h) = self.light_map_r.size();
        self.light_map_r
            .for_each_sub_wrapping_mut(1..w - 1, 1..h - 1, |_, _, t| *t = MIN_BRIGHTNESS);
        self.light_map_g
            .for_each_sub_wrapping_mut(1..w - 1, 1..h - 1, |_, _, t| *t = MIN_BRIGHTNESS);
        self.light_map_b
            .for_each_sub_wrapping_mut(1..w - 1, 1..h - 1, |_, _, t| *t = MIN_BRIGHTNESS);

        // Generate a fade map.
        let lights = gen_fade_map(
            self.view_pos,
            &self.foreground_tiles,
            &self.background_tiles,
            &mut self.fade_map,
            &mut self.light_map_r,
            &mut self.light_map_g,
            &mut self.light_map_b,
        );

        // Generate final light map.
        propogate_light_map_unbounded(&mut self.light_map_r, &self.fade_map, lights.clone());
        //propogate_light_map_unbounded(&mut self.light_map_g, &self.fade_map, lights.clone());
        //propogate_light_map_unbounded(&mut self.light_map_b, &self.fade_map, lights.clone());

        // Request from the server any chunks that may now be onscreen (Should client be the one to ask this?).
        request_chunks_from_server(
            self.view_pos,
            self.view_size,
            &mut self.chunks,
            &mut self.outbound,
        );

        // Send server player data
        if let Some(player) = self.humanoids.get(&self.player_id) {
            self.outbound.push(NetEvent::HumanoidUpdate(
                self.player_id,
                player.physics.x,
                player.physics.y,
            ));
        }
    }

    pub fn postframe(&mut self, _timestamp: u64) -> (Option<GameFrame>, &[NetEvent]) {
        // Clone the visible tiles
        const VISIBLE_TILE_BUFFER: usize = 2;
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
        let light_map_r = self
            .light_map_r
            .clone_sub(camx1 - lmx..camx2 - lmx, camy1 - lmy..camy2 - lmy)
            .unwrap();
        let light_map_g = self
            .light_map_g
            .clone_sub(camx1 - lmx..camx2 - lmx, camy1 - lmy..camy2 - lmy)
            .unwrap();
        let light_map_b = self
            .light_map_b
            .clone_sub(camx1 - lmx..camx2 - lmx, camy1 - lmy..camy2 - lmy)
            .unwrap();

        // Prepare player data.
        let humanoid_positions: Vec<(f32, f32)> = self
            .humanoids
            .values()
            .map(|h| (h.physics.x, h.physics.y))
            .collect();

        #[rustfmt::skip]
        let debug_text = {
            let left_queue = self.left_queue;
            let right_queue = self.right_queue;
            let up_queue = self.up_queue;
            let (world_w, world_h) = (self.world_w, self.world_h);
            let player_phys = self.humanoids.get(&self.player_id).map(|p| p.physics);
            format!("\
                player: {player_phys:.01?}\n\
                world size: {world_w}x{world_h}\n\
                left:  {left_queue:08b}\n\
                right: {right_queue:08b}\n\
                up:    {up_queue:08b}\n\
            ")
        };

        // Construct frame.
        let frame = (!self.exit).then(|| GameFrame {
            debug_text,

            view_x: self.view_pos.0,
            view_y: self.view_pos.1,
            view_w: self.view_size.0,
            view_h: self.view_size.1,

            humanoid_positions,

            tiles_x,
            tiles_y,
            foreground_tiles,
            background_tiles,

            light_x: camx1, // TEMP
            light_y: camy1, // TEMP
            light_map_r,
            light_map_g,
            light_map_b,
        });

        // Return.
        (frame, &self.outbound)
    }
}

#[derive(Copy, Clone, Debug)]
struct Humanoid {
    state: HumanoidState,
    physics: HumanoidPhysics,
}

//
pub fn request_chunks_from_server(
    (x, y): (usize, usize),
    (w, h): (usize, usize),
    chunks: &mut FastArray2D<(u16, u16)>,
    outbound: &mut Vec<NetEvent>,
) {
    const CHUNK_LOAD_BUFFER_SIZE_PX: usize = CHUNK_LOAD_BUFFER_SIZE * CHUNK_SIZE;
    const CHUNK_SIZE_PX: usize = TILE_SIZE * CHUNK_SIZE;
    let x1 = ifdiv(x.saturating_sub(CHUNK_LOAD_BUFFER_SIZE_PX), CHUNK_SIZE_PX);
    let y1 = ifdiv(y.saturating_sub(CHUNK_LOAD_BUFFER_SIZE_PX), CHUNK_SIZE_PX);
    let x2 = icdiv(x + w + CHUNK_LOAD_BUFFER_SIZE_PX, CHUNK_SIZE_PX);
    let y2 = icdiv(y + h + CHUNK_LOAD_BUFFER_SIZE_PX, CHUNK_SIZE_PX);
    chunks.for_each_sub_wrapping_mut(x1..x2, y1..y2, |x, y, cached_xy| {
        let new_xy = (x as u16, y as u16);
        if new_xy != *cached_xy {
            *cached_xy = new_xy;
            outbound.push(NetEvent::RequestChunk(x as u16, y as u16));
        }
    });
}
