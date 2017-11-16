/// Utilities to write SDP offers and answers using Janus's SDP parsing machinery.

use super::ffi;
use super::glib;
use super::libc;
pub use ffi::sdp::janus_sdp_generate_answer as generate_answer;
pub use ffi::sdp::janus_sdp_generate_offer as generate_offer;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::str;
use utils::GLibString;

pub type RawSdp = ffi::sdp::janus_sdp;
pub type RawMLine = ffi::sdp::janus_sdp_mline;
pub type RawAttribute = ffi::sdp::janus_sdp_attribute;
pub use ffi::sdp::janus_sdp_mtype as MediaType;
pub use ffi::sdp::janus_sdp_mdirection as MediaDirection;

/// SDP attributes which may refer to a specific RTP payload type.
static MEDIA_PAYLOAD_ATTRIBUTES: [&'static str; 3] = ["rtpmap", "fmtp", "rtcp-fb"];

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
        self.to_cstr().to_str().unwrap()
    }
    pub fn to_cstr(&self) -> &'static CStr {
        unsafe {
            CStr::from_ptr(match *self {
                AudioCodec::Opus => cstr!("opus"),
                AudioCodec::Pcmu => cstr!("pcmu"),
                AudioCodec::Pcma => cstr!("pcma"),
                AudioCodec::G722 => cstr!("g722"),
                AudioCodec::Isac16 => cstr!("isac16"),
                AudioCodec::Isac32 => cstr!("isac32"),
            })
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
        self.to_cstr().to_str().unwrap()
    }
    pub fn to_cstr(&self) -> &'static CStr {
        unsafe {
            CStr::from_ptr(match *self {
                VideoCodec::Vp8 => cstr!("vp8"),
                VideoCodec::Vp9 => cstr!("vp9"),
                VideoCodec::H264 => cstr!("h264"),
            })
        }
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// Parameters controlling SDP offer answering behavior. Used as keys in the parameter list
/// for `janus_sdp_generate_answer`. See sdp-utils.h in the Janus source for more details.
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
    pub fn parse(offer: &CStr) -> Result<Self, Box<Error>> {
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

    /// Gets the payload type number for a codec in this SDP, or None if the codec isn't present.
    pub fn get_payload_type(&self, codec_name: &CStr) -> Option<i32> {
        unsafe {
            match ffi::sdp::janus_sdp_get_codec_pt(self.ptr, codec_name.as_ptr()) {
                err if err < 0 => None,
                n => Some(n),
            }
        }
    }

    /// Rewrites any references from one dynamically assigned payload type in this SDP to another dynamically assigned
    /// payload type.
    pub fn rewrite_payload_type(&mut self, from: i32, to: i32) {
        let from_pt_string = from.to_string();
        let to_pt_string = to.to_string();
        for (_media, m_lines) in self.get_mlines() {
            unsafe {
                for m_line in m_lines {
                    // 1. replace the payload type ID in this media line's payload type list
                    if !glib::g_list_find(m_line.ptypes, from as *const _).is_null() {
                        // payload type data in the list is cast to pointers
                        m_line.ptypes = glib::g_list_remove(m_line.ptypes, from as *const _);
                        m_line.ptypes = glib::g_list_prepend(m_line.ptypes, to as *mut _);
                    }
                    // 2. rewrite the values of attribute lines with the old payload type to have the new payload type
                    let mut attr_node = m_line.attributes;
                    while let Some(node) = attr_node.as_ref() {
                        let next = node.next; // we might delete this link, so grab next now!
                        let data = node.data as *mut RawAttribute;
                        let attr = data.as_ref().expect("Null data in SDP attribute node :(");
                        let name = CStr::from_ptr(attr.name).to_str().expect("Invalid attribute name in SDP :(");
                        if MEDIA_PAYLOAD_ATTRIBUTES.contains(&name) {
                            // each of the attributes with payload types in the values look like "$pt $stuff"
                            // where $stuff is specifying payload-type-specfic options; just rewrite $pt
                            let value = CStr::from_ptr(attr.value).to_str().expect("Invalid attribute value in SDP :(");
                            if value.starts_with(&from_pt_string) {
                                let new_val = if name == "fmtp" {
                                    CString::new("107 profile-level-id=42e01f;packetization-mode=1").unwrap()
                                } else {
                                    CString::new(value.replacen(&from_pt_string, &to_pt_string, 1)).unwrap()
                                };
                                // value string is copied into the attribute
                                let new_attr = ffi::sdp::janus_sdp_attribute_create(attr.name, new_val.as_ptr());
                                m_line.attributes = glib::g_list_prepend(m_line.attributes, new_attr as *mut _);
                                m_line.attributes = glib::g_list_delete_link(m_line.attributes, attr_node);
                                ffi::sdp::janus_sdp_attribute_destroy(data);
                            }
                        }
                        attr_node = next;
                    }
                }
            }
        }
    }

    /// Returns a map of all the SDP media lines per SDP media type.
    pub fn get_mlines(&self) -> HashMap<MediaType, Vec<&mut RawMLine>> {
        let mut result = HashMap::new();
        unsafe {
            let mut ml_node = (*self.ptr).m_lines;
            while let Some(node) = ml_node.as_ref() {
                let ml = (node.data as *mut RawMLine).as_mut().expect("Null data in SDP media node :(");
                result.entry(ml.type_).or_insert_with(Vec::new).push(ml);
                ml_node = node.next;
            }
            result
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
/// (This has to be a macro because `generate_answer` is variadic.)
macro_rules! answer_sdp {
    ($sdp:expr $(, $param:expr, $value:expr)* $(,)*) => {{
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

#[macro_export]
/// Generates an SDP offer given some parameters.
/// (This has to be a macro because `generate_offer` is variadic.)
macro_rules! offer_sdp {
    ($name:expr, $address:expr $(, $param:expr, $value:expr)* $(,)*) => {{
        unsafe {
            let result = $crate::sdp::generate_offer(
                $name,
                $address,
                $($param, $value,)*
                $crate::sdp::OfferAnswerParameters::Done
            );
            $crate::sdp::Sdp::new(result).expect("Mysterious error generating SDP offer :(")
        }
    }}
}
