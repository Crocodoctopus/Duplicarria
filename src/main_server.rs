#[macro_use]
extern crate lazy_static;
extern crate bincode;
extern crate cgmath;
extern crate crossbeam_channel;
#[macro_use]
extern crate serde;

mod array2d;
mod common;
mod game;
mod io;
mod net;
mod server;
mod time;

fn main() {
    // Create a server
    let (server_port, server_handle) = server::launch_server(0xCAFE);

    // Wait on server
    server_handle.join().unwrap();

    println!("");
}
