use game::tile::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetEvent {
    // connection
    Connect,
    Accept(u16, u16), // world w/h (in chunks)
    Ping,
    Disconnect,
    Close,

    // To server
    RequestChunk(u16, u16),
    BreakForeground(u16, u16),
    BreakBackground(u16, u16),

    // To client
    UpdateForegroundTile(u16, u16, Tile),
    UpdateBackgroundTile(u16, u16, Tile),
    UpdateForegroundChunk(u16, u16, Box<[Tile]>),
    UpdateBackgroundChunk(u16, u16, Box<[Tile]>),

    // chat
    ChatMessage(String),
}

impl IntoIterator for NetEvent {
    type Item = Self;
    type IntoIter = std::iter::Once<Self>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}
