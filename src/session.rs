/// Utilities to make it easier to maintain Janus session state between plugin callbacks.

use std::error::Error;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use super::PluginSession;

/// An error indicating that someone handed us a null plugin session handle.
#[derive(Debug)]
pub struct NullHandleError;

impl Error for NullHandleError {
    fn description(&self) -> &str {
        "A null session handle was provided."
    }
}

impl fmt::Display for NullHandleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

/// A wrapper for a Janus session. Contains a pointer to the Janus PluginSession (which is used to identify
/// this session in the Janus FFI) and any Rust state associated with the session.
#[derive(Debug)]
pub struct SessionWrapper<T> {
    pub handle: *mut PluginSession,
    state: T,
}

impl<T> SessionWrapper<T> {

    /// Allocates a boxed, reference-counted state wrapper associated with a Janus PluginSession
    /// (whose plugin_handle will then point to the contents of the box).
    pub fn associate(handle: *mut PluginSession, state: T) -> Result<Box<Arc<Self>>, NullHandleError> {
        unsafe {
            match handle.as_mut() {
                Some(x) => {
                    let mut result = Box::new(Arc::new(Self { handle, state }));
                    x.plugin_handle = result.as_mut() as *mut Arc<Self> as *mut _;
                    Ok(result)
                },
                None => Err(NullHandleError)
            }
        }
    }

    /// Retrieves and clones the reference-counted state wrapper associated with a Janus PluginSession.
    pub fn from_ptr<'a>(handle: *mut PluginSession) -> Result<Arc<Self>, NullHandleError> {
        unsafe {
            match handle.as_ref() {
                Some(x) => Ok(Arc::clone((x.plugin_handle as *mut Arc<Self>).as_ref().unwrap())),
                None => Err(NullHandleError)
            }
        }
    }
}

impl<T> Deref for SessionWrapper<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.state
    }
}

// the pointer is opaque to Janus code, so this handle is threadsafe to the extent that the state is

unsafe impl<T: Sync> Sync for SessionWrapper<T> {}
unsafe impl<T: Send> Send for SessionWrapper<T> {}

#[cfg(test)]
mod tests {

    use super::*;
    use std::ptr;

    #[test]
    fn handle_round_trip() {
        struct State(i32);
        let mut handle = PluginSession {
            gateway_handle: ptr::null_mut(),
            plugin_handle: ptr::null_mut(),
            stopped_bitfield: 0,
            __padding: Default::default()
        };
        let ptr = &mut handle as *mut _;
        let session = SessionWrapper::associate(ptr, State(42)).unwrap();
        assert_eq!(session.as_ref() as *const _ as *mut _, handle.plugin_handle);
        assert_eq!(SessionWrapper::<State>::from_ptr(ptr).unwrap().state.0, 42);
    }
}
