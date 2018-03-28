/// Helper macro to produce a Janus plugin instance. Should be called with
/// a `LibraryMetadata` instance and a series of exported plugin event handlers.
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
