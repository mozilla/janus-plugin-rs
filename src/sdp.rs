extern crate glib_sys as glib;
extern crate libc;

use super::ffi;
pub use ffi::sdp::janus_sdp_generate_answer as generate_answer;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::fmt;
use std::ops::Deref;
use std::os::raw::c_char;

pub type RawSdp = ffi::sdp::janus_sdp;
pub type MediaType = ffi::sdp::janus_sdp_mtype;
pub type MediaDirection = ffi::sdp::janus_sdp_mdirection;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Available Janus audio codecs. See utils.c.
pub enum AudioCodec {
    Opus,
    Pcmu,
    Pcma,
    G722,
    Isac16,
    Isac32,
}

impl AudioCodec {
    pub fn to_str(&self) -> &'static str {
        match *self {
            AudioCodec::Opus => "opus",
            AudioCodec::Pcmu => "pcmu",
            AudioCodec::Pcma => "pcma",
            AudioCodec::G722 => "g722",
            AudioCodec::Isac16 => "isac16",
            AudioCodec::Isac32 => "isac32",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Available Janus video codecs. See utils.c.
pub enum VideoCodec {
    Vp8,
    Vp9,
    H264,
}

impl VideoCodec {
    pub fn to_str(&self) -> &'static str {
        match *self {
            VideoCodec::Vp8 => "vp8",
            VideoCodec::Vp9 => "vp9",
            VideoCodec::H264 => "h264",
        }
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Parameters controlling SDP offer answering behavior. Used as keys in the parameter list
/// for janus_sdp_generate_answer. See sdp-utils.h in the Janus source for more details.
pub enum OfferAnswerParameters {
    /// Used to signal the end of the offer-answer parameter list.
    Done = 0,
    /// Whether to accept or reject audio.
    Audio = 1,
    /// Whether to accept or reject video.
    Video = 2,
    /// Whether to accept or reject data.
    Data = 3,
    /// The MediaDirection for the audio stream.
    AudioDirection = 4,
    /// The MediaDirection for the video stream.
    VideoDirection = 5,
    /// The AudioCodec for the audio stream.
    AudioCodec = 6,
    /// The VideoCodec for the video stream.
    VideoCodec = 7,
    /// The payload type for the audio stream.
    AudioPayloadType = 8,
    /// The payload type for the video stream.
    VideoPayloadType = 9,
    /// Whether to negotiate telephone events.
    AudioDtmf = 10,
    /// Whether to add RTCP-FB attributes.
    VideoRtcpfbDefaults = 11,
    /// Whether to add attributes for H.264 video.
    VideoH264Fmtp = 12,
}

#[derive(Debug)]
/// An SDP session description.
pub struct Sdp {
    pub contents: *mut RawSdp,
}

impl Sdp {
    pub fn new(ptr: *mut RawSdp) -> Sdp {
        Sdp { contents: ptr }
    }
}

impl Deref for Sdp {
    type Target = RawSdp;

    fn deref(&self) -> &RawSdp {
        unsafe { &*self.contents }
    }
}

impl Drop for Sdp {
    fn drop(&mut self) {
        unsafe { ffi::sdp::janus_sdp_free(self.contents) }
    }
}

#[derive(Debug)]
/// A C-style string which was allocated using glibc.
pub struct GLibString<'a> {
    pub contents: &'a CStr,
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
/// An error emitted by Janus when attempting to parse a client-supplied SDP.
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

#[macro_export]
/// Given an SDP offer from a client, generates an SDP answer.
/// (This has to be a macro because generate_answer is variadic.)
macro_rules! answer_sdp {
    ($sdp:expr $(, $param:expr, $value:expr),*) => {{
        let result = unsafe {
            $crate::sdp::generate_answer(
                $sdp.contents,
                $($param, $value,)*
                $crate::sdp::OfferAnswerParameters::Done
            )
        };
        $crate::sdp::Sdp::new(result)
    }}
}

/// Parses an SDP offer string from a client into a structured SDP object.
pub fn parse_sdp(offer: CString) -> Result<Sdp, Box<Error>> {
    let mut error_buffer = Vec::<u8>::with_capacity(512);
    let error_ptr = error_buffer.as_mut_ptr() as *mut c_char;
    let result = unsafe { ffi::sdp::janus_sdp_parse(offer.as_ptr(), error_ptr, error_buffer.capacity()) };
    if result.is_null() {
        unsafe { error_buffer.set_len(libc::strlen(error_ptr)) }
        Err(Box::new(SdpParsingError {
            details: String::from_utf8(error_buffer)?,
        }))
    } else {
        Ok(Sdp { contents: result })
    }
}

/// Writes a structured SDP object into a string.
pub fn write_sdp(answer: &Sdp) -> GLibString {
    unsafe {
        GLibString {
            contents: CStr::from_ptr(ffi::sdp::janus_sdp_write(answer.contents)),
        }
    }
}
