#![allow(clippy::all)]
#[cfg(target_os = "linux")]
include!(concat!(env!("OUT_DIR"), "/resolve1.rs"));
