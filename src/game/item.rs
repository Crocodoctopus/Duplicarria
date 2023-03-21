pub const ITEM_GRAVITY: f32 = 9.8 * 16.;
pub const ITEM_MAX_VELOCITY: f32 = 900.;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum ItemId {
    Dirt,
    Stone,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    pub id: ItemId,
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
}

pub fn update_item_physics_x(dt: f32, item: &mut Item, ddx: f32) {
    item.x += 0.5 * ddx * dt * dt + item.dx * dt;
    item.dx += ddx * dt;
    item.dx = item.dx.clamp(-ITEM_MAX_VELOCITY, ITEM_MAX_VELOCITY);
}

pub fn update_item_physics_y(dt: f32, item: &mut Item, ddy: f32) {
    item.y += 0.5 * ddy * dt * dt + item.dy * dt;
    item.dy += ddy * dt;
    item.dy = item.dy.clamp(-ITEM_MAX_VELOCITY, ITEM_MAX_VELOCITY);
}
