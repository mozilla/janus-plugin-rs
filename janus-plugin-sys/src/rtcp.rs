#![allow(non_camel_case_types)]

use super::glib_sys::{gboolean, GSList};
use std::os::raw::{c_char, c_int, c_uint};

extern "C" {
    pub fn janus_rtcp_get_sender_ssrc(packet: *mut c_char, len: c_int) -> c_uint;
    pub fn janus_rtcp_get_receiver_ssrc(packet: *mut c_char, len: c_int) -> c_uint;
    pub fn janus_rtcp_filter(packet: *mut c_char, len: c_int, newlen: *mut c_int) -> *mut c_char;
    pub fn janus_rtcp_has_bye(packet: *mut c_char, len: c_int) -> gboolean;
    pub fn janus_rtcp_has_fir(packet: *mut c_char, len: c_int) -> gboolean;
    pub fn janus_rtcp_has_pli(packet: *mut c_char, len: c_int) -> gboolean;
    pub fn janus_rtcp_get_nacks(packet: *mut c_char, len: c_int) -> *mut GSList;
    pub fn janus_rtcp_remove_nacks(packet: *mut c_char, len: c_int) -> c_int;
    pub fn janus_rtcp_get_remb(packet: *mut c_char, len: c_int) -> u32;
    pub fn janus_rtcp_cap_remb(packet: *mut c_char, len: c_int, bitrate: u32) -> c_int;
    pub fn janus_rtcp_sdes(packet: *mut c_char, len: c_int, cname: *const c_char, cnamelen: c_int) -> c_int;
    pub fn janus_rtcp_remb(packet: *mut c_char, len: c_int, bitrate: u32) -> c_int;
    pub fn janus_rtcp_fir(packet: *mut c_char, len: c_int, seqnr: *mut c_int) -> c_int;
    pub fn janus_rtcp_fir_legacy(packet: *mut c_char, len: c_int, seqnr: *mut c_int) -> c_int;
    pub fn janus_rtcp_pli(packet: *mut c_char, len: c_int) -> c_int;
    pub fn janus_rtcp_nacks(packet: *mut c_char, len: c_int, nacks: *mut GSList) -> c_int;
}
