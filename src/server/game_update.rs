use crate::array2d::*;
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;

use crate::game::humanoid::*;
use crate::game::item::*;
use crate::game::net::*;
use crate::game::tile::*;

pub struct GameUpdate {
    kill: bool,

    connections: HashMap<SocketAddr, Connection>,

    // Items.
    item_id_counter: u64,
    items: BTreeMap<u64, Item>,

    // Tiles.
    world_w: usize,
    world_h: usize,
    foreground_tiles: Array2D<Tile>,
    background_tiles: Array2D<Tile>,

    // Humanoids.
    humanoid_id_counter: u64,
    humanoids: BTreeMap<u64, Humanoid>,
}

impl GameUpdate {
    pub fn new() -> Self {
        // World size.
        let world_w = CHUNK_SIZE * 512;
        let world_h = CHUNK_SIZE * 128;

        // Create 1024 * 256 chunk world.
        let foreground_tiles = Array2D::from_closure(world_w, world_h, |x, y| {
            let h = (7.0 * ((x as f32 / 4.).sin() + 1.0) / 2.0) as usize + 15;
            if y < h {
                return Tile::None;
            }
            if y - h < 5 {
                return Tile::Dirt;
            }
            return Tile::Stone;
        });

        // Temp item.
        let item_id_counter = 1;
        let items = BTreeMap::from_iter(std::iter::once((
            0,
            Item {
                x: 256.,
                y: 32.,
                dx: 0.,
                dy: 0.,
                id: ItemId::Dirt,
            },
        )));

        Self {
            kill: false,

            connections: HashMap::new(),

            item_id_counter,
            items,

            world_w,
            world_h,
            background_tiles: foreground_tiles.clone_sub(0..world_w, 0..world_h).unwrap(),
            foreground_tiles,

            humanoid_id_counter: 0,
            humanoids: BTreeMap::new(),
        }
    }

    pub fn preframe(
        &mut self,
        timestamp_us: u64,
        net_events: impl Iterator<Item = (NetEvent, SocketAddr)>,
    ) {
        let timestamp_ms = timestamp_us / 1_000;
        let _timestamp_s = timestamp_us / 1_000_000;

        // Add NetEvent to all NetEvent vectors, except for the one that matches addr.
        let partial_broadcast = |connections: &mut HashMap<SocketAddr, Connection>,
                                 addr: SocketAddr,
                                 event: NetEvent| {
            connections
                .iter_mut()
                .filter(|(&k, _)| k != addr)
                .for_each(|(_, c)| {
                    c.net_events.push(event.clone());
                })
        };

        // Add NetEvent to all NetEvent vectors.
        let _broadcast = |connections: &mut HashMap<SocketAddr, Connection>,
                          _addr: SocketAddr,
                          event: NetEvent| {
            connections.iter_mut().for_each(|(_, c)| {
                c.net_events.push(event.clone());
            })
        };

        for (event, addr) in net_events {
            // Handle connect.
            if matches!(event, NetEvent::Connect) && !self.connections.contains_key(&addr) {
                // Get an id
                let humanoid_id = self.humanoid_id_counter;
                self.humanoid_id_counter += 1;

                // Create humanoid.
                let humanoid = Humanoid {
                    state: HumanoidState {
                        action_state: HumanoidActionState::Idle,
                        direction: HumanoidDirection::Right,
                        timestamp_ms: timestamp_ms as u16,
                    },
                    physics: HumanoidPhysics {
                        x: 32.,
                        y: 32.,
                        dx: 0.,
                        dy: 0.,
                        grounded: true,
                    },
                };

                // Create event vec with Accept event.
                let net_events = vec![NetEvent::Accept(
                    self.world_w as u16,
                    self.world_h as u16,
                    humanoid_id,
                )];

                // Establish connection.
                println!("[Server] {:?} has connected.", addr);
                self.humanoids.insert(humanoid_id, humanoid);
                let _connection = self.connections.entry(addr).or_insert_with(|| Connection {
                    last_msg: timestamp_ms as u16,
                    humanoid_id,
                    net_events,
                });

                continue;
            }

            // Get connection, skip if not connected.
            let Some(connection) = self.connections.get_mut(&addr) else {
                continue;
            };

            // Handle net message.
            connection.last_msg = timestamp_ms as u16;
            match event {
                NetEvent::Connect => {
                    // Connection handling is done above.
                    unreachable!()
                }
                NetEvent::Disconnect => {
                    self.connections.remove(&addr);
                }
                NetEvent::HumanoidUpdate(id, x, y) => {
                    let Some(humanoid) = self.humanoids.get_mut(&id) else {
                        continue;
                    };

                    humanoid.physics.x = x;
                    humanoid.physics.y = y;
                }
                NetEvent::Close => self.kill = true,
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
                    connection.net_events.push(NetEvent::UpdateForegroundChunk(
                        x,
                        y,
                        fg.into_raw(),
                    ));
                    connection.net_events.push(NetEvent::UpdateBackgroundChunk(
                        x,
                        y,
                        bg.into_raw(),
                    ));
                }
                NetEvent::BreakForeground(x, y) => {
                    match self.foreground_tiles.get_mut(x as _, y as _) {
                        Some(tile) => {
                            *tile = Tile::None;
                            partial_broadcast(
                                &mut self.connections,
                                addr,
                                NetEvent::UpdateForegroundTile(x, y, Tile::None),
                            );
                        }
                        None => {}
                    }
                }
                NetEvent::BreakBackground(x, y) => {
                    match self.background_tiles.get_mut(x as _, y as _) {
                        Some(tile) => {
                            *tile = Tile::None;
                            partial_broadcast(
                                &mut self.connections,
                                addr,
                                NetEvent::UpdateBackgroundTile(x, y, Tile::None),
                            );
                        }
                        None => {}
                    }
                }
                _ => {}
            }
        }

