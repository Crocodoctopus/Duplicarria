#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
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
    let (_server_port, server_handle) = server::launch_server(0xCAFE);

    // Wait on server
    server_handle.join().unwrap();

    println!("");
}
