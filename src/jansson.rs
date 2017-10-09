extern crate libc;

use jansson_sys;
use std::ops::Deref;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::mem;
use std::str;
use std::slice;

/// A pointer to a raw Jansson value struct.
pub type RawJanssonValue = jansson_sys::json_t;

/// A safe wrapper for a Jansson JSON value. Automatically increases and decreases the refcount
/// of the underlying value when cloned/dropped.
#[derive(Debug)]
pub struct JanssonValue {
    pub ptr: *mut RawJanssonValue
}

impl JanssonValue {
    /// Creates a wrapper for the given Jansson value.
    pub fn new(ptr: *mut RawJanssonValue) -> Option<Self> {
        if ptr.is_null() { None } else { Some(Self { ptr: ptr }) }
    }

    /// Decodes a JSON string into a Jansson value, returning an error if decoding fails.
    pub fn from_str(input: &str, decoding_flags: usize) -> Result<Self, Box<Error+Send+Sync>> {
        Self::from_cstr(&CString::new(input)?, decoding_flags)
    }

    /// Encodes a Jansson value as a JSON string.
    pub fn to_string(self, encoding_flags: usize) -> String {
        self.to_cstring(encoding_flags).into_string().unwrap()
    }

    pub fn from_cstr(input: &CStr, decoding_flags: usize) -> Result<Self, Box<Error+Send+Sync>> {
        unsafe {
            let mut error: jansson_sys::json_error_t = mem::uninitialized();
            let result = jansson_sys::json_loads(input.as_ptr(), decoding_flags, &mut error as *mut _);
            if result.is_null() {
                let ptr = &error.text as *const _;
                let len = libc::strlen(ptr);
                let sli = slice::from_raw_parts(ptr as *mut u8, len);
                Err(From::from(str::from_utf8(sli)?))
            } else {
                Ok(JanssonValue::new(result).unwrap())
            }
        }
    }

    pub fn to_cstring(self, encoding_flags: usize) -> CString {
        unsafe {
            let output = jansson_sys::json_dumps(self.ptr, encoding_flags);
            let result = CStr::from_ptr(output).to_owned();
            libc::free(output as *mut _);
            result
        }
    }
}

impl Deref for JanssonValue {
    type Target = RawJanssonValue;

    fn deref(&self) -> &RawJanssonValue {
        unsafe { &*self.ptr }
    }
}

impl Clone for JanssonValue {
    fn clone(&self) -> Self {
        unsafe { jansson_sys::json_incref(self.ptr) };
        Self { ptr: self.ptr }
    }
}

impl Drop for JanssonValue {
    fn drop(&mut self) {
        unsafe { jansson_sys::json_decref(self.ptr) }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn round_trip() {
        let json = r#"{"a": "alpha", "b": true, "c": false, "d": 42, "e": 1.25, "f": null, "g": [1, 2, 3]}"#;
        assert_eq!(json, JanssonValue::from_str(json, 0).unwrap().to_string(0));
    }

    #[test]
    fn produce_jansson_errors() {
        let json = r#"{"a":"#;
        let result = JanssonValue::from_str(json, 0);
        assert!(result.is_err());
    }
}
