use crate::array2d::*;
use std::collections::HashMap;
use std::net::SocketAddr;

use crate::game::net::*;
use crate::game::tile::*;

pub struct GameUpdate {
    kill: bool,

    connections: HashMap<SocketAddr, Vec<NetEvent>>,

    //
    foreground_tiles: Array2D<Tile>,
    background_tiles: Array2D<Tile>,
    //
    //light_map: Array2D<u8>,
}

impl GameUpdate {
    pub fn new() -> Self {
        // Create 1024 * 256 chunk world.
        let foreground_tiles = Array2D::from_closure(CHUNK_SIZE * 512, CHUNK_SIZE * 128, |x, y| {
            let h = (7.0 * ((x as f32 / 4.).sin() + 1.0) / 2.0) as usize + 15;
            if y < h {
                return Tile::None;
            }
            if y - h < 5 {
                return Tile::Dirt;
            }
            return Tile::Stone;
        });

        // Create light map

        Self {
            kill: false,

            connections: HashMap::new(),

            background_tiles: foreground_tiles
                .clone_sub(0..CHUNK_SIZE * 512, 0..CHUNK_SIZE * 128)
                .unwrap(),
            foreground_tiles,
        }
    }

    pub fn preframe(
        &mut self,
        _timestamp: u64,
        net_events: impl Iterator<Item = (NetEvent, SocketAddr)>,
    ) {
        // Add NetEvent to all NetEvent vectors, except for the one that matches addr
        let partial_broadcast = |connections: &mut HashMap<SocketAddr, Vec<NetEvent>>,
                                 addr: SocketAddr,
                                 event: NetEvent| {
            connections
                .iter_mut()
                .filter(|(&k, _)| k == addr)
                .for_each(|(_, vec)| {
                    vec.push(event.clone());
                })
        };

        for (event, addr) in net_events {
            // Handle connect.
            if matches!(event, NetEvent::Connect) && !self.connections.contains_key(&addr) {
                self.connections.insert(addr, vec![NetEvent::Accept]);
                println!("[Server] {:?} has connected.", addr);
                continue;
            }

            // Get connection, skip if not connected.
            let connection = match self.connections.get_mut(&addr) {
                Some(v) => v,
                None => continue,
            };

            // Handle net message.
            match event {
                // Connection handling is done above.
                NetEvent::Connect => {} // Redundant connect
                NetEvent::Disconnect => {
                    self.connections.remove(&addr);
                }

                NetEvent::Close => self.kill = true,

                // On chunk request
                NetEvent::RequestChunk(x, y) => {
                    let xr = CHUNK_SIZE * x as usize..CHUNK_SIZE * (x as usize + 1);
                    let yr = CHUNK_SIZE * y as usize..CHUNK_SIZE * (y as usize + 1);
                    let fg = self
                        .foreground_tiles
                        .clone_sub(xr.clone(), yr.clone())
                        .unwrap();
                    let bg = self
                        .background_tiles
                        .clone_sub(xr.clone(), yr.clone())
                        .unwrap();
                    connection.push(NetEvent::UpdateForegroundChunk(x, y, fg.into_raw()));
                    connection.push(NetEvent::UpdateBackgroundChunk(x, y, bg.into_raw()));
                }

                //
                NetEvent::BreakForeground(x, y) => {
                    match self.foreground_tiles.get_mut(x as _, y as _) {
                        Some(x) => {
                            *x = Tile::None;
                            partial_broadcast(&mut self.connections, addr, event);
                        }
                        None => {}
                    }
                }

                //
                NetEvent::BreakBackground(x, y) => {
                    match self.background_tiles.get_mut(x as _, y as _) {
                        Some(x) => {
                            *x = Tile::None;
                            partial_broadcast(&mut self.connections, addr, event);
                        }
                        None => {}
                    }
                }

                _ => {}
            }
        }
    }

    pub fn step(&mut self, _timestamp: u64, _frametime: u64) {}

    pub fn postframe(
        &mut self,
        _timestamp: u64,
        send_to: impl Fn(SocketAddr, Vec<NetEvent>),
    ) -> bool {
        use std::mem::take;
        for (&addr, events) in self.connections.iter_mut() {
            send_to(addr, take(events));
        }

        return self.kill;
    }
}
