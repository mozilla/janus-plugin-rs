extern crate serde_json;

use jansson_sys;
pub use jansson_sys::json_t as JanssonValue;
use self::serde_json::value::Value as SerdeJsonValue;
use std::os::raw::c_char;
use std::error::Error;
use std::ffi::CString;
use std::ptr;

/// Inefficiently converts a Jansson JSON value to a serde JSON value.
pub fn to_serde_json(v: *mut JanssonValue) -> Result<SerdeJsonValue, Box<Error>> {
    unsafe {
        let size = jansson_sys::json_dumpb(v, ptr::null_mut(), 0, 0);
        let mut buffer = Vec::<u8>::with_capacity(size);
        let ptr = buffer.as_mut_ptr() as *mut c_char;
        jansson_sys::json_dumpb(v, ptr, size, 0);
        buffer.set_len(size);
        Ok(serde_json::from_str(&String::from_utf8(buffer)?)?)
    }
}

/// Inefficiently converts a serde JSON value to a Jansson JSON value.
pub fn from_serde_json(v: SerdeJsonValue) -> Result<*mut JanssonValue, Box<Error>> {
    unsafe {
        let json = CString::new(v.to_string())?.into_raw();
        let result = jansson_sys::json_loads(json, 0, ptr::null_mut());
        let _ = CString::from_raw(json); // reclaim & free memory
        Ok(result)
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
