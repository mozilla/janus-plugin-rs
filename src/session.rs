/// Utilities to make it easier to maintain Janus session state between plugin callbacks.

use std::ops::Deref;
use std::sync::Arc;
use super::PluginHandle;

/// A representation of session state associated with a Janus plugin session handle.
#[derive(Debug)]
pub struct SessionHandle<T> {
    pub handle: *mut PluginHandle,
    state: T,
}

impl<T> SessionHandle<T> {

    /// Allocates a boxed, reference-counted SessionHandle associated with an opaque Janus handle
    /// (whose plugin_handle will then point to the contents of the box).
    pub fn establish(handle: *mut PluginHandle, state: T) -> Box<Arc<Self>> {
        let result = Box::new(Arc::new(Self { handle, state: state }));
        unsafe { (*handle).plugin_handle = result.as_ref() as *const _ as *mut _ };
        result
    }

    /// Retrieves the reference-counted SessionHandle pointed to by an opaque Janus handle.
    pub fn from_ptr<'a>(handle: *mut PluginHandle) -> &'a Arc<Self> {
        unsafe { &*((*handle).plugin_handle as *mut Arc<Self>) }
    }
}

impl<T> Deref for SessionHandle<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.state
    }
}

// the pointer is opaque to Janus code, so this handle is threadsafe to the extent that the state is

unsafe impl<T: Sync> Sync for SessionHandle<T> {}
unsafe impl<T: Send> Send for SessionHandle<T> {}

#[cfg(test)]
mod tests {

    use super::*;
    use std::ptr;

    #[test]
    fn handle_round_trip() {
        struct State(i32);
        let mut handle = PluginHandle {
            gateway_handle: ptr::null_mut(),
            plugin_handle: ptr::null_mut(),
            stopped_bitfield: 0,
            __padding: Default::default()
        };
        let ptr = &mut handle as *mut _;
        let session = SessionHandle::establish(ptr, State(42));
        assert_eq!(session.as_ref() as *const _ as *mut _, handle.plugin_handle);
        assert_eq!(SessionHandle::<State>::from_ptr(ptr).state.0, 42);
    }
}
