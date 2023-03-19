pub const HUMANOID_GRAVITY: f32 = 0.1;
pub const HUMANOID_MAX_VELOCITY: f32 = 16.;
pub const HUMANOID_WIDTH: usize = 32 - 4;
pub const HUMANOID_HEIGHT: usize = 48 - 4;

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

#[derive(Copy, Clone, Debug)]
pub struct HumanoidPhysics {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub ddx: f32,
    pub ddy: f32,
    pub grounded: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum HumanoidAi {
    Player,
}

pub fn update_humanoid_physics_x(state: &mut HumanoidState, physics: &mut HumanoidPhysics) {
    let max_velocity = 3.;

    // Clamp velocity
    physics.dx += physics.ddx;
    physics.dx = physics.dx.clamp(-max_velocity, max_velocity);

    // Apply velocity to position
    physics.x += physics.dx;
}

pub fn update_humanoid_physics_y(state: &mut HumanoidState, physics: &mut HumanoidPhysics) {
    if matches!(state.action_state, HumanoidActionState::Jump) && physics.grounded {
        // Jump
        physics.dy = -5.0;
    }

    // Apply gravity to dy
    physics.dy += HUMANOID_GRAVITY;

    // Clamp velocity
    physics.dy = physics
        .dy
        .clamp(-HUMANOID_MAX_VELOCITY, HUMANOID_MAX_VELOCITY);

    // Apply velocity to position
    physics.y += physics.dy;
}
