// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use core::ffi::c_char;
use core::ptr::{NonNull, null, null_mut};
use std::borrow::Cow;
use std::collections::HashMap;

use pyo3_ffi::PyObject;

use crate::deserialize::default::deserialize_default;
use crate::deserialize::DeserializeError;
use crate::deserialize::deserializer::Callable;
use crate::deserialize::pyobject::*;
use crate::ffi::yyjson::*;
use crate::str::unicode_from_str;
use crate::typeref::{YYJSON_ALLOC, YYJSON_BUFFER_SIZE, yyjson_init};

const YYJSON_TAG_BIT: u8 = 8;

const YYJSON_VAL_SIZE: usize = core::mem::size_of::<yyjson_val>();

const TAG_ARRAY: u8 = 0b00000110;
const TAG_DOUBLE: u8 = 0b00010100;
const TAG_FALSE: u8 = 0b00000011;
const TAG_INT64: u8 = 0b00001100;
const TAG_NULL: u8 = 0b00000010;
const TAG_OBJECT: u8 = 0b00000111;
const TAG_STRING: u8 = 0b00000101;
const TAG_TRUE: u8 = 0b00001011;
const TAG_UINT64: u8 = 0b00000100;

macro_rules! is_yyjson_tag {
    ($elem:expr, $tag:expr) => {
        unsafe { (*$elem).tag as u8 == $tag }
    };
}

fn yyjson_doc_get_root(doc: *mut yyjson_doc) -> *mut yyjson_val {
    unsafe { (*doc).root }
}

fn unsafe_yyjson_get_len(val: *mut yyjson_val) -> usize {
    unsafe { ((*val).tag >> YYJSON_TAG_BIT) as usize }
}

fn unsafe_yyjson_get_first(ctn: *mut yyjson_val) -> *mut yyjson_val {
    unsafe { ctn.add(1) }
}

fn yyjson_read_max_memory_usage(len: usize) -> usize {
    (12 * len) + 256
}

fn unsafe_yyjson_is_ctn(val: *mut yyjson_val) -> bool {
    unsafe { (*val).tag as u8 & 0b00000110 == 0b00000110 }
}

fn unsafe_yyjson_get_next_container(val: *mut yyjson_val) -> *mut yyjson_val {
    unsafe { ((val as *mut u8).add((*val).uni.ofs)) as *mut yyjson_val }
}

fn unsafe_yyjson_get_next_non_container(val: *mut yyjson_val) -> *mut yyjson_val {
    unsafe { ((val as *mut u8).add(YYJSON_VAL_SIZE)) as *mut yyjson_val }
}

pub fn deserialize_yyjson<const WITH_DEFAULT: bool>(
    data: &'static str, default: &Option<HashMap<String, Callable>>
) -> Result<NonNull<pyo3_ffi::PyObject>, DeserializeError<'static>> {
    let mut err = yyjson_read_err {
        code: YYJSON_READ_SUCCESS,
        msg: null(),
        pos: 0,
    };
    let doc = if yyjson_read_max_memory_usage(data.len()) < YYJSON_BUFFER_SIZE {
        read_doc_with_buffer(data, &mut err)
    } else {
        read_doc_default(data, &mut err)
    };
    if unlikely!(doc.is_null()) {
        let msg: Cow<str> = unsafe { core::ffi::CStr::from_ptr(err.msg).to_string_lossy() };
        Err(DeserializeError::from_yyjson(msg, err.pos as i64, data))
    } else {
        let val = yyjson_doc_get_root(doc);

        if unlikely!(!unsafe_yyjson_is_ctn(val)) {
            let pyval = match ElementType::from_tag(val) {
                ElementType::String => parse_yy_string(val),
                ElementType::Uint64 => parse_yy_u64(val),
                ElementType::Int64 => parse_yy_i64(val),
                ElementType::Double => parse_yy_f64(val),
                ElementType::Null => parse_none(),
                ElementType::True => parse_true(),
                ElementType::False => parse_false(),
                ElementType::Array => unreachable!(),
                ElementType::Object => unreachable!(),
            };
            unsafe { yyjson_doc_free(doc) };
            Ok(pyval)
        } else if is_yyjson_tag!(val, TAG_ARRAY) {
            let pyval = nonnull!(ffi!(PyList_New(unsafe_yyjson_get_len(val) as isize)));
            if unsafe_yyjson_get_len(val) > 0 {
                populate_yy_array::<WITH_DEFAULT>(pyval.as_ptr(), val, default);
            }
            unsafe { yyjson_doc_free(doc) };
            Ok(pyval)
        } else {
            let mut pyval = nonnull!(ffi!(_PyDict_NewPresized(
                unsafe_yyjson_get_len(val) as isize
            )));
            populate_yy_object_wrapper::<WITH_DEFAULT>(default, val, &mut pyval);
            unsafe { yyjson_doc_free(doc) };
            Ok(pyval)
        }
    }
}

