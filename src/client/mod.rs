mod client_state;
mod functions;
pub mod input_event;
mod render_frame;
mod render_state;

use crossbeam_channel::{Receiver, Sender};
use glutin::{NotCurrent, WindowedContext};
use std::net::UdpSocket;
use std::{thread, thread::JoinHandle};

use crate::net::*;
use crate::time::*;
use self::client_state::*;
use self::input_event::*;
use self::render_frame::*;
use self::render_state::*;

pub fn launch_client(
    windowed_context: WindowedContext<NotCurrent>,
    input_recv: Receiver<InputEvent>,
) -> (JoinHandle<()>, JoinHandle<()>) {
    let (render_send, render_recv) = crossbeam_channel::bounded(1);

    // Spawn client update thread.
    let update_handle = thread::Builder::new()
        .name(String::from("client_update_thread"))
        .spawn(move || client_update_thread(render_send, input_recv))
        .unwrap();

    // Spawn client render thread.
    let render_handle = thread::Builder::new()
        .name(String::from("client::render_thread"))
        .spawn(move || client_render_thread(windowed_context, render_recv))
        .unwrap();

    (update_handle, render_handle)
}

pub fn client_update_thread(
    render_send: Sender<RenderFrame>,
    input_recv: Receiver<InputEvent>,
) -> ! {
    println!("[Client] Update thread start.");
    let frametime = 16_666; // ns
    let mut timestamp = get_microseconds_as_u64();

    // Debug.
    let mut print_acc = 0;
    let mut preframe_us = 0.;
    let mut step_us = 0.;
    let mut postframe_us = 0.;
    let mut last_print = get_microseconds_as_u64();

    // Create client state.
    let mut client_state = ClientState::new();

    // Create a server (and connect).
    let (server_port, server_handle) = crate::server::launch_server(0);
    let socket = UdpSocket::bind(("127.0.0.1", 0)).unwrap();
    socket.connect(("127.0.0.1", server_port));
    socket.set_nonblocking(true);
    send(&socket, vec![crate::shared::net_event::NetEvent::Connect]);

    loop {
        // Wait until enough has passed for at least 1 frame.
        let next_timestamp = wait(timestamp + frametime);

        // Preframe timing group:
        {
            let start = crate::time::get_microseconds_as_u64();

            // Run preframe.
            client_state.preframe(timestamp, input_recv.try_iter(), recv(&socket).into_iter());

            preframe_us += (crate::time::get_microseconds_as_u64() - start) as f32 / 1000.;
        }

        // Step timing group:
        {
            let start = crate::time::get_microseconds_as_u64();

            // Simulate the time between timestamp and next_timestamp.
            let frames = (next_timestamp - timestamp) / frametime;
            for _ in 0..frames {
                client_state.step(timestamp, frametime);
                timestamp += frametime;
            }

            step_us += (crate::time::get_microseconds_as_u64() - start) as f32 / 1000.;
        }

        // Postframe timing group:
        {
            let start = crate::time::get_microseconds_as_u64();

            // Run postframe.
            let (frame, net_events) = client_state.postframe(timestamp);

            // Send frame to render thread.
            match frame {
                Some(rs) => render_send.send(rs).unwrap(),
                None => break,
            };

            // Send net events to server.
            send(&socket, net_events);

            postframe_us += (crate::time::get_microseconds_as_u64() - start) as f32 / 1000.;
        };

        // Print fps.
        print_acc += 1;
        if timestamp - last_print > 5000000 {
            last_print = timestamp;
            println!(
                "Frame: {:.03}ms\n  Preframe: {:.03}ms\n  Step: {:.03}ms\n  Postframe: {:.03}ms",
                (preframe_us + step_us + postframe_us) / print_acc as f32,
                preframe_us / print_acc as f32,
                step_us / print_acc as f32,
                postframe_us / print_acc as f32,
            );
            print_acc = 0;
            preframe_us = 0.;
            step_us = 0.;
            postframe_us = 0.;
        }
    }

    // Wait for server shutdown.
    server_handle.join();

    println!("[Client] Update thread closed.");
    std::process::exit(0);
}

pub fn client_render_thread(
    windowed_context: WindowedContext<NotCurrent>,
    render_recv: Receiver<RenderFrame>,
) -> ! {
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
    let mut render_state = RenderState::new();

    for frame in render_recv.iter().skip(1) {
        // Render frame.
        unsafe {
            ezgl::gl::Clear(ezgl::gl::COLOR_BUFFER_BIT | ezgl::gl::DEPTH_BUFFER_BIT);
        }
        render_state.render(frame);
        windowed_context.swap_buffers().unwrap();
    }

    println!("[Client] Render thread closed.");
    std::process::exit(0);
}
