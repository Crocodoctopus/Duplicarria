use crate::game::tile::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetEvent {
    // Connection
    Connect,
    Accept(u16, u16, u64), // world_w, world_h, player_x, player_y
    Ping,
    Disconnect,
    Close,

    // Humanoid.
    HumanoidUpdate(u64, f32, f32), // id, x, y

    // To server.
    RequestChunk(u16, u16),
    BreakForeground(u16, u16),
    BreakBackground(u16, u16),

    // To client.
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
