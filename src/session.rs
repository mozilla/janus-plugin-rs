/// Utilities to make it easier to maintain Janus session state between plugin callbacks.
use crate::PluginSession;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

/// An error indicating that someone handed us a null plugin session handle.
#[derive(Debug, Clone, Copy)]
pub struct NullHandleError;

impl Error for NullHandleError {}

impl fmt::Display for NullHandleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("A null session handle was provided.")
    }
}

/// A wrapper for a Janus session. Contains a pointer to the Janus `PluginSession` (which is used to identify
/// this session in the Janus FFI) and any Rust state associated with the session.
#[derive(Debug, Clone)]
pub struct SessionWrapper<T> {
    pub handle: *mut PluginSession,
    state: T,
}

impl<T> Hash for SessionWrapper<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.handle.hash(state);
    }
}

impl<T> PartialEq for SessionWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl<T> Eq for SessionWrapper<T> {}

impl<T> SessionWrapper<T> {
    /// Allocates a boxed, reference-counted state wrapper associated with a Janus PluginSession
    /// (whose plugin_handle will then point to the contents of the box).
    pub unsafe fn associate(handle: *mut PluginSession, state: T) -> Result<Box<Arc<Self>>, NullHandleError> {
        match handle.as_mut() {
            Some(x) => {
                let mut result = Box::new(Arc::new(Self { handle, state }));
                x.plugin_handle = result.as_mut() as *mut Arc<Self> as *mut _;
                Ok(result)
            }
            None => Err(NullHandleError),
        }
    }

    /// Retrieves and clones the reference-counted state wrapper associated with a Janus PluginSession.
    pub unsafe fn from_ptr(handle: *mut PluginSession) -> Result<Arc<Self>, NullHandleError> {
        match handle.as_ref() {
            Some(x) => Ok(Arc::clone(
                (x.plugin_handle as *mut Arc<Self>).as_ref().unwrap()
            )),
            None => Err(NullHandleError),
        }
    }

    /// Returns the opaque pointer for this session.
    pub fn as_ptr(&self) -> *mut PluginSession {
        self.handle
    }
}

impl<T> Deref for SessionWrapper<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.state
    }
}

impl<T> Drop for SessionWrapper<T> {
    fn drop(&mut self) {
        unsafe {
            // the Janus core assumes that we will store the handle reference, and increments it on our behalf.
            // we're only responsible for decrementing it when we are done with it.
            let refcount = &(*self.handle).ref_;
            super::refcount::decrease(refcount);
        }
    }
}

// the pointer is opaque to Janus code, so this handle is threadsafe to the extent that the state is

unsafe impl<T: Sync> Sync for SessionWrapper<T> {}
unsafe impl<T: Send> Send for SessionWrapper<T> {}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::refcount::ReferenceCount;
    use std::ptr;

    #[test]
    fn handle_round_trip() {
        struct State(i32);
        extern "C" fn janus_session_free(_session_ref: *const ReferenceCount) {}
        let mut handle = PluginSession {
            gateway_handle: ptr::null_mut(),
            plugin_handle: ptr::null_mut(),
            stopped: 0,
            ref_: ReferenceCount {
                count: 1,
                free: janus_session_free,
            },
        };

        let ptr = &mut handle as *mut _;
        let session = unsafe { SessionWrapper::associate(ptr, State(42)).unwrap() };
        assert_eq!(session.as_ref() as *const _ as *mut _, handle.plugin_handle);
        assert_eq!(unsafe { SessionWrapper::<State>::from_ptr(ptr).unwrap().state.0 }, 42);
    }
}
