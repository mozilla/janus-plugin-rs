/// Utilities to manipulate RTCP packets. For reference, see:
///
/// <https://tools.ietf.org/html/rfc3605> (RTCP)
/// <https://tools.ietf.org/html/rfc4585> (definition of PLI, others)
/// <https://tools.ietf.org/html/rfc5104> (definition of FIR, others)
/// <https://tools.ietf.org/html/draft-alvestrand-rmcat-remb-03> (definition of REMB)

use super::ffi;

/// Returns whether this RTCP packet is a FIR packet.
pub fn has_fir(packet: &[i8]) -> bool {
    unsafe { ffi::rtcp::janus_rtcp_has_fir(packet.as_ptr() as *mut _, packet.len() as i32) == 1 }
}

/// Returns whether this RTCP packet is a PLI packet.
pub fn has_pli(packet: &[i8]) -> bool {
    unsafe { ffi::rtcp::janus_rtcp_has_pli(packet.as_ptr() as *mut _, packet.len() as i32) == 1 }
}

/// If this RTCP packet is an REMB packet, returns the bitrate it contains; else None.
pub fn get_remb(packet: &[i8]) -> Option<u32> {
    unsafe {
        match ffi::rtcp::janus_rtcp_get_remb(packet.as_ptr() as *mut _, packet.len() as i32) {
            0 => None,
            n => Some(n)
        }
    }
}

/// Increments the given sequence number, then allocates and writes a new FIR packet with the new sequence number.
pub fn gen_fir(seq: &mut i32) -> Vec<i8> {
    let mut packet = Vec::with_capacity(20);
    let result = unsafe { ffi::rtcp::janus_rtcp_fir(packet.as_mut_ptr(), 20, seq) };
    match result {
        // errors should only be the result of invalid inputs to janus_rtcp_fir
        err if err < 0 => unreachable!(format!("Error generating FIR packet (code {}) :(", err)),
        len => {
            unsafe { packet.set_len(len as usize) };
            packet
        }
    }
}

/// Allocates and writes a new PLI packet.
pub fn gen_pli() -> Vec<i8> {
    let mut packet = Vec::with_capacity(12);
    let result = unsafe { ffi::rtcp::janus_rtcp_pli(packet.as_mut_ptr(), 12) };
    match result {
        // errors should only be the result of invalid inputs to janus_rtcp_pli
        err if err < 0 => unreachable!(format!("Error generating PLI packet (code {}) :(", err)),
        len => {
            unsafe { packet.set_len(len as usize) };
            packet
        }
    }
}

/// Allocates and writes a new REMB packet with the given bitrate.
pub fn gen_remb(bitrate: u32) -> Vec<i8> {
    let mut packet = Vec::with_capacity(24);
    let result = unsafe { ffi::rtcp::janus_rtcp_remb(packet.as_mut_ptr(), 24, bitrate) };
    match result {
        // errors should only be the result of invalid inputs to janus_rtcp_remb
        err if err < 0 => unreachable!(format!("Error generating REMB packet (code {}).", err)),
        len => {
            unsafe { packet.set_len(len as usize) };
            packet
        }
    }
}