        // Cull connections if they haven't been heard from in 5 seconds
        self.connections.retain(|addr, connection| {
            if (timestamp_ms as u16).wrapping_sub(connection.last_msg) < 5_000 {
                return true;
            }

            // Remove all state associated with this key.
            println!("Disconnected {addr:?}.");
            self.humanoids.remove(&connection.humanoid_id);

            return false;
        });
    }

    pub fn step(&mut self, timestamp: u64, frametime: u64) {
        let dt = frametime as f32 / 1_000_000.;
    }

    pub fn postframe(
        &mut self,
        _timestamp: u64,
        send_to: impl Fn(SocketAddr, &Vec<NetEvent>) -> usize,
    ) -> bool {
        // Sync all humanoids with all players.
        for connection in &mut self.connections.values_mut() {
            let humanoids =
                BTreeMap::from_iter(self.humanoids.iter().map(|(k, v)| (*k, v.physics)));
            connection
                .net_events
                .push(NetEvent::HumanoidData(humanoids));
        }

        // Ping all connections.
        for connection in &mut self.connections.values_mut() {
            connection.net_events.push(NetEvent::Ping);
        }

        // Net stuff =/

        println!("#############");
        let mut sent = 0;
        for (&addr, connection) in self.connections.iter_mut() {
            if connection.net_events.len() > 0 {
                let s: String = format!("{:?}", connection.net_events)
                    .chars()
                    .take(200)
                    .collect();
                println!("[server] {s} sent to {addr:?}");
                sent += send_to(addr, &connection.net_events);
                connection.net_events.clear();
            }
        }
        println!("Total data sent: {sent} bytes");

        return self.kill;
    }
}

#[derive(Copy, Clone, Debug)]
struct Humanoid {
    state: HumanoidState,
    physics: HumanoidPhysics,
}

struct Connection {
    last_msg: u16,
    humanoid_id: u64, // the ID this connection owns
    net_events: Vec<NetEvent>,
}
