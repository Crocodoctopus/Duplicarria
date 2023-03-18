pub const HUMANOID_GRAVITY: f32 = 0.1;
pub const HUMANOID_MAX_VELOCITY: f32 = 16.;
pub const HUMANOID_WIDTH: usize = 32;
pub const HUMANOID_HEIGHT: usize = 48;

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
    pub grounded: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum HumanoidAi {
    Player,
}

pub fn update_humanoid_physics_x(state: &mut HumanoidState, physics: &mut HumanoidPhysics) {
    // Accelerate.
    let (ddx, max_velocity) = match (state.action_state, state.direction) {
        (HumanoidActionState::Run, HumanoidDirection::Left) if physics.grounded => {
            (-0.1, 1.0)
        },
        (HumanoidActionState::Run, HumanoidDirection::Right) if physics.grounded => {
            (0.1, 1.0)
        }
        _ => { (0.0, HUMANOID_MAX_VELOCITY) }
    };

    // Clamp velocity
    physics.dx += ddx;
    physics.dx = physics
        .dx
        .clamp(-max_velocity, max_velocity);

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
