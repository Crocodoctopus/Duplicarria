pub const HUMANOID_GRAVITY: f32 = 0.1;
pub const HUMANOID_MAX_VELOCITY: f32 = 16.;
pub const HUMANOID_WIDTH: usize = 32 - 4;
pub const HUMANOID_HEIGHT: usize = 48 - 4;

pub use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
pub enum HumanoidActionState {
    Idle,
    Run,
    Jump,
}

#[derive(Copy, Clone, Debug)]
pub enum HumanoidDirection {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
pub struct HumanoidState {
    pub action_state: HumanoidActionState,
    pub direction: HumanoidDirection,
    pub timestamp_ms: u16, // Last change timestamp
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct HumanoidPhysics {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub grounded: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum HumanoidAi {
    Player,
}

pub fn update_humanoid_physics_x(physics: &mut HumanoidPhysics, ddx: f32) {
    physics.dx += ddx;
    physics.dx = physics
        .dx
        .clamp(-HUMANOID_MAX_VELOCITY, HUMANOID_MAX_VELOCITY);
    physics.x += physics.dx;
}

pub fn update_humanoid_physics_y(physics: &mut HumanoidPhysics, ddy: f32) {
    physics.dy += ddy;
    physics.dy = physics
        .dy
        .clamp(-HUMANOID_MAX_VELOCITY, HUMANOID_MAX_VELOCITY);
    physics.y += physics.dy;
}
