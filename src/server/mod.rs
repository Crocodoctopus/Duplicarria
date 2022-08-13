mod server_state;

use std::net::UdpSocket;
use std::{thread, thread::JoinHandle};

use crate::time::*;
use self::server_state::*;

pub fn launch_server(port: u16) -> (u16, JoinHandle<()>) {
    // Create socket.
    let socket = UdpSocket::bind(("127.0.0.1", port)).unwrap();
    socket.set_nonblocking(true);
    let port = socket.local_addr().unwrap().port();

    // Spawn server update thread.
    let update_handle = thread::Builder::new()
        .name(String::from("server_update_thread"))
        .spawn(move || server_update_thread(socket))
        .unwrap();

    (port, update_handle)
}

pub fn server_update_thread(socket: UdpSocket) -> ! {
    println!("[Server] Update thread start.");
    let frametime = 16_666; // ns
    let mut timestamp = get_microseconds_as_u64();

    // Create server state.
    let mut server_state = ServerState::new(socket);

    while !server_state.kill {
        // Wait until enough has passed for at least 1 frame
        let next_timestamp = wait(timestamp + frametime);

        // Run preframe.
        server_state.preframe(timestamp);

        // Simulate the time between timestamp and next_timestamp:
        let frames = (next_timestamp - timestamp) / frametime;
        for _ in 0..frames {
            server_state.step(timestamp, frametime);
            timestamp += frametime;
        }

        // Run postframe.
        server_state.postframe(timestamp);
    }

    println!("[Server] Update thread closed.");
    std::process::exit(0);
}
