use super::tile::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetEvent {
    // connection
    Connect,
    Accept,
    Ping,
    Disconnect,

    //
    RequestChunk(u16, u16),

    // To client
    UpdateChunk(u16, u16, Box<[Tile]>),

    // chat
    ChatMessage(String),
}
