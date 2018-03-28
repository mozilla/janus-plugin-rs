use jansson_sys::json_t;
use std::os::raw::{c_char, c_int, c_uint};

#[repr(C)]
#[derive(Debug)]
pub struct janus_eventhandler {
    pub init: unsafe extern "C" fn(config_path: *const c_char) -> c_int,
    pub destroy: unsafe extern "C" fn(),
    pub get_api_compatibility: unsafe extern "C" fn() -> c_int,
    pub get_version: unsafe extern "C" fn() -> c_int,
    pub get_version_string: unsafe extern "C" fn() -> *const c_char,
    pub get_description: unsafe extern "C" fn() -> *const c_char,
    pub get_name: unsafe extern "C" fn() -> *const c_char,
    pub get_author: unsafe extern "C" fn() -> *const c_char,
    pub get_package: unsafe extern "C" fn() -> *const c_char,
    pub incoming_event: unsafe extern "C" fn(event: *mut json_t),
    pub handle_request: unsafe extern "C" fn(request: *mut json_t) -> *mut json_t,
    pub events_mask: c_uint,
}
