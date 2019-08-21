use jansson_sys::json_t;
use std::os::raw::{c_char, c_int, c_void};

#[repr(C)]
#[derive(Debug)]
pub struct janus_callbacks {
    pub push_event: extern "C" fn(
        handle: *mut janus_plugin_session,
        plugin: *mut janus_plugin,
        transaction: *const c_char,
        message: *mut json_t,
        jsep: *mut json_t,
    ) -> c_int,
    pub relay_rtp: extern "C" fn(handle: *mut janus_plugin_session, video: c_int, buf: *mut c_char, len: c_int),
    pub relay_rtcp: extern "C" fn(handle: *mut janus_plugin_session, video: c_int, buf: *mut c_char, len: c_int),
    pub relay_data: extern "C" fn(handle: *mut janus_plugin_session, label: *mut c_char, buf: *mut c_char, len: c_int),
    pub close_pc: extern "C" fn(handle: *mut janus_plugin_session),
    pub end_session: extern "C" fn(handle: *mut janus_plugin_session),
    pub events_is_enabled: extern "C" fn() -> c_int,
    pub notify_event: extern "C" fn(plugin: *mut janus_plugin, handle: *mut janus_plugin_session, event: *mut json_t),
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum janus_plugin_result_type {
    JANUS_PLUGIN_ERROR = -1,
    JANUS_PLUGIN_OK = 0,
    JANUS_PLUGIN_OK_WAIT = 1,
}

#[cfg(not(feature="refcount"))]
#[repr(C)]
#[derive(Debug)]
pub struct janus_plugin_session {
    pub gateway_handle: *mut c_void,
    pub plugin_handle: *mut c_void,
    pub stopped_bitfield: u8, // todo: clean this up
    pub __padding: [u8; 7usize],
}

#[cfg(feature="refcount")]
#[repr(C)]
#[derive(Debug)]
pub struct janus_plugin_session {
    pub gateway_handle: *mut c_void,
    pub plugin_handle: *mut c_void,
    pub stopped: c_int,
    pub ref_: ::janus_refcount,
}

#[repr(C)]
#[derive(Debug)]
pub struct janus_plugin_result {
    pub type_: janus_plugin_result_type,
    pub text: *const c_char,
    pub content: *mut json_t,
}

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
    pub handle_message: unsafe extern "C" fn(
        handle: *mut janus_plugin_session,
        transaction: *mut c_char,
        message: *mut json_t,
        jsep: *mut json_t,
    ) -> *mut janus_plugin_result,
    pub handle_admin_message: unsafe extern "C" fn(message: *mut json_t) -> *mut json_t,
    pub setup_media: unsafe extern "C" fn(handle: *mut janus_plugin_session),
    pub incoming_rtp: unsafe extern "C" fn(handle: *mut janus_plugin_session, video: c_int, buf: *mut c_char, len: c_int),
    pub incoming_rtcp: unsafe extern "C" fn(handle: *mut janus_plugin_session, video: c_int, buf: *mut c_char, len: c_int),
    pub incoming_data: unsafe extern "C" fn(handle: *mut janus_plugin_session, label: *mut c_char, buf: *mut c_char, len: c_int),
    pub slow_link: unsafe extern "C" fn(handle: *mut janus_plugin_session, uplink: c_int, video: c_int),
    pub hangup_media: unsafe extern "C" fn(handle: *mut janus_plugin_session),
    pub destroy_session: unsafe extern "C" fn(handle: *mut janus_plugin_session, error: *mut c_int),
    pub query_session: unsafe extern "C" fn(handle: *mut janus_plugin_session) -> *mut json_t,
}

extern "C" {
    pub fn janus_plugin_result_new(type_: janus_plugin_result_type, text: *const c_char, content: *mut json_t) -> *mut janus_plugin_result;
    pub fn janus_plugin_result_destroy(result: *mut janus_plugin_result);
}
