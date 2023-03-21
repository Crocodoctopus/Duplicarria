#![allow(dead_code)]

use lazy_static::lazy_static;
use std::time::{Duration, Instant};

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
    // Sleep for the duration, with a buffer
    let buffer = 2_000; // us
    std::thread::sleep(Duration::from_micros(
        time.saturating_sub(get_microseconds_as_u64())
            .saturating_sub(buffer),
    ));

    // Spin for the remaining time
    while get_microseconds_as_u64() < time {
        std::hint::spin_loop();
        std::thread::yield_now();
    }

    // Return the current time, which should be close to ``time``
    return get_microseconds_as_u64();
}
