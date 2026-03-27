#![allow(unexpected_cfgs)]

use std::ffi::c_char;

#[path = "../browser/mod.rs"]
pub mod browser;
#[path = "../core/mod.rs"]
pub mod core;
#[path = "../network/mod.rs"]
pub mod network;
#[path = "../platform/mod.rs"]
pub mod platform;
#[path = "../privacy/mod.rs"]
pub mod privacy;
#[path = "../services/mod.rs"]
pub mod services;
#[path = "../storage/mod.rs"]
pub mod storage;
#[path = "../utils/mod.rs"]
pub mod utils;

pub mod api;

static NETRA_CONNECTION_MESSAGE: &[u8] = b"Netra Rust bridge connected\0";

/// Returns a simple integer used to prove the Rust DLL loaded successfully.
#[unsafe(no_mangle)]
pub extern "C" fn netra_connection_smoke_test() -> i32 {
    2026
}

/// Returns a static bridge message used by the Flutter smoke test.
#[unsafe(no_mangle)]
pub extern "C" fn netra_connection_message() -> *const c_char {
    NETRA_CONNECTION_MESSAGE.as_ptr().cast()
}
