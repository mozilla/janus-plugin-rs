extern crate chrono;
extern crate colored;
extern crate janus_plugin_sys as janus;
extern crate jansson_sys as jansson;

use chrono::Local;
use colored::{Color, Colorize};
use std::fmt;
use std::fmt::Write;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
pub use janus::JANUS_PLUGIN_API_VERSION as API_VERSION;
pub use janus::janus_callbacks as PluginCallbacks;
pub use janus::janus_plugin as Plugin;
pub use janus::janus_plugin_result as PluginResult;
pub use janus::janus_plugin_result_type as PluginResultType;
pub use janus::janus_plugin_session as PluginSession;
pub use jansson::json_t as Json;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LogLevel {
    Dbg,
    Huge,
    Verb,
    Info,
    Warn,
    Err,
    Fatal
}

impl LogLevel {
    fn color(&self) -> Option<Color> {
        match *self {
            LogLevel::Fatal => Some(Color::Magenta),
            LogLevel::Err => Some(Color::Red),
            LogLevel::Warn => Some(Color::Yellow),
            _ => None,
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = format!("[{:?}]", self).to_uppercase();
        match self.color() {
            Some(c) => name.color(c).fmt(f),
            None => f.write_str(&name),
        }
    }
}

/// Writes a message at the given log level to the Janus log.
pub fn log(level: LogLevel, message: &str) {
    let mut output = String::with_capacity(message.len() + 40);
    if unsafe { janus::janus_log_timestamps == 1 } {
        write!(output, "{} ", Local::now().format("[%a %b %e %T %Y]")).unwrap()
    }
    if level >= LogLevel::Warn {
        write!(output, "{} ", level).unwrap();
    }
    output.push_str(message);
    output.push('\n');
    unsafe { janus::janus_vprintf(CString::new(output).unwrap().as_ptr()) }
}

/// Allocates a Janus plugin result. Should be destroyed with destroy_result.
pub fn create_result(type_: PluginResultType, text: *const c_char, content: *mut Json) -> Box<PluginResult> {
    unsafe { Box::from_raw(janus::janus_plugin_result_new(type_, text, content)) }
}

/// Destroys a Janus plugin result.
pub fn destroy_result(result: Box<PluginResult>) {
    unsafe { janus::janus_plugin_result_destroy(Box::into_raw(result)) }
}

/// Represents metadata about this plugin which Janus can query at runtime.
pub struct PluginMetadata {
    pub version: c_int,
    pub version_str: *const c_char,
    pub description: *const c_char,
    pub name: *const c_char,
    pub author: *const c_char,
    pub package: *const c_char,
}

/// Helper macro to define a library as containing a Janus plugin. Should be called with
/// a PluginMetadata instance and a series of exported plugin event handlers.
#[macro_export]
macro_rules! export_plugin {
    ($md:expr, $($cb:ident),*) => {
        extern fn get_api_compatibility() -> c_int { $crate::API_VERSION }
        extern fn get_version() -> c_int { $md.version }
        extern fn get_version_string() -> *const c_char { $md.version_str }
        extern fn get_description() -> *const c_char { $md.description }
        extern fn get_name() -> *const c_char { $md.name }
        extern fn get_author() -> *const c_char { $md.author }
        extern fn get_package() -> *const c_char { $md.package }
        const PLUGIN: $crate::Plugin = $crate::Plugin {
            get_api_compatibility,
            get_version,
            get_version_string,
            get_description,
            get_name,
            get_author,
            get_package,
            $($cb,)*
        };

        /// Called by Janus to create an instance of this plugin, using the provided callbacks to dispatch events.
        #[no_mangle]
        pub extern "C" fn create() -> *const $crate::Plugin { &PLUGIN }
    }
}
