#[derive(Copy, Clone, Debug)]
pub enum HumanoidRunState {
    Idle,
    Run,
    Jump,
}

#[derive(Copy, Clone, Debug)]
pub enum HumanoidUseState {
    None,
    Swing,
}

#[derive(Copy, Clone, Debug)]
pub enum HumanoidDirection {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug)]
pub struct HumanoidState {
    pub run_state: HumanoidRunState,
    pub use_state: HumanoidUseState,
    pub direction: HumanoidDirection,
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
