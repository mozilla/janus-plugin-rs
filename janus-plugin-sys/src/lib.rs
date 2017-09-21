#![allow(non_camel_case_types)]
use std::os::raw::{c_char, c_int, c_void};

/// The Janus API version this library's plugins are compatible with.
pub const JANUS_PLUGIN_API_VERSION: c_int = 8;

/// A code representing the result status of a Janus event callback.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum janus_plugin_result_type {
    JANUS_PLUGIN_ERROR = -1,
    JANUS_PLUGIN_OK = 0,
    JANUS_PLUGIN_OK_WAIT = 1,
}

/// A map from Janus gateway sessions and Janus plugin sessions.
#[repr(C)]
#[derive(Debug)]
pub struct janus_plugin_session {
    pub gateway_handle: *mut c_void,
    pub plugin_handle: *mut c_void,
    pub _bitfield_1: u8,
    pub __bindgen_padding_0: [u8; 7usize],
}

/// The result of a Janus event callback.
#[repr(C)]
#[derive(Debug)]
pub struct janus_plugin_result {
    pub type_: janus_plugin_result_type,
    pub text: *const c_char,
    pub content: *mut json_t,
}

/// An interface by which plugins can send data back to the gateway.
#[repr(C)]
#[derive(Debug)]
pub struct janus_callbacks {
    pub push_event: extern "C" fn(handle: *mut janus_plugin_session, plugin: *mut janus_plugin, transaction: *const c_char, message: *mut json_t, jsep: *mut json_t) -> c_int,
    pub relay_rtp: extern "C" fn(handle: *mut janus_plugin_session, video: c_int, buf: *mut c_char, len: c_int),
    pub relay_rtcp: extern "C" fn(handle: *mut janus_plugin_session, video: c_int, buf: *mut c_char, len: c_int),
    pub relay_data: extern "C" fn(handle: *mut janus_plugin_session, buf: *mut c_char, len: c_int),
    pub close_pc: extern "C" fn(handle: *mut janus_plugin_session),
    pub end_session: extern "C" fn(handle: *mut janus_plugin_session),
    pub events_is_enabled: extern "C" fn() -> c_int,
    pub notify_event: extern "C" fn(plugin: *mut janus_plugin, handle: *mut janus_plugin_session, event: *mut json_t)
}

/// A  plugin descriptor that contains all of the event callbacks invokeable by the gateway.
#[repr(C)]
#[derive(Debug)]
pub struct janus_plugin {
    pub init: unsafe extern "C" fn(callback: *mut janus_callbacks, config_path: *const c_char) -> c_int,
    pub destroy: unsafe extern "C" fn(),
    pub get_api_compatibility: unsafe extern "C" fn() -> c_int,
    pub get_version: unsafe extern "C" fn() -> c_int,
    pub get_version_string: unsafe extern "C" fn() -> *const c_char,
    pub get_description: unsafe extern "C" fn() -> *const c_char,
    pub get_name: unsafe extern "C" fn() -> *const c_char,
    pub get_author: unsafe extern "C" fn() -> *const c_char,
    pub get_package: unsafe extern "C" fn() -> *const c_char,
    pub create_session: unsafe extern "C" fn(handle: *mut janus_plugin_session, error: *mut c_int),
    pub handle_message: unsafe extern "C" fn(handle: *mut janus_plugin_session, transaction: *mut c_char, message: *mut json_t, jsep: *mut json_t) -> *mut janus_plugin_result,
    pub setup_media: unsafe extern "C" fn(handle: *mut janus_plugin_session),
    pub incoming_rtp: unsafe extern "C" fn(handle: *mut janus_plugin_session, video: c_int, buf: *mut c_char, len: c_int),
    pub incoming_rtcp: unsafe extern "C" fn(handle: *mut janus_plugin_session, video: c_int, buf: *mut c_char, len: c_int),
    pub incoming_data: unsafe extern "C" fn(handle: *mut janus_plugin_session, buf: *mut c_char, len: c_int),
    pub slow_link: unsafe extern "C" fn(handle: *mut janus_plugin_session, uplink: c_int, video: c_int),
    pub hangup_media: unsafe extern "C" fn(handle: *mut janus_plugin_session),
    pub destroy_session: unsafe extern "C" fn(handle: *mut janus_plugin_session, error: *mut c_int),
    pub query_session: unsafe extern "C" fn(handle: *mut janus_plugin_session) -> *mut json_t,
}

/// The type of a RapidJSON value.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum json_type {
    JSON_OBJECT = 0,
    JSON_ARRAY = 1,
    JSON_STRING = 2,
    JSON_INTEGER = 3,
    JSON_REAL = 4,
    JSON_TRUE = 5,
    JSON_FALSE = 6,
    JSON_NULL = 7,
}

/// A RapidJSON value.
#[repr(C)]
#[derive(Debug)]
pub struct json_t {
    pub type_: json_type,
    pub refcount: usize,
}

extern "C" {
    pub fn janus_plugin_result_new(type_: janus_plugin_result_type, text: *const c_char, content: *mut json_t) -> *mut janus_plugin_result;
    pub fn janus_vprintf(format: *const c_char, ...);
    pub fn json_object() -> *mut json_t;
}
