mod game_update;

use std::net::UdpSocket;
use std::{thread, thread::JoinHandle};

use self::game_update::*;
use crate::net::*;
use crate::time::*;

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

pub fn server_update_thread(socket: UdpSocket) {
    println!("[Server] Update thread start.");
    let frametime = 16_666; // us
    let mut timestamp = get_microseconds_as_u64();

    // Create server state.
    let mut game_update = GameUpdate::new();

    loop {
        // Wait until enough has passed for at least 1 frame
        let next_timestamp = wait(timestamp + frametime);

        // Run preframe.
        game_update.preframe(timestamp, recv_from(&socket).into_iter());

        // Simulate the time between timestamp and next_timestamp:
        let frames = (next_timestamp - timestamp) / frametime;
        for _ in 0..frames {
            game_update.step(timestamp, frametime);
            timestamp += frametime;
        }

        // Run postframe.
        use crate::game::net_event::NetEvent;
        let send_to_fn = |addr, net_events: Vec<NetEvent>| send_to(&socket, addr, net_events);
        if game_update.postframe(timestamp, send_to_fn) == true {
            break;
        }
    }

    println!("[Server] Update thread closed.");
    return;
}
