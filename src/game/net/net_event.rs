use game::tile::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetEvent {
    // connection
    Connect,
    Accept,
    Ping,
    Disconnect,
    Close,

    // To server
    RequestChunk(u16, u16),
    BreakForeground(u16, u16),
    BreakBackground(u16, u16),

    // To client
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