fn read_doc_default(data: &'static str, err: &mut yyjson_read_err) -> *mut yyjson_doc {
    unsafe { yyjson_read_opts(data.as_ptr() as *mut c_char, data.len(), null_mut(), err) }
}

fn read_doc_with_buffer(data: &'static str, err: &mut yyjson_read_err) -> *mut yyjson_doc {
    unsafe {
        yyjson_read_opts(
            data.as_ptr() as *mut c_char,
            data.len(),
            &YYJSON_ALLOC.get_or_init(yyjson_init).alloc,
            err,
        )
    }
}

enum ElementType {
    String,
    Uint64,
    Int64,
    Double,
    Null,
    True,
    False,
    Array,
    Object,
}

impl ElementType {
    fn from_tag(elem: *mut yyjson_val) -> Self {
        match unsafe { (*elem).tag as u8 } {
            TAG_STRING => Self::String,
            TAG_UINT64 => Self::Uint64,
            TAG_INT64 => Self::Int64,
            TAG_DOUBLE => Self::Double,
            TAG_NULL => Self::Null,
            TAG_TRUE => Self::True,
            TAG_FALSE => Self::False,
            TAG_ARRAY => Self::Array,
            TAG_OBJECT => Self::Object,
            _ => unreachable!(),
        }
    }
}

#[inline(always)]
fn parse_yy_string(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    nonnull!(unicode_from_str(str_from_slice!(
        (*elem).uni.str_ as *const u8,
        unsafe_yyjson_get_len(elem)
    )))
}

#[inline(always)]
fn parse_yy_u64(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    parse_u64(unsafe { (*elem).uni.u64_ })
}

#[inline(always)]
fn parse_yy_i64(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    parse_i64(unsafe { (*elem).uni.i64_ })
}

#[inline(always)]
fn parse_yy_f64(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    parse_f64(unsafe { (*elem).uni.f64_ })
}

macro_rules! append_to_list {
    ($dptr:expr, $pyval:expr) => {
        unsafe {
            core::ptr::write($dptr, $pyval);
            $dptr = $dptr.add(1);
        }
    };
}

#[inline(never)]
fn populate_yy_array<const WITH_DEFAULT: bool>(
    list: *mut pyo3_ffi::PyObject,
    elem: *mut yyjson_val,
    default: &Option<HashMap<String, Callable>>
) {
    unsafe {
        let len = unsafe_yyjson_get_len(elem);
        assume!(len >= 1);
        let mut next = unsafe_yyjson_get_first(elem);
        let mut dptr = (*(list as *mut pyo3_ffi::PyListObject)).ob_item;

        for _ in 0..len {
            let val = next;
            if unlikely!(unsafe_yyjson_is_ctn(val)) {
                next = unsafe_yyjson_get_next_container(val);
                if is_yyjson_tag!(val, TAG_ARRAY) {
                    let pyval = nonnull!(ffi!(PyList_New(unsafe_yyjson_get_len(val) as isize)));
                    append_to_list!(dptr, pyval.as_ptr());
                    if unsafe_yyjson_get_len(val) > 0 {
                        populate_yy_array::<WITH_DEFAULT>(pyval.as_ptr(), val, default);
                    }
                } else {
                    let mut pyval = nonnull!(ffi!(_PyDict_NewPresized(
                        unsafe_yyjson_get_len(val) as isize
                    )));
                    populate_yy_object_wrapper::<WITH_DEFAULT>(default, val, &mut pyval);
                    append_to_list!(dptr, pyval.as_ptr());

                }
            } else {
                next = unsafe_yyjson_get_next_non_container(val);
                let pyval = match ElementType::from_tag(val) {
                    ElementType::String => parse_yy_string(val),
                    ElementType::Uint64 => parse_yy_u64(val),
                    ElementType::Int64 => parse_yy_i64(val),
                    ElementType::Double => parse_yy_f64(val),
                    ElementType::Null => parse_none(),
                    ElementType::True => parse_true(),
                    ElementType::False => parse_false(),
                    ElementType::Array => unreachable!(),
                    ElementType::Object => unreachable!(),
                };
                append_to_list!(dptr, pyval.as_ptr());
            }
        }
    }
}

