#![allow(dead_code)]

use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    static ref ROOT: PathBuf = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
}

pub fn get_root() -> PathBuf {
    ROOT.clone()
}
