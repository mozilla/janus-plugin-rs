extern crate janus_plugin_sys as janus;
extern crate glib_sys as glib;

use std::ffi::{CStr, CString};
use std::ops::Deref;

pub type RawSdp = janus::sdp::janus_sdp;

pub struct Sdp {
    contents: *mut RawSdp,
}

impl Deref for Sdp {
    type Target = RawSdp;

    fn deref(&self) -> &RawSdp {
        unsafe { &*self.contents }
    }
}

impl Drop for Sdp {
    fn drop(&mut self) {
        unsafe { janus::sdp::janus_sdp_free(self.contents) }
    }
}

pub struct GLibString<'a> {
    pub contents: &'a CStr
}

impl<'a> Deref for GLibString<'a> {
    type Target = CStr;

    fn deref(&self) -> &CStr {
        return self.contents;
    }
}

impl<'a> Drop for GLibString<'a> {
    fn drop(&mut self) {
        unsafe { glib::g_free(self.contents.as_ptr() as *mut _) }
    }
}

pub fn parse_sdp(offer: &str, err_capacity: usize) -> Result<Sdp, String> {
    let offer_buffer = CString::new(offer).unwrap().as_ptr();
    let mut error_buffer = Vec::<u8>::with_capacity(err_capacity);
    let result = unsafe {
        janus::sdp::janus_sdp_parse(offer_buffer, error_buffer.as_mut_ptr() as *mut i8, error_buffer.capacity())
    };
    if result.is_null() {
        Err(CString::new(error_buffer).unwrap().to_str().unwrap().to_owned())
    } else {
        Ok(Sdp { contents: result })
    }
}

pub fn answer_sdp(sdp: &Sdp) -> Sdp {
    let result = unsafe { janus::sdp::janus_sdp_generate_answer(sdp as *const _ as *mut _, 0) };
    Sdp { contents: result }
}

pub fn write_sdp(answer: &Sdp) -> GLibString {
    unsafe { GLibString { contents: CStr::from_ptr(janus::sdp::janus_sdp_write(answer.contents)) }}
}
