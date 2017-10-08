#![allow(non_camel_case_types)]
// See https://jansson.readthedocs.io/ for API documentation.

use std::os::raw::{c_char, c_int, c_longlong, c_void};

/// The type of a JSON value.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum json_type {
    JSON_OBJECT = 0,
    JSON_ARRAY = 1,
    JSON_STRING = 2,
    JSON_INTEGER = 3,
    JSON_REAL = 4,
    JSON_TRUE = 5,
    JSON_FALSE = 6,
    JSON_NULL = 7,
}

/// Flags that can be passed to JSON decoding functions.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum json_decoding_flags {
    JSON_REJECT_DUPLICATES = 1,
    JSON_DISABLE_EOF_CHECK = 2,
    JSON_DECODE_ANY = 4,
    JSON_DECODE_INT_AS_REAL = 8,
    JSON_ALLOW_NUL = 16
}

/// Flags that can be passed to JSON encoding functions.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum json_encoding_flags {
    JSON_COMPACT = 32,
    JSON_ENSURE_ASCII = 64,
    JSON_SORT_KEYS = 128,
    JSON_PRESERVE_ORDER = 256,
    JSON_ENCODE_ANY = 512,
    JSON_ESCAPE_SLASH = 1024,
    JSON_EMBED = 65536
}

/// The maximum possible indentation when pretty-printing JSON.
pub const JSON_MAX_INDENT: u32 = 31;

/// A JSON value.
#[repr(C)]
#[derive(Debug)]
pub struct json_t {
    pub type_: json_type,
    pub refcount: usize,
}

/// An error that occurred during JSON processing.
#[repr(C)]
pub struct json_error_t {
    pub line: c_int,
    pub column: c_int,
    pub position: c_int,
    pub source: [c_char; 80usize],
    pub text: [c_char; 160usize],
}

pub type json_load_callback_t = unsafe extern "C" fn(buffer: *mut c_void, buflen: usize, data: *mut c_void) -> usize;
pub type json_dump_callback_t = unsafe extern "C" fn(buffer: *const c_char, size: usize, data: *mut c_void) -> c_int;
pub type json_malloc_t = unsafe extern "C" fn(arg1: usize) -> *mut c_void;
pub type json_free_t = unsafe extern "C" fn(arg1: *mut c_void);
pub type json_int_t = c_longlong;

extern "C" {
    pub fn json_object() -> *mut json_t;
    pub fn json_array() -> *mut json_t;
    pub fn json_string(value: *const c_char) -> *mut json_t;
    pub fn json_stringn(value: *const c_char, len: usize) -> *mut json_t;
    pub fn json_string_nocheck(value: *const c_char) -> *mut json_t;
    pub fn json_stringn_nocheck(value: *const c_char, len: usize) -> *mut json_t;
    pub fn json_integer(value: json_int_t) -> *mut json_t;
    pub fn json_real(value: f64) -> *mut json_t;
    pub fn json_true() -> *mut json_t;
    pub fn json_false() -> *mut json_t;
    pub fn json_null() -> *mut json_t;
    pub fn json_delete(json: *mut json_t);
    pub fn json_object_seed(seed: usize);
    pub fn json_object_size(object: *const json_t) -> usize;
    pub fn json_object_get(object: *const json_t, key: *const c_char) -> *mut json_t;
    pub fn json_object_set_new(object: *mut json_t, key: *const c_char, value: *mut json_t) -> c_int;
    pub fn json_object_set_new_nocheck(object: *mut json_t, key: *const c_char, value: *mut json_t) -> c_int;
    pub fn json_object_del(object: *mut json_t, key: *const c_char) -> c_int;
    pub fn json_object_clear(object: *mut json_t) -> c_int;
    pub fn json_object_update(object: *mut json_t, other: *mut json_t) -> c_int;
    pub fn json_object_update_existing(object: *mut json_t, other: *mut json_t) -> c_int;
    pub fn json_object_update_missing(object: *mut json_t, other: *mut json_t) -> c_int;
    pub fn json_object_iter(object: *mut json_t) -> *mut c_void;
    pub fn json_object_iter_at(object: *mut json_t, key: *const c_char) -> *mut c_void;
    pub fn json_object_key_to_iter(key: *const c_char) -> *mut c_void;
    pub fn json_object_iter_next(object: *mut json_t, iter: *mut c_void) -> *mut c_void;
    pub fn json_object_iter_key(iter: *mut c_void) -> *const c_char;
    pub fn json_object_iter_value(iter: *mut c_void) -> *mut json_t;
    pub fn json_object_iter_set_new(object: *mut json_t, iter: *mut c_void, value: *mut json_t) -> c_int;
    pub fn json_array_size(array: *const json_t) -> usize;
    pub fn json_array_get(array: *const json_t, index: usize) -> *mut json_t;
    pub fn json_array_set_new(array: *mut json_t, index: usize, value: *mut json_t) -> c_int;
    pub fn json_array_append_new(array: *mut json_t, value: *mut json_t) -> c_int;
    pub fn json_array_insert_new(array: *mut json_t, index: usize, value: *mut json_t) -> c_int;
    pub fn json_array_remove(array: *mut json_t, index: usize) -> c_int;
    pub fn json_array_clear(array: *mut json_t) -> c_int;
    pub fn json_array_extend(array: *mut json_t, other: *mut json_t) -> c_int;
    pub fn json_string_value(string: *const json_t) -> *const c_char;
    pub fn json_string_length(string: *const json_t) -> usize;
    pub fn json_integer_value(integer: *const json_t) -> json_int_t;
    pub fn json_real_value(real: *const json_t) -> f64;
    pub fn json_number_value(json: *const json_t) -> f64;
    pub fn json_string_set(string: *mut json_t, value: *const c_char) -> c_int;
    pub fn json_string_setn(string: *mut json_t, value: *const c_char, len: usize) -> c_int;
    pub fn json_string_set_nocheck(string: *mut json_t, value: *const c_char) -> c_int;
    pub fn json_string_setn_nocheck(string: *mut json_t, value: *const c_char, len: usize) -> c_int;
    pub fn json_integer_set(integer: *mut json_t, value: json_int_t) -> c_int;
    pub fn json_real_set(real: *mut json_t, value: f64) -> c_int;
    pub fn json_pack(fmt: *const c_char, ...) -> *mut json_t;
    pub fn json_pack_ex(error: *mut json_error_t, flags: usize, fmt: *const c_char, ...) -> *mut json_t;
    pub fn json_unpack(root: *mut json_t, fmt: *const c_char, ...) -> c_int;
    pub fn json_unpack_ex(root: *mut json_t, error: *mut json_error_t, flags: usize, fmt: *const c_char, ...) -> c_int;
    pub fn json_equal(value1: *mut json_t, value2: *mut json_t) -> c_int;
    pub fn json_copy(value: *mut json_t) -> *mut json_t;
    pub fn json_deep_copy(value: *const json_t) -> *mut json_t;
    pub fn json_loads(input: *const c_char, flags: usize, error: *mut json_error_t) -> *mut json_t;
    pub fn json_loadb(buffer: *const c_char, buflen: usize, flags: usize, error: *mut json_error_t) -> *mut json_t;
    pub fn json_loadfd(input: c_int, flags: usize, error: *mut json_error_t) -> *mut json_t;
    pub fn json_load_file(path: *const c_char, flags: usize, error: *mut json_error_t) -> *mut json_t;
    pub fn json_load_callback(callback: json_load_callback_t, data: *mut c_void, flags: usize, error: *mut json_error_t) -> *mut json_t;
    pub fn json_dumps(json: *const json_t, flags: usize) -> *mut c_char;
    pub fn json_dumpb(json: *const json_t, buffer: *mut c_char, size: usize, flags: usize) -> usize;
    pub fn json_dumpfd(json: *const json_t, output: c_int, flags: usize) -> c_int;
    pub fn json_dump_file(json: *const json_t, path: *const c_char, flags: usize) -> c_int;
    pub fn json_dump_callback(json: *const json_t, callback: json_dump_callback_t, data: *mut c_void, flags: usize) -> c_int;
    pub fn json_set_alloc_funcs(malloc_fn: json_malloc_t, free_fn: json_free_t);
    pub fn json_get_alloc_funcs(malloc_fn: *mut json_malloc_t, free_fn: *mut json_free_t);
}

