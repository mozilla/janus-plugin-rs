#![deny(missing_debug_implementations)]

use janus_plugin_sys as ffi;
use bitflags::bitflags;
pub use debug::LogLevel;
pub use debug::log;
pub use crate::jansson::{JanssonDecodingFlags, JanssonEncodingFlags, JanssonValue, RawJanssonValue};
pub use session::SessionWrapper;
pub use ffi::events::janus_eventhandler as EventHandler;
pub use ffi::plugin::janus_callbacks as PluginCallbacks;
pub use ffi::plugin::janus_plugin as Plugin;
pub use ffi::plugin::janus_plugin_result as RawPluginResult;
pub use ffi::plugin::janus_plugin_session as PluginSession;
use ffi::plugin::janus_plugin_result_type as PluginResultType;
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
pub mod refcount;

bitflags! {
    /// Flags that control which events an event handler receives.
    pub struct JanusEventType: u32 {
        const JANUS_EVENT_TYPE_SESSION   = 1 << 0;
        const JANUS_EVENT_TYPE_HANDLE    = 1 << 1;
        const JANUS_EVENT_TYPE_JSEP      = 1 << 3; // yes, really
        const JANUS_EVENT_TYPE_WEBRTC    = 1 << 4;
        const JANUS_EVENT_TYPE_MEDIA     = 1 << 5;
        const JANUS_EVENT_TYPE_PLUGIN    = 1 << 6;
        const JANUS_EVENT_TYPE_TRANSPORT = 1 << 7;
        const JANUS_EVENT_TYPE_CORE      = 1 << 8;
    }
}

/// An error emitted by the Janus core in response to a plugin pushing an event.
#[derive(Debug, Clone, Copy)]
pub struct JanusError {
    pub code: i32
}

/// A result from pushing an event to Janus core.
pub type JanusResult = Result<(), JanusError>;

impl JanusError {
    /// Returns Janus's description text for this error.
    pub fn to_cstr(self) -> &'static CStr {
        unsafe { CStr::from_ptr(ffi::janus_get_api_error(self.code)) }
    }
    /// Converts a Janus result code to either success or a potential error.
    pub fn from(val: i32) -> JanusResult {
        match val {
            0 => Ok(()),
            e => Err(JanusError { code: e })
        }
    }
}

impl Error for JanusError {
    fn description(&self) -> &'static str {
        self.to_cstr().to_str().unwrap()
    }
}

impl fmt::Display for JanusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (code: {})", self.description(), self.code)
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
        Self { ptr: ffi::plugin::janus_plugin_result_new(type_, text, content) }
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
        unsafe { ffi::plugin::janus_plugin_result_destroy(self.ptr) }
    }
}

unsafe impl Send for PluginResult {}

#[derive(Debug)]
/// Represents metadata about this library which Janus can query at runtime.
pub struct LibraryMetadata<'a> {
    pub api_version: c_int,
    pub version: c_int,
    pub version_str: &'a CStr,
    pub description: &'a CStr,
    pub name: &'a CStr,
    pub author: &'a CStr,
    pub package: &'a CStr,
}

/// Helper macro to produce a Janus plugin instance. Should be called with
/// a `LibraryMetadata` instance and a series of exported plugin callbacks.
#[macro_export]
macro_rules! build_plugin {
    ($md:expr, $($cb:ident),*) => {{
        extern "C" fn get_api_compatibility() -> c_int { $md.api_version }
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

/// Helper macro to produce a Janus event handler instance. Should be called with
/// a `LibraryMetadata` instance and a series of exported event handler callbacks.
#[macro_export]
macro_rules! build_eventhandler {
    ($md:expr, $mask:expr, $($cb:ident),*) => {{
        extern "C" fn get_api_compatibility() -> c_int { $md.api_version }
        extern "C" fn get_version() -> c_int { $md.version }
        extern "C" fn get_version_string() -> *const c_char { $md.version_str.as_ptr() }
        extern "C" fn get_description() -> *const c_char { $md.description.as_ptr() }
        extern "C" fn get_name() -> *const c_char { $md.name.as_ptr() }
        extern "C" fn get_author() -> *const c_char { $md.author.as_ptr() }
        extern "C" fn get_package() -> *const c_char { $md.package.as_ptr() }
        $crate::EventHandler {
            events_mask: $mask,
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

/// Macro to export a Janus event handler instance from this module.
#[macro_export]
macro_rules! export_eventhandler {
    ($evh:expr) => {
        /// Called by Janus to create an instance of this event handler, using the provided callbacks to dispatch events.
        #[no_mangle]
        pub extern "C" fn create() -> *const $crate::EventHandler { $evh }
    }
}
