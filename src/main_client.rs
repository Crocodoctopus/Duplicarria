#![allow(dead_code)]
#![allow(unused_must_use)]

#[macro_use]
extern crate lazy_static;
extern crate bincode;
extern crate cgmath;
extern crate crossbeam_channel;
extern crate glutin;
#[macro_use]
extern crate serde;
extern crate ezgl;

//mod array2d;
mod array2d;
mod client;
mod io;
mod net;
mod server;
mod shared;
mod time;

use glutin::{
    dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder, Api, ContextBuilder, GlRequest,
};

fn main() {
    // Build window and event loop.
    let event_loop = EventLoop::new();
    let windowed_context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (4, 1)))
        .with_vsync(true)
        .build_windowed(
            WindowBuilder::new()
                .with_title("Trar")
                .with_inner_size(LogicalSize::new(1080f64, 720f64)),
            &event_loop,
        )
        .unwrap();

    // Create communication channels.
    let (input_send, input_recv) = crossbeam_channel::bounded(50);

    // Spawn client.
    let (update_handle, render_handle) = client::launch_client(windowed_context, input_recv);
    std::mem::forget(update_handle);
    std::mem::forget(render_handle);

    // Handle input (This call permanently hijacks main).
    event_loop.run(move |event, _, out| {
        use crate::client::input_event::*;
        use glutin::dpi::*;
        use glutin::event::*;
        use glutin::event_loop::*;

        *out = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Destroyed => *out = ControlFlow::Exit,
                WindowEvent::CloseRequested => input_send.send(InputEvent::Close).unwrap(),
                WindowEvent::Resized(PhysicalSize { width, height }) => input_send
                    .send(InputEvent::WindowResize(width as u16, height as u16))
                    .unwrap(),
                WindowEvent::Focused(state) => input_send.send(InputEvent::Focused(state)).unwrap(),

                // Mouse input
                WindowEvent::CursorMoved {
                    position: PhysicalPosition { x, y },
                    ..
                } => input_send
                    .send(InputEvent::CursorMove(x as f32, y as f32))
                    .unwrap(),
                WindowEvent::MouseInput { state, button, .. } => {
                    // Map button state.
                    let button_state = match state {
                        ElementState::Pressed => KeyState::Down,
                        ElementState::Released => KeyState::Up,
                    };

                    // Map button type.
                    let input_button = match button {
                        MouseButton::Left => InputKey::LeftClick,
                        MouseButton::Right => InputKey::RightClick,
                        MouseButton::Middle => InputKey::MiddleClick,
                        MouseButton::Other(v) => InputKey::MouseButton(v as u8),
                    };

                    // Send.
                    input_send
                        .send(InputEvent::KeyEvent(button_state, input_button))
                        .unwrap();
                }

                // Keyboard input.
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => {
                    // Map key state.
                    let key_state = match state {
                        ElementState::Pressed => KeyState::Down,
                        ElementState::Released => KeyState::Up,
                    };

                    // Map key type.
                    let input_key = match key {
                        VirtualKeyCode::W => InputKey::W,
                        VirtualKeyCode::A => InputKey::A,
                        VirtualKeyCode::S => InputKey::S,
                        VirtualKeyCode::D => InputKey::D,
                        _ => return,
                    };

                    // Send.
                    input_send
                        .send(InputEvent::KeyEvent(key_state, input_key))
                        .unwrap();
                }
                _ => {}
            },
            _ => {}
        }
    });
}
