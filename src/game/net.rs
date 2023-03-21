use crate::game::humanoid::*;
use crate::game::item::*;
use crate::game::tile::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetEvent {
    // Connection
    Connect,
    Accept(u16, u16, u64), // world_w, world_h, player_x, player_y
    Ping,
    Disconnect,
    Close,

    // To server.
    RequestChunk(u16, u16),
    UpdateHumanoid(u64, HumanoidPhysics),
    BreakForeground(u16, u16),
    BreakBackground(u16, u16),

    // To client.
    HumanoidData(BTreeMap<u64, HumanoidPhysics>),
    ItemData(BTreeMap<u64, Item>),
    UpdateForegroundTile(u16, u16, Tile),
    UpdateBackgroundTile(u16, u16, Tile),
    UpdateForegroundChunk(u16, u16, Box<[Tile]>),
    UpdateBackgroundChunk(u16, u16, Box<[Tile]>),

    // Chat.
    ChatMessage(String),
}

impl IntoIterator for NetEvent {
    type Item = Self;
    type IntoIter = std::iter::Once<Self>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}
