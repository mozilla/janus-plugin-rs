extern crate janus_plugin_sys as internal;

use std::os::raw::{c_char, c_int};
pub use internal::JANUS_PLUGIN_API_VERSION as API_VERSION;
pub use internal::janus_callbacks as PluginCallbacks;
pub use internal::janus_plugin as Plugin;
pub use internal::janus_plugin_result as PluginResult;
pub use internal::janus_plugin_result_type as PluginResultType;
pub use internal::janus_plugin_session as PluginSession;
pub use internal::json_t as Json;
pub use internal::janus_vprintf as log;

/// Represents metadata about this plugin which Janus can query at runtime.
pub struct PluginMetadata {
    pub version: c_int,
    pub version_str: *const c_char,
    pub description: *const c_char,
    pub name: *const c_char,
    pub author: *const c_char,
    pub package: *const c_char
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
