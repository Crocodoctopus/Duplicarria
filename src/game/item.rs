pub const ITEM_GRAVITY: f32 = 0.1;
pub const ITEM_MAX_VELOCITY: f32 = 16.;

#[derive(Copy, Clone, Debug)]
pub enum ItemId {
    Dirt,
    Stone,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: ItemId,
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
}

fn update_item_physics_x(item: &mut Item, ddx: f32) {
    item.dx += ddx;
    item.dx = item.dx.clamp(-ITEM_MAX_VELOCITY, ITEM_MAX_VELOCITY);
    item.x += item.dx;
}

fn update_item_physics_y(item: &mut Item, ddy: f32) {
    item.dy += ddy;
    item.dy = item.dy.clamp(-ITEM_MAX_VELOCITY, ITEM_MAX_VELOCITY);
    item.y += item.dy;
}
