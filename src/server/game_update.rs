use array2d::*;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};

use crate::shared::net_event::*;
use crate::shared::tile::*;

pub struct GameUpdate {
    kill: bool,

    broadcast: Vec<NetEvent>,
    connections: HashMap<SocketAddr, Vec<NetEvent>>,

    //
    tiles: Array2D<Tile>,
}

impl GameUpdate {
    pub fn new() -> Self {
        // Create 1024 * 256 chunk world.
        let tiles = Array2D::from_closure(8 * 1024, 8 * 256, |x, y| {
            let h = (7.0 * ((x as f32 / 4.).sin() + 1.0) / 2.0) as usize + 15;
            if y < h {
                return Tile::None;
            }
            if y - h < 5 {
                return Tile::Dirt;
            }
            return Tile::Stone;
        });

        Self {
            kill: false,

            broadcast: Vec::new(),
            connections: HashMap::new(),

            tiles,
        }
    }

    pub fn preframe(
        &mut self,
        _timestamp: u64,
        net_events: impl Iterator<Item = (NetEvent, SocketAddr)>,
    ) {
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
            println!("[Server] {addr:?}: {event:?}");
            match event {
                // Connection handling is done above.
                NetEvent::Connect => {} // Redundant connect
                NetEvent::Disconnect => {
                    self.connections.remove(&addr);
                }

                NetEvent::Close => self.kill = true,

                // If a chunk is requested, send back in 8x8 chunks.
                NetEvent::RequestChunk(x, y) => {
                    let (xus, yus) = (x as usize, y as usize);
                    let chunk = self
                        .tiles
                        .clone_sub(8 * xus..8 * xus + 8, 8 * yus..8 * yus + 8)
                        .unwrap()
                        .into_raw();
                    connection.push(NetEvent::UpdateChunk(x, y, chunk));
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
            send_to(addr, self.broadcast.clone());
        }
        self.broadcast.clear();

        return self.kill;
    }
}
