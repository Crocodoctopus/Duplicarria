struct HumanoidState {
    enum State {
        Idle,
        Run,
        Jump,
    }

    state: State,
    pos: (f32, f32),
}