#[inline(always)]
fn populate_yy_object_wrapper<const WITH_DEFAULT: bool>(default: &Option<HashMap<String, Callable>>, val: *mut yyjson_val, pyval: &mut NonNull<PyObject>) {
    if unsafe_yyjson_get_len(val) > 0 {
        let overriding_obj = populate_yy_object::<WITH_DEFAULT>(pyval.as_ptr(), val, default);
        if WITH_DEFAULT {
            if let Some(new_obj) = overriding_obj {
                *pyval = unsafe { NonNull::new_unchecked(new_obj) };
            }
        }
    }
}

macro_rules! add_to_dict {
    ($dict:expr, $pykey:expr, $pyval:expr) => {
        unsafe { pyo3_ffi::_PyDict_SetItem_KnownHash($dict, $pykey, $pyval, str_hash!($pykey)) }
    };
}

#[inline(never)]
fn populate_yy_object<const WITH_DEFAULT: bool>(
    dict: *mut PyObject,
    elem: *mut yyjson_val,
    default: &Option<HashMap<String, Callable>>
) -> Option<*mut PyObject> {
    unsafe {
        let len = unsafe_yyjson_get_len(elem);
        assume!(len >= 1);
        let mut next_key = unsafe_yyjson_get_first(elem);
        let mut next_val = next_key.add(1);
        for _ in 0..len {
            let key = next_key;
            let val = next_val;
            let key_str = str_from_slice!((*key).uni.str_ as *const u8, unsafe_yyjson_get_len(key));
            let pykey = get_unicode_key(key_str);
            let mut pyval;
            if unlikely!(unsafe_yyjson_is_ctn(val)) {
                next_key = unsafe_yyjson_get_next_container(val);
                next_val = next_key.add(1);
                if is_yyjson_tag!(val, TAG_ARRAY) {
                    pyval = nonnull!(ffi!(PyList_New(unsafe_yyjson_get_len(val) as isize)));
                    add_to_dict!(dict, pykey, pyval.as_ptr());
                    reverse_pydict_incref!(pykey);
                    reverse_pydict_incref!(pyval.as_ptr());
                    if unsafe_yyjson_get_len(val) > 0 {
                        populate_yy_array::<WITH_DEFAULT>(pyval.as_ptr(), val, default);
                    }
                } else {
                    pyval = nonnull!(ffi!(_PyDict_NewPresized(
                        unsafe_yyjson_get_len(val) as isize
                    )));
                    populate_yy_object_wrapper::<WITH_DEFAULT>(default, val, &mut pyval);
                    add_to_dict!(dict, pykey, pyval.as_ptr());
                    reverse_pydict_incref!(pykey);
                    reverse_pydict_incref!(pyval.as_ptr());
                }
            } else {
                next_key = unsafe_yyjson_get_next_non_container(val);
                next_val = next_key.add(1);
                pyval = match ElementType::from_tag(val) {
                    ElementType::String => parse_yy_string(val),
                    ElementType::Uint64 => parse_yy_u64(val),
                    ElementType::Int64 => parse_yy_i64(val),
                    ElementType::Double => parse_yy_f64(val),
                    ElementType::Null => parse_none(),
                    ElementType::True => parse_true(),
                    ElementType::False => parse_false(),
                    ElementType::Array => unreachable!(),
                    ElementType::Object => unreachable!(),
                };
                add_to_dict!(dict, pykey, pyval.as_ptr());
                reverse_pydict_incref!(pykey);
                reverse_pydict_incref!(pyval.as_ptr());
            }
            if WITH_DEFAULT && len == 1 {
                if let Some(callable) = default.as_ref().unwrap_unchecked().get(key_str) {
                    if let Ok(deserialized_obj) = deserialize_default(callable, pyval) {
                        return Some(deserialized_obj);
                    }
                }
            }
        }
    }
    None
}
