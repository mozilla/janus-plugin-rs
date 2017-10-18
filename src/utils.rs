/// Wrapper types and helpers for working with the Janus FFI layer.

use super::glib;
use super::libc;
use std::ffi::CStr;
use std::ops::Deref;
use std::os::raw::c_char;

/// A C-style string which was allocated using glibc.
#[derive(Debug)]
pub struct GLibString {
    ptr: *const CStr,
}

impl GLibString {
    /// Creates a GLibString from a glibc-allocated pointer to a C-style string.
    pub unsafe fn from_chars(chars: *const c_char) -> Option<Self> {
        chars.as_ref().map(|c| Self { ptr: CStr::from_ptr(c) })
    }
}

impl Deref for GLibString {
    type Target = CStr;

    fn deref(&self) -> &CStr {
        unsafe { &*self.ptr }
    }
}

impl Drop for GLibString {
    fn drop(&mut self) {
        unsafe { glib::g_free(self.ptr as *mut _) }
    }
}

unsafe impl Send for GLibString {}
unsafe impl Sync for GLibString {}

/// A C-style string which was allocated using libc.
#[derive(Debug)]
pub struct LibcString {
    ptr: *const CStr,
}

impl LibcString {
    /// Creates a LibcString from a libc-allocated pointer to a C-style string.
    pub unsafe fn from_chars(chars: *const c_char) -> Option<Self> {
        chars.as_ref().map(|c| Self { ptr: CStr::from_ptr(c) })
    }
}

impl Deref for LibcString {
    type Target = CStr;

    fn deref(&self) -> &CStr {
        unsafe { &*self.ptr }
    }
}

impl Drop for LibcString {
    fn drop(&mut self) {
        unsafe { libc::free(self.ptr as *mut _) }
    }
}

unsafe impl Send for LibcString {}
unsafe impl Sync for LibcString {}
