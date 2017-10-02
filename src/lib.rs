#![deny(missing_debug_implementations)]

extern crate jansson_sys;
extern crate janus_plugin_sys as ffi;

pub use debug::LogLevel;
pub use debug::log;
pub use ffi::JANUS_PLUGIN_API_VERSION as API_VERSION;
pub use ffi::janus_callbacks as PluginCallbacks;
pub use ffi::janus_plugin as Plugin;
pub use ffi::janus_plugin_result as PluginResult;
pub use ffi::janus_plugin_result_type as PluginResultType;
pub use ffi::janus_plugin_session as PluginSession;
pub use jansson_sys::json_t as Json;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

pub mod debug;
pub mod sdp;

/// Converts a Janus gateway error code to an error message.
pub fn get_api_error(error: i32) -> &'static str {
    unsafe {
        CStr::from_ptr(ffi::janus_get_api_error(error))
            .to_str()
            .unwrap()
    }
}

/// Allocates a Janus plugin result. Should be destroyed with destroy_result.
pub fn create_result(type_: PluginResultType, text: *const c_char, content: *mut Json) -> Box<PluginResult> {
    unsafe { Box::from_raw(ffi::janus_plugin_result_new(type_, text, content)) }
}

/// Destroys a Janus plugin result.
pub fn destroy_result(result: Box<PluginResult>) {
    unsafe { ffi::janus_plugin_result_destroy(Box::into_raw(result)) }
}

#[derive(Debug)]
/// Represents metadata about this plugin which Janus can query at runtime.
pub struct PluginMetadata {
    pub version: c_int,
    pub version_str: *const c_char,
    pub description: *const c_char,
    pub name: *const c_char,
    pub author: *const c_char,
    pub package: *const c_char,
}

/// Helper macro to produce a Janus plugin instance. Should be called with
/// a PluginMetadata instance and a series of exported plugin event handlers.
#[macro_export]
macro_rules! build_plugin {
    ($md:expr, $($cb:ident),*) => {{
        extern fn get_api_compatibility() -> c_int { $crate::API_VERSION }
        extern fn get_version() -> c_int { $md.version }
        extern fn get_version_string() -> *const c_char { $md.version_str }
        extern fn get_description() -> *const c_char { $md.description }
        extern fn get_name() -> *const c_char { $md.name }
        extern fn get_author() -> *const c_char { $md.author }
        extern fn get_package() -> *const c_char { $md.package }
        $crate::Plugin {
            get_api_compatibility,
            get_version,
            get_version_string,
            get_description,
            get_name,
            get_author,
            get_package,
            $($cb,)*
        }
    }}
}

/// Macro to export a Janus plugin instance from this module.
#[macro_export]
macro_rules! export_plugin {
    ($pl:expr) => {
        /// Called by Janus to create an instance of this plugin, using the provided callbacks to dispatch events.
        #[no_mangle]
        pub extern "C" fn create() -> *const $crate::Plugin { $pl }
    }
}
