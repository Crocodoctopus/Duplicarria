#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[repr(u8)]
pub enum Tile {
    None = 0,
    Dirt = 1,
    Stone = 2,
}
