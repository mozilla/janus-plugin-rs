#![allow(non_camel_case_types)]

use glib_sys::{gboolean, GList};
use std::os::raw::{c_char, c_int, c_long, c_short, c_ulong};

pub type guint16 = c_short;
pub type guint64 = c_ulong;
pub type gint64 = c_long;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum janus_sdp_mtype {
    JANUS_SDP_AUDIO = 0,
    JANUS_SDP_VIDEO = 1,
    JANUS_SDP_APPLICATION = 2,
    JANUS_SDP_OTHER = 3,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum janus_sdp_mdirection {
    JANUS_SDP_DEFAULT = 0,
    JANUS_SDP_SENDRECV = 1,
    JANUS_SDP_SENDONLY = 2,
    JANUS_SDP_RECVONLY = 3,
    JANUS_SDP_INACTIVE = 4,
    JANUS_SDP_INVALID = 5,
}

#[repr(C)]
#[derive(Debug)]
pub struct janus_sdp {
    pub version: c_int,
    pub o_name: *mut c_char,
    pub o_sessid: guint64,
    pub o_version: guint64,
    pub o_ipv4: gboolean,
    pub o_addr: *mut c_char,
    pub s_name: *mut c_char,
    pub t_start: guint64,
    pub t_stop: guint64,
    pub c_ipv4: gboolean,
    pub c_addr: *mut c_char,
    pub attributes: *mut GList,
    pub m_lines: *mut GList,
}

#[repr(C)]
#[derive(Debug)]
pub struct janus_sdp_mline {
    pub type_: janus_sdp_mtype,
    pub type_str: *mut c_char,
    pub port: guint16,
    pub proto: *mut c_char,
    pub fmts: *mut GList,
    pub ptypes: *mut GList,
    pub c_ipv4: gboolean,
    pub c_addr: *mut c_char,
    pub b_name: *mut c_char,
    pub b_value: c_int,
    pub direction: janus_sdp_mdirection,
    pub attributes: *mut GList,
}

#[repr(C)]
#[derive(Debug)]
pub struct janus_sdp_attribute {
    pub name: *mut c_char,
    pub value: *mut c_char,
    pub direction: janus_sdp_mdirection,
}

extern "C" {
    pub fn janus_sdp_parse_mtype(type_: *const c_char) -> janus_sdp_mtype;
    pub fn janus_sdp_mtype_str(type_: janus_sdp_mtype) -> *const c_char;
    pub fn janus_sdp_parse_mdirection(direction: *const c_char) -> janus_sdp_mdirection;
    pub fn janus_sdp_mdirection_str(direction: janus_sdp_mdirection) -> *const c_char;
    pub fn janus_sdp_mline_create(
        type_: janus_sdp_mtype,
        port: guint16,
        proto: *const c_char,
        direction: janus_sdp_mdirection,
    ) -> *mut janus_sdp_mline;
    pub fn janus_sdp_mline_destroy(mline: *mut janus_sdp_mline);
    pub fn janus_sdp_mline_find(sdp: *mut janus_sdp, type_: janus_sdp_mtype) -> *mut janus_sdp_mline;
    pub fn janus_sdp_attribute_create(name: *const c_char, value: *const c_char, ...) -> *mut janus_sdp_attribute;
    pub fn janus_sdp_attribute_destroy(attr: *mut janus_sdp_attribute);
    pub fn janus_sdp_attribute_add_to_mline(mline: *mut janus_sdp_mline, attr: *mut janus_sdp_attribute) -> c_int;
    pub fn janus_sdp_parse(sdp: *const c_char, error: *mut c_char, errlen: usize) -> *mut janus_sdp;
    pub fn janus_sdp_remove_payload_type(sdp: *mut janus_sdp, pt: c_int) -> c_int;
    pub fn janus_sdp_write(sdp: *mut janus_sdp) -> *mut c_char;
    pub fn janus_sdp_new(name: *const c_char, address: *const c_char) -> *mut janus_sdp;
    pub fn janus_sdp_generate_offer(name: *const c_char, address: *const c_char, ...) -> *mut janus_sdp;
    pub fn janus_sdp_generate_answer(offer: *mut janus_sdp, ...) -> *mut janus_sdp;
    pub fn janus_sdp_get_codec_pt(sdp: *mut janus_sdp, codec: *const c_char) -> c_int;
    pub fn janus_sdp_get_codec_pt_full(sdp: *mut janus_sdp, codec: *const c_char, profile: *const c_char) -> c_int;
    pub fn janus_sdp_get_codec_name(sdp: *mut janus_sdp, pt: c_int) -> *const c_char;
    pub fn janus_sdp_get_codec_rtpmap(codec: *const c_char) -> *const c_char;
    pub fn janus_sdp_destroy(sdp: *mut janus_sdp);
}
