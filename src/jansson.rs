/// Utilities to work with Jansson JSON values, which are exposed in the Janus plugin API.
extern crate libc;

use jansson_sys;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::mem;
use std::ops::Deref;
use std::slice;
use std::str;

/// A pointer to a raw Jansson value struct.
pub type RawJanssonValue = jansson_sys::json_t;

bitflags! {
    /// Flags that can be passed to JSON decoding functions.
    pub struct JanssonDecodingFlags: usize {
        const JSON_REJECT_DUPLICATES = 0x0001;
        const JSON_DISABLE_EOF_CHECK = 0x0002;
        const JSON_DECODE_ANY = 0x0004;
        const JSON_DECODE_INT_AS_REAL = 0x0008;
        const JSON_ALLOW_NUL = 0x0010;
    }
}

bitflags! {
    /// Flags that can be passed to JSON encoding functions.
    pub struct JanssonEncodingFlags: usize {
        const JSON_COMPACT = 0x0020;
        const JSON_ENSURE_ASCII = 0x0040;
        const JSON_SORT_KEYS = 0x0080;
        const JSON_PRESERVE_ORDER = 0x0100;
        const JSON_ENCODE_ANY = 0x0200;
        const JSON_ESCAPE_SLASH = 0x0400;
        const JSON_EMBED = 0x0800;
    }
}

/// A safe wrapper for a Jansson JSON value. Automatically increases and decreases the refcount
/// of the underlying value when cloned/dropped.
#[derive(Debug)]
pub struct JanssonValue {
    pub ptr: *mut RawJanssonValue,
}

impl JanssonValue {
    /// Creates a wrapper for the given Jansson value.
    pub fn new(ptr: *mut RawJanssonValue) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr: ptr })
        }
    }

    /// Decodes a JSON string into a Jansson value, returning an error if decoding fails.
    pub fn from_str(input: &str, decoding_flags: JanssonDecodingFlags) -> Result<Self, Box<Error + Send + Sync>> {
        Self::from_cstr(&CString::new(input)?, decoding_flags)
    }

    pub fn from_cstr(input: &CStr, decoding_flags: JanssonDecodingFlags) -> Result<Self, Box<Error + Send + Sync>> {
        unsafe {
            let mut error: jansson_sys::json_error_t = mem::uninitialized();
            let result = jansson_sys::json_loads(input.as_ptr(), decoding_flags.bits(), &mut error as *mut _);
            match Self::new(result) {
                Some(val) => Ok(val),
                None => {
                    let ptr = &error.text as *const _;
                    let len = libc::strlen(ptr);
                    let sli = slice::from_raw_parts(ptr as *mut u8, len);
                    Err(From::from(str::from_utf8(sli)?))
                }
            }
        }
    }

    pub fn to_string(self, encoding_flags: JanssonEncodingFlags) -> String {
        let cstring = self.to_libcstring(encoding_flags);
        cstring.to_str().expect("Null bytes in Jansson output; what's going on?").to_owned()
    }

    pub fn to_libcstring(self, encoding_flags: JanssonEncodingFlags) -> LibcString {
        unsafe {
            let output = jansson_sys::json_dumps(self.ptr, encoding_flags.bits());
            LibcString { contents: CStr::from_ptr(output) as *const _ as *mut _ }
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

/// A C-style string which was allocated using libc.
#[derive(Debug)]
pub struct LibcString {
    contents: *mut CStr,
}

impl Deref for LibcString {
    type Target = CStr;

    fn deref(&self) -> &CStr {
        unsafe { &*self.contents }
    }
}

impl Drop for LibcString {
    fn drop(&mut self) {
        unsafe { libc::free(self.contents as *mut _) }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn round_trip() {
        let json = r#"{"a": "alpha", "b": true, "c": false, "d": 42, "e": 1.25, "f": null, "g": [1, 2, 3]}"#;
        let result = JanssonValue::from_str(json, JanssonDecodingFlags::empty()).unwrap();
        assert_eq!(json, result.to_string(JanssonEncodingFlags::empty()));
    }

    #[test]
    fn produce_jansson_errors() {
        let json = r#"{"a":"#;
        let result = JanssonValue::from_str(json, JanssonDecodingFlags::empty());
        assert!(result.is_err());
    }
}
