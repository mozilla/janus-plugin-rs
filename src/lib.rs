#![deny(missing_debug_implementations)]

#[macro_use]
extern crate bitflags;
extern crate jansson_sys;
extern crate janus_plugin_sys as ffi;
extern crate glib_sys as glib;
extern crate libc;
extern crate serde;

pub use debug::LogLevel;
pub use debug::log;
pub use ffi::JANUS_PLUGIN_API_VERSION as API_VERSION;
pub use ffi::janus_callbacks as PluginCallbacks;
pub use ffi::janus_plugin as Plugin;
pub use ffi::janus_plugin_result as RawPluginResult;
pub use ffi::janus_plugin_session as PluginSession;
pub use jansson::{JanssonDecodingFlags, JanssonEncodingFlags, JanssonValue, RawJanssonValue};
pub use session::SessionWrapper;
use ffi::janus_plugin_result_type as PluginResultType;
use std::error::Error;
use std::fmt;
use std::ffi::CStr;
use std::mem;
use std::ops::Deref;
use std::os::raw::{c_char, c_int};
use std::ptr;

pub mod debug;
pub mod rtcp;
pub mod sdp;
pub mod session;
pub mod jansson;
pub mod utils;

#[cfg(feature="refcount")]
pub mod refcount;

/// An error emitted by the Janus core in response to a plugin.
#[derive(Debug, Clone, Copy)]
pub struct JanusError(pub i32);

impl JanusError {
    /// Returns Janus's description text for this error.
    pub fn to_cstr(&self) -> &'static CStr {
        unsafe { CStr::from_ptr(ffi::janus_get_api_error(self.0)) }
    }
}

impl Error for JanusError {
    fn description(&self) -> &'static str {
        self.to_cstr().to_str().unwrap()
    }
}

impl fmt::Display for JanusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (code: {})", self.description(), self.0)
    }
}

/// Converts a Janus gateway result code to either success or a potential error.
pub fn get_result(error: i32) -> Result<(), JanusError> {
    match error {
        0 => Ok(()),
        e => Err(JanusError(e))
    }
}

/// A Janus plugin result; what a plugin returns to the gateway as a direct response to a signalling message.
#[derive(Debug)]
pub struct PluginResult {
    ptr: *mut RawPluginResult,
}

impl PluginResult {
    /// Creates a new plugin result.
    pub unsafe fn new(type_: PluginResultType, text: *const c_char, content: *mut RawJanssonValue) -> Self {
        Self { ptr: ffi::janus_plugin_result_new(type_, text, content) }
    }

    /// Creates a plugin result indicating a synchronously successful request. The provided response
    /// JSON will be passed back to the client.
    pub fn ok(response: JanssonValue) -> Self {
        unsafe { Self::new(PluginResultType::JANUS_PLUGIN_OK, ptr::null(), response.into_raw()) }
    }

    /// Creates a plugin result indicating an asynchronous request in progress. If provided, the hint text
    /// will be synchronously passed back to the client in the acknowledgement.
    pub fn ok_wait(hint: Option<&'static CStr>) -> Self {
        let hint_ptr = hint.map(|x| x.as_ptr()).unwrap_or_else(ptr::null);
        unsafe { Self::new(PluginResultType::JANUS_PLUGIN_OK_WAIT, hint_ptr, ptr::null_mut()) }
    }

    /// Creates a plugin result indicating an error. The provided error text will be synchronously passed
    /// back to the client.
    pub fn error(msg: &'static CStr) -> Self {
        unsafe { Self::new(PluginResultType::JANUS_PLUGIN_ERROR, msg.as_ptr(), ptr::null_mut()) }
    }

    /// Transfers ownership of this result to the wrapped raw pointer. The consumer is responsible for calling
    /// `janus_plugin_result_destroy` on the pointer when finished.
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
pub struct PluginMetadata<'pl> {
    pub version: c_int,
    pub version_str: &'pl CStr,
    pub description: &'pl CStr,
    pub name: &'pl CStr,
    pub author: &'pl CStr,
    pub package: &'pl CStr,
}

/// Helper macro to produce a Janus plugin instance. Should be called with
/// a `PluginMetadata` instance and a series of exported plugin event handlers.
#[macro_export]
macro_rules! build_plugin {
    ($md:expr, $($cb:ident),*) => {{
        extern "C" fn get_api_compatibility() -> c_int { $crate::API_VERSION }
        extern "C" fn get_version() -> c_int { $md.version }
        extern "C" fn get_version_string() -> *const c_char { $md.version_str.as_ptr() }
        extern "C" fn get_description() -> *const c_char { $md.description.as_ptr() }
        extern "C" fn get_name() -> *const c_char { $md.name.as_ptr() }
        extern "C" fn get_author() -> *const c_char { $md.author.as_ptr() }
        extern "C" fn get_package() -> *const c_char { $md.package.as_ptr() }
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
