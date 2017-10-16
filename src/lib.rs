#![deny(missing_debug_implementations)]

#[macro_use]
extern crate bitflags;
extern crate jansson_sys;
extern crate janus_plugin_sys as ffi;

pub use debug::LogLevel;
pub use debug::log;
pub use ffi::JANUS_PLUGIN_API_VERSION as API_VERSION;
pub use ffi::janus_callbacks as PluginCallbacks;
pub use ffi::janus_plugin as Plugin;
pub use ffi::janus_plugin_result as PluginResultInfo;
pub use ffi::janus_plugin_result_type as PluginResultType;
pub use ffi::janus_plugin_session as PluginSession;
pub use jansson::{JanssonDecodingFlags, JanssonEncodingFlags, JanssonValue, RawJanssonValue};
pub use session::SessionWrapper;
use std::error::Error;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;

pub mod debug;
pub mod sdp;
pub mod session;
pub mod jansson;

/// Converts a Janus gateway result code to either success or a potential error.
pub fn get_result(error: i32) -> Result<(), Box<Error>> {
    match error {
        0 => Ok(()),
        e => {
            let msg = unsafe { CStr::from_ptr(ffi::janus_get_api_error(e)).to_str()? };
            Err(From::from(format!("{} (code: {})", msg, e)))
        }
    }
}

/// Allocates a Janus plugin result. Should be destroyed with destroy_result.
pub fn create_result(type_: PluginResultType, text: *const c_char, content: Option<&JanssonValue>) -> Box<PluginResultInfo> {
    let content_ptr = match content {
        Some(x) => x.ptr,
        None => ptr::null_mut(),
    };
    unsafe { Box::from_raw(ffi::janus_plugin_result_new(type_, text, content_ptr)) }
}

/// Destroys a Janus plugin result.
pub fn destroy_result(result: Box<PluginResultInfo>) {
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
        extern "C" fn get_api_compatibility() -> c_int { $crate::API_VERSION }
        extern "C" fn get_version() -> c_int { $md.version }
        extern "C" fn get_version_string() -> *const c_char { $md.version_str }
        extern "C" fn get_description() -> *const c_char { $md.description }
        extern "C" fn get_name() -> *const c_char { $md.name }
        extern "C" fn get_author() -> *const c_char { $md.author }
        extern "C" fn get_package() -> *const c_char { $md.package }
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
