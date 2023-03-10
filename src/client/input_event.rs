#![allow(dead_code)]

pub enum InputEvent {
    CursorMove(f32, f32),
    WindowResize(u16, u16),
    Focused(bool),
    KeyEvent(KeyState, InputKey),
    Close,
}

pub enum KeyState {
    Up,
    Down,
}

pub enum InputKey {
    W,
    A,
    S,
    D,
    Z,
    Left,
    Right,
    Up,
    Down,
    Space,
    LeftClick,
    RightClick,
    MiddleClick,
    MouseButton(u8),
}
