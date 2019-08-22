/// Utilities for working with Janus reference counts.
use super::ffi;
use super::glib;
use std::ffi::CString;
pub use ffi::janus_refcount as ReferenceCount;

/// Atomically increment the given reference count by 1.
pub fn increase(refcount: &ReferenceCount) {
    let field = &refcount.count;
    unsafe {
        if ffi::refcount_debug == 1 {
            let msg = CString::new(format!("[rust:increase] {:p} ({:?})\n", refcount, field + 1)).unwrap();
            ffi::janus_vprintf(msg.as_ptr());
        }
        glib::g_atomic_int_inc(field as *const _ as *mut _);
    }
}

/// Atomically decrement the given reference count by 1. If it's 0, call free.
pub fn decrease(refcount: &ReferenceCount) {
    let field = &refcount.count;
    unsafe {
        if ffi::refcount_debug == 1 {
            let msg = CString::new(format!("[rust:decrease] {:p} ({:?})\n", refcount, field - 1)).unwrap();
            ffi::janus_vprintf(msg.as_ptr());
        }
        if glib::g_atomic_int_dec_and_test(field as *const _ as *mut _) == 1 {
            (refcount.free)(refcount);
        }
    }
}
