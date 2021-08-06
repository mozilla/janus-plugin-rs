/// This modules defines stubs for functions from janus-plugin-sys crate to enable linking when
/// compiling for running unit tests.
use libc::c_void;
use std::os::raw::{c_char, c_int};

// lib.rs

#[no_mangle]
pub static refcount_debug: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn janus_vprintf(_format: *const c_char, _args: *mut c_void) {}
