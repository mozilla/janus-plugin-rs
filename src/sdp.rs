extern crate libc;
extern crate janus_plugin_sys as janus;
extern crate glib_sys as glib;

use std::error::Error;
use std::fmt;
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::os::raw::c_char;

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

#[derive(Debug)]
pub struct SdpParsingError {
    pub details: String,
}

impl fmt::Display for SdpParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for SdpParsingError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub fn parse_sdp(offer: CString, err_capacity: usize) -> Result<Sdp, Box<Error>> {
    let mut error_buffer = Vec::<u8>::with_capacity(err_capacity);
    let error_ptr = error_buffer.as_mut_ptr() as *mut c_char;
    let result = unsafe { janus::sdp::janus_sdp_parse(offer.as_ptr(), error_ptr, error_buffer.capacity()) };
    if result.is_null() {
        unsafe { error_buffer.set_len(libc::strlen(error_ptr)) }
        Err(Box::new(SdpParsingError { details: CString::new(error_buffer)?.into_string()? }))
    } else {
        Ok(Sdp { contents: result })
    }
}

pub fn answer_sdp(sdp: &Sdp) -> Sdp {
    let result = unsafe { janus::sdp::janus_sdp_generate_answer(sdp.contents, 0) };
    Sdp { contents: result }
}

pub fn write_sdp(answer: &Sdp) -> GLibString {
    unsafe { GLibString { contents: CStr::from_ptr(janus::sdp::janus_sdp_write(answer.contents)) }}
}
