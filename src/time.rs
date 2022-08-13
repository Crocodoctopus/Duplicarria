#![allow(dead_code)]

use std::time::Instant;

lazy_static! {
    static ref PROGRAM_START: Instant = Instant::now();
}

pub fn get_seconds_as_u64() -> u64 {
    let start = *PROGRAM_START;
    Instant::now().duration_since(start).as_secs()
}

pub fn get_milliseconds_as_u64() -> u64 {
    let start = *PROGRAM_START;
    Instant::now().duration_since(start).as_nanos() as u64 / 1_000_000
}

pub fn get_microseconds_as_u64() -> u64 {
    let start = *PROGRAM_START;
    Instant::now().duration_since(start).as_nanos() as u64 / 1_000
}

pub fn wait(time: u64) -> u64 {
    while get_microseconds_as_u64() < time {
        std::hint::spin_loop();
        std::thread::yield_now();
    }
    return get_microseconds_as_u64();
}
