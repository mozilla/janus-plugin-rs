#![deny(missing_debug_implementations)]

#[macro_use]
extern crate bitflags;
extern crate jansson_sys;
extern crate janus_plugin_sys as ffi;
extern crate glib_sys as glib;
extern crate libc;

pub use debug::LogLevel;
pub use debug::log;
pub use ffi::JANUS_PLUGIN_API_VERSION as API_VERSION;
pub use ffi::janus_callbacks as PluginCallbacks;
pub use ffi::janus_plugin as Plugin;
pub use ffi::janus_plugin_result as RawPluginResult;
pub use ffi::janus_plugin_result_type as PluginResultType;
pub use ffi::janus_plugin_session as PluginSession;
pub use jansson::{JanssonDecodingFlags, JanssonEncodingFlags, JanssonValue, RawJanssonValue};
pub use session::SessionWrapper;
use std::error::Error;
use std::ffi::CStr;
use std::mem;
use std::ops::Deref;
use std::os::raw::{c_char, c_int};
use std::ptr;

pub mod debug;
pub mod sdp;
pub mod session;
pub mod jansson;
pub mod utils;

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

/// A Janus plugin result; what a plugin returns to the gateway as a direct response to a signalling message.
#[derive(Debug)]
pub struct PluginResult {
    ptr: *mut RawPluginResult,
}

impl PluginResult {
    /// Creates a new plugin result.
    pub fn new(type_: PluginResultType, text: *const c_char, content: Option<JanssonValue>) -> Self {
        let content_ptr = match content {
            Some(x) => x.into_raw(),
            None => ptr::null_mut(),
        };
        Self { ptr: unsafe { ffi::janus_plugin_result_new(type_, text, content_ptr) } }
    }

    /// Transfers ownership of this result to the wrapped raw pointer. The consumer is responsible for calling
    /// janus_plugin_result_destroy on the pointer when finished.
    pub fn into_raw(self) -> *mut RawPluginResult {
        let ptr = self.ptr;
        mem::forget(self);
        ptr
    }
}

impl Deref for PluginResult {
    type Target = RawPluginResult;

    fn deref(&self) -> &RawPluginResult {
        unsafe { &*self.ptr }
    }
}

impl Drop for PluginResult {
    fn drop(&mut self) {
        unsafe { ffi::janus_plugin_result_destroy(self.ptr) }
    }
}

unsafe impl Send for PluginResult {}

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
