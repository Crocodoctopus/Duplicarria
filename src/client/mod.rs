mod game_frame;
mod game_render;
mod game_update;
pub mod input_event;

use crossbeam_channel::{Receiver, Sender};
use glutin::{NotCurrent, WindowedContext};
use std::net::UdpSocket;
use std::{thread, thread::JoinHandle};

use self::game_frame::*;
use self::game_render::*;
use self::game_update::*;
use self::input_event::*;
use crate::game::net::*;
use crate::net::*;
use crate::time::*;

pub fn time<T>(t: &mut u64, mut f: impl FnMut() -> T) -> T {
    let start = crate::time::get_microseconds_as_u64();
    let out = f();
    *t += crate::time::get_microseconds_as_u64() - start;
    out
}

pub fn launch_client(
    windowed_context: WindowedContext<NotCurrent>,
    input_recv: Receiver<InputEvent>,
) -> (JoinHandle<()>, JoinHandle<()>) {
    let (render_send, render_recv) = crossbeam_channel::unbounded();

    // Spawn client update thread.
    let glutin::dpi::PhysicalSize { width, height } = windowed_context.window().inner_size();
    let update_handle = thread::Builder::new()
        .name(String::from("client_update_thread"))
        .spawn(move || client_update_thread(render_send, input_recv, (width as _, height as _)))
        .unwrap();

    // Spawn client render thread.
    let render_handle = thread::Builder::new()
        .name(String::from("client::render_thread"))
        .spawn(move || client_render_thread(windowed_context, render_recv))
        .unwrap();

    (update_handle, render_handle)
}

pub fn client_update_thread(
    render_send: Sender<GameFrame>,
    input_recv: Receiver<InputEvent>,
    (window_w, window_h): (f32, f32),
) {
    println!("[Client] Update thread start.");
    let frametime = 16_666; // us
    let mut timestamp = get_microseconds_as_u64();

    // Debug.
    let mut print_acc = 0;
    let mut preframe_us = 0u64;
    let mut step_us = 0u64;
    let mut postframe_us = 0u64;

    // Create client state.
    let mut game_update = GameUpdate::new(window_w, window_h);

    // Create a server (and connect).
    let server_port = 0xCAFE;
    let socket = UdpSocket::bind(("127.0.0.1", 0)).unwrap();
    socket.connect(("127.0.0.1", server_port));
    socket.set_nonblocking(true);
    send(&socket, NetEvent::Connect);

    loop {
        // Wait until enough has passed for at least 1 frame.
        let next_timestamp = wait(timestamp + frametime);

        // Run preframe.
        time(&mut preframe_us, || {
            game_update.preframe(timestamp, input_recv.try_iter(), recv(&socket).into_iter())
        });

        // Simulate the time between timestamp and next_timestamp.
        time(&mut step_us, || {
            let frames = (next_timestamp - timestamp) / frametime;
            for _ in 0..frames {
                game_update.step(timestamp, frametime);
                timestamp += frametime;
                print_acc += 1; // This occurs once every 16.666ms
            }
        });

        // Run postframe.
        let (frame, net_events) = time(&mut postframe_us, || game_update.postframe(timestamp));

        // Send net messages.
        send(&socket, net_events);

        // Send frame to render thread.
        match frame {
            Some(rs) => render_send.send(rs).unwrap(),
            None => break,
        };

        // Print fps.
        if print_acc > 5_000_000 / frametime {
            println!(
                "Frame: {:.03}ms\n  Preframe: {:.03}ms\n  Step: {:.03}ms\n  Postframe: {:.03}ms",
                (preframe_us + step_us + postframe_us) as f32 / (print_acc as f32 * 1000.),
                preframe_us as f32 / (print_acc as f32 * 1000.),
                step_us as f32 / (print_acc as f32 * 1000.),
                postframe_us as f32 / (print_acc as f32 * 1000.),
            );
            (print_acc, preframe_us, step_us, postframe_us) = (0, 0, 0, 0);
        }
    }

    // Send kill.
    send(&socket, NetEvent::Close);

    println!("[Client] Update thread closed.");
    return;
}

pub fn client_render_thread(
    windowed_context: WindowedContext<NotCurrent>,
    render_recv: Receiver<GameFrame>,
) {
    println!("[Client] Render thread start.");

    // Initialize context.
    let windowed_context = unsafe {
        let ctx = windowed_context.make_current().unwrap();
        ezgl::gl::load_with(|s| ctx.get_proc_address(s) as *const _);
        ezgl::gl::ClearColor(
            0x15 as f32 / 256.,
            0x9F as f32 / 256.,
            0xEA as f32 / 256.,
            1.,
        );
        ezgl::bind_vao();
        ctx
    };

    // Initialize render state.
    let mut game_render = unsafe { GameRender::new() };

    // Wait on current frame.
    let mut current_frame = render_recv.recv().unwrap();
    
    loop {
        // Get most recent frame.
        while !render_recv.is_empty() {
            current_frame = render_recv.recv().unwrap();
        }

        // Render frame.
        unsafe {
            game_render.render(&current_frame);
        }

        // Swap buffers.
        windowed_context.swap_buffers().unwrap();
    }

    println!("[Client] Render thread closed.");
    return;
}
