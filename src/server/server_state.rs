use array2d::*;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};

use crate::net::*;
use crate::shared::net_event::*;
use crate::shared::tile::*;

pub struct ServerState {
    pub kill: bool,
    socket: UdpSocket,
    broadcast: Vec<NetEvent>,
    connections: HashMap<SocketAddr, Vec<NetEvent>>,

    //
    tiles: Array2D<Tile>,
}

impl ServerState {
    pub fn new(socket: UdpSocket) -> Self {
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
            socket,
            broadcast: Vec::new(),
            connections: HashMap::new(),

            tiles,
        }
    }

    pub fn preframe(&mut self, _timestamp: u64) {
        for (event, addr) in recv_from(&self.socket) {
            // Handle connect requests first.
            if matches!(event, NetEvent::Connect) {
                self.connections.insert(addr, vec![NetEvent::Accept]);
                continue;
            }

            // Get connection, skip if not connected.
            let connection = match self.connections.get_mut(&addr) {
                Some(v) => v,
                None => continue,
            };

            println!("[Server] {event:?}");
            // Handle net message.
            match event {
                // Connection handling is done above.
                NetEvent::Connect => unreachable!(),

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

    pub fn postframe(&mut self, _timestamp: u64) {
        // Send all pending net messages, clearing the them in the process.
        use std::mem::replace;
        let broadcast = replace(&mut self.broadcast, Vec::new());
        for (&addr, events) in self.connections.iter_mut() {
            let events = replace(events, Vec::new());
            send_to(&self.socket, addr, events);
            send_to(&self.socket, addr, broadcast.iter().cloned());
        }
    }
}