#[cfg(test)]
#[macro_use]
extern crate cstr_macro;

#[cfg(test)]
mod tests {

    use super::*;
    use std::ffi::CStr;
    use std::ptr;

    #[test]
    fn object_encoding() {
        unsafe {
            let x = json_object();
            json_object_set_new(x, cstr!("a"), json_string(cstr!("alpha")));
            json_object_set_new(x, cstr!("b"), json_true());
            json_object_set_new(x, cstr!("c"), json_false());
            json_object_set_new(x, cstr!("d"), json_integer(42));
            json_object_set_new(x, cstr!("e"), json_real(1.25));
            json_object_set_new(x, cstr!("f"), json_null());
            let ys = json_array();
            json_array_append_new(ys, json_integer(1));
            json_array_append_new(ys, json_integer(3));
            json_array_insert_new(ys, 1, json_integer(2));
            json_object_set_new(x, cstr!("g"), ys);
            let json = r#"{"a": "alpha", "b": true, "c": false, "d": 42, "e": 1.25, "f": null, "g": [1, 2, 3]}"#;
            assert_eq!(json, CStr::from_ptr(json_dumps(x, 0)).to_str().unwrap());
        }
    }

    #[test]
    fn object_decoding() {
        unsafe {
            let json = cstr!(r#"{"a": {"aa": [true, false], "ab": null}, "b": {}, "c": "charlie", "d": 8.75}"#);
            let root = json_loads(json, 0, ptr::null_mut());
            assert!((*root).type_ == json_type::JSON_OBJECT);
            let a = json_object_get(root, cstr!("a"));
            assert!((*a).type_ == json_type::JSON_OBJECT);
            let aa = json_object_get(a, cstr!("aa"));
            assert!((*aa).type_ == json_type::JSON_ARRAY);
            assert_eq!(json_array_size(aa), 2);
            assert!((*json_array_get(aa, 0)).type_ == json_type::JSON_TRUE);
            assert!((*json_array_get(aa, 1)).type_ == json_type::JSON_FALSE);
            let ab = json_object_get(a, cstr!("ab"));
            assert!((*ab).type_ == json_type::JSON_NULL);
            let b = json_object_get(root, cstr!("b"));
            assert!((*b).type_ == json_type::JSON_OBJECT);
            assert_eq!(json_object_size(b), 0);
            let c = json_object_get(root, cstr!("c"));
            assert!((*c).type_ == json_type::JSON_STRING);
            assert_eq!("charlie", CStr::from_ptr(json_string_value(c)).to_str().unwrap());
            let d = json_object_get(root, cstr!("d"));
            assert!((*d).type_ == json_type::JSON_REAL);
            assert_eq!(8.75, json_real_value(d));
        }
    }
}
