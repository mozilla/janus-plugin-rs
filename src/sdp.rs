/// Utilities to write SDP offers and answers using Janus's SDP parsing machinery.

use super::ffi;
use super::libc;
pub use ffi::sdp::janus_sdp_generate_answer as generate_answer;
use std::error::Error;
use std::ffi::CString;
use std::ops::Deref;
use std::str;
use utils::GLibString;

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
    pub ptr: *mut RawSdp, // annoyingly pub because of answer_sdp macro
}

impl Sdp {
    pub unsafe fn new(ptr: *mut RawSdp) -> Option<Self> {
        ptr.as_mut().map(|p| Self { ptr: p })
    }

    /// Parses an SDP offer string from a client into a structured SDP object.
    pub fn parse(offer: CString) -> Result<Self, Box<Error>> {
        let mut error_buffer = Vec::with_capacity(512);
        let error_ptr = error_buffer.as_mut_ptr() as *mut _;
        unsafe {
            let result = ffi::sdp::janus_sdp_parse(offer.as_ptr(), error_ptr, error_buffer.capacity());
            Sdp::new(result).ok_or_else(|| {
                error_buffer.set_len(libc::strlen(error_ptr));
                From::from(str::from_utf8(&error_buffer).expect("SDP error not valid UTF-8 :("))
            })
        }
    }

    /// Writes this SDP into a string.
    pub fn to_string(&self) -> GLibString {
        unsafe {
            let sdp = ffi::sdp::janus_sdp_write(self.ptr);
            GLibString::from_chars(sdp).expect("Mysterious error writing SDP to string :(")
        }
    }
}

impl Deref for Sdp {
    type Target = RawSdp;

    fn deref(&self) -> &RawSdp {
        unsafe { &*self.ptr }
    }
}

impl Drop for Sdp {
    fn drop(&mut self) {
        unsafe { ffi::sdp::janus_sdp_free(self.ptr) }
    }
}

unsafe impl Send for Sdp {}

#[macro_export]
/// Given an SDP offer from a client, generates an SDP answer.
/// (This has to be a macro because generate_answer is variadic.)
macro_rules! answer_sdp {
    ($sdp:expr $(, $param:expr, $value:expr),*) => {{
        unsafe {
            let result = $crate::sdp::generate_answer(
                $sdp.ptr,
                $($param, $value,)*
                $crate::sdp::OfferAnswerParameters::Done
            );
            $crate::sdp::Sdp::new(result).expect("Mysterious error generating SDP answer :(")
        }
    }}
}
