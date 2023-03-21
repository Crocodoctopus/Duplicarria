pub const HUMANOID_GRAVITY: f32 = 9.8 * 16.;
pub const HUMANOID_MAX_VELOCITY: f32 = 900.; // Movement above 15 pixels per frame will probably
                                             // cause problems.
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

pub fn update_humanoid_physics_x(dt: f32, physics: &mut HumanoidPhysics, ddx: f32) {
    physics.x += 0.5 * ddx * dt * dt + physics.dx * dt;
    physics.dx += ddx * dt;
    physics.dx = physics
        .dx
        .clamp(-HUMANOID_MAX_VELOCITY, HUMANOID_MAX_VELOCITY);
}

pub fn update_humanoid_physics_y(dt: f32, physics: &mut HumanoidPhysics, ddy: f32) {
    physics.y += 0.5 * ddy * dt * dt + physics.dy * dt;
    physics.dy += ddy * dt;
    physics.dy = physics
        .dy
        .clamp(-HUMANOID_MAX_VELOCITY, HUMANOID_MAX_VELOCITY);
}
