extern crate serde_json;
extern crate libc;

use jansson_sys;
use self::serde_json::value::Value as SerdeJsonValue;
use std::ops::Deref;
use std::os::raw::c_char;
use std::error::Error;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;
use std::slice;

pub type RawJanssonValue = jansson_sys::json_t;

/// A Jansson JSON value.
#[derive(Debug)]
pub struct JanssonValue {
    contents: *mut RawJanssonValue
}

impl JanssonValue {
    pub fn new(ptr: *mut RawJanssonValue) -> Self {
        Self { contents: ptr }
    }
}

impl Deref for JanssonValue {
    type Target = *mut RawJanssonValue;

    fn deref(&self) -> &*mut RawJanssonValue {
        &self.contents
    }
}

impl Clone for JanssonValue {
    fn clone(&self) -> Self {
        unsafe { jansson_sys::json_incref(self.contents) };
        Self::new(self.contents)
    }
}

impl Drop for JanssonValue {
    fn drop(&mut self) {
        unsafe { jansson_sys::json_decref(self.contents) }
    }
}


/// Inefficiently converts a Jansson JSON value to a serde JSON value.
pub fn to_serde_json(v: JanssonValue) -> Result<SerdeJsonValue, Box<Error>> {
    unsafe {
        let size = jansson_sys::json_dumpb(*v, ptr::null_mut(), 0, 0);
        let mut buffer = Vec::<u8>::with_capacity(size);
        let ptr = buffer.as_mut_ptr() as *mut c_char;
        jansson_sys::json_dumpb(*v, ptr, size, 0);
        buffer.set_len(size);
        Ok(serde_json::from_str(str::from_utf8(&buffer)?)?)
    }
}

/// Inefficiently converts a serde JSON value to a Jansson JSON value.
pub fn from_serde_json(v: SerdeJsonValue) -> Result<JanssonValue, Box<Error>> {
    unsafe {
        let json = CString::new(v.to_string())?.into_raw();
        let mut error: jansson_sys::json_error_t = mem::uninitialized();
        let result = jansson_sys::json_loads(json, 0, &mut error as *mut _);
        let _ = CString::from_raw(json); // reclaim & free memory
        if result.is_null() {
            let ptr = &error.text as *const _;
            let len = libc::strlen(ptr);
            let sli = slice::from_raw_parts(ptr as *mut u8, len);
            Err(From::from(str::from_utf8(sli)?))
        } else {
            Ok(JanssonValue::new(result))
        }


    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn round_trip() {
        let json = r#"{"a":"alpha","b":true,"c":false,"d":42,"e":1.25,"f":null,"g":[1,2,3]}"#;
        let serde = serde_json::from_str(json).unwrap();
        let jansson = from_serde_json(serde).unwrap();
        assert_eq!(json, to_serde_json(jansson).unwrap().to_string());
    }
}
