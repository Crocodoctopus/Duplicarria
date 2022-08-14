use super::tile::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetEvent {
    // connection
    Connect,
    Accept,
    Ping,
    Disconnect,
    Close,

    //
    RequestChunk(u16, u16),

    // To client
    UpdateChunk(u16, u16, Box<[Tile]>),

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
