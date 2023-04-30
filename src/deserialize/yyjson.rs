// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::pyobject::*;
use crate::deserialize::DeserializeError;
use crate::str::*;
use crate::typeref::*;
use crate::yyjson::*;
use std::borrow::Cow;
use std::os::raw::c_char;
use std::ptr::{null, null_mut, NonNull};

const YYJSON_TYPE_MASK: u8 = 0x07;
const YYJSON_SUBTYPE_MASK: u8 = 0x18;
const YYJSON_TAG_BIT: u8 = 8;

const YYJSON_VAL_SIZE: usize = std::mem::size_of::<yyjson_val>();

const TYPE_AND_SUBTYPE_MASK: u8 = YYJSON_TYPE_MASK | YYJSON_SUBTYPE_MASK;

const TAG_ARRAY: u8 = 0b00000110;
const TAG_DOUBLE: u8 = 0b00010100;
const TAG_FALSE: u8 = 0b00000011;
const TAG_INT64: u8 = 0b00001100;
const TAG_NULL: u8 = 0b00000010;
const TAG_OBJECT: u8 = 0b00000111;
const TAG_STRING: u8 = 0b00000101;
const TAG_TRUE: u8 = 0b00001011;
const TAG_UINT64: u8 = 0b00000100;

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
    unsafe { ((*val).tag as u8) & 0b00000110 == 0b00000110 }
}

fn unsafe_yyjson_get_next(val: *mut yyjson_val) -> *mut yyjson_val {
    unsafe {
        if unlikely!(unsafe_yyjson_is_ctn(val)) {
            ((val as *mut u8).add((*val).uni.ofs)) as *mut yyjson_val
        } else {
            ((val as *mut u8).add(YYJSON_VAL_SIZE)) as *mut yyjson_val
        }
    }
}

pub fn deserialize_yyjson(
    data: &'static str,
) -> Result<NonNull<pyo3_ffi::PyObject>, DeserializeError<'static>> {
    unsafe {
        let allocator = if yyjson_read_max_memory_usage(data.len()) < YYJSON_BUFFER_SIZE {
            YYJSON_ALLOC.get_or_init(yyjson_init) as *const yyjson_alc
        } else {
            null_mut()
        };
        let mut err = yyjson_read_err {
            code: YYJSON_READ_SUCCESS,
            msg: null(),
            pos: 0,
        };
        let doc: *mut yyjson_doc = yyjson_read_opts(
            data.as_ptr() as *mut c_char,
            data.len(),
            YYJSON_READ_NOFLAG,
            allocator,
            &mut err,
        );
        if unlikely!(doc.is_null()) {
            let msg: Cow<str> = std::ffi::CStr::from_ptr(err.msg).to_string_lossy();
            Err(DeserializeError::from_yyjson(msg, err.pos as i64, data))
        } else {
            let root = yyjson_doc_get_root(doc);
            let ret = parse_node(root);
            yyjson_doc_free(doc);
            Ok(ret)
        }
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
        match unsafe { (*elem).tag as u8 & TYPE_AND_SUBTYPE_MASK } {
            TAG_STRING => Self::String,
            TAG_UINT64 => Self::Uint64,
            TAG_INT64 => Self::Int64,
            TAG_DOUBLE => Self::Double,
            TAG_NULL => Self::Null,
            TAG_TRUE => Self::True,
            TAG_FALSE => Self::False,
            TAG_ARRAY => Self::Array,
            TAG_OBJECT => Self::Object,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

fn parse_yy_string(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    nonnull!(unicode_from_str(str_from_slice!(
        (*elem).uni.str_ as *const u8,
        unsafe_yyjson_get_len(elem)
    )))
}

#[inline(never)]
fn parse_yy_array(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    unsafe {
        let len = unsafe_yyjson_get_len(elem);
        let list = ffi!(PyList_New(len as isize));
        if len == 0 {
            return nonnull!(list);
        }
        let mut cur = unsafe_yyjson_get_first(elem);
        for idx in 0..=len - 1 {
            let next = unsafe_yyjson_get_next(cur);
            let val = parse_node(cur).as_ptr();
            ffi!(PyList_SET_ITEM(list, idx as isize, val));
            cur = next;
        }
        nonnull!(list)
    }
}

#[inline(never)]
fn parse_yy_object(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    unsafe {
        let len = unsafe_yyjson_get_len(elem);
        if len == 0 {
            return nonnull!(ffi!(PyDict_New()));
        }
        let mut key = unsafe_yyjson_get_first(elem);
        let dict = ffi!(_PyDict_NewPresized(len as isize));
        for _ in 0..=len - 1 {
            let key_str = str_from_slice!((*key).uni.str_ as *const u8, unsafe_yyjson_get_len(key));
            let (pykey, pyhash) = get_unicode_key(key_str);
            let pyval = parse_node(key.add(1)).as_ptr();
            let _ = ffi!(_PyDict_SetItem_KnownHash(dict, pykey, pyval, pyhash));
            py_decref_without_destroy!(pykey);
            py_decref_without_destroy!(pyval);
            key = unsafe_yyjson_get_next(key.add(1));
        }
        nonnull!(dict)
    }
}

pub fn parse_node(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    match ElementType::from_tag(elem) {
        ElementType::String => parse_yy_string(elem),
        ElementType::Uint64 => parse_u64(unsafe { (*elem).uni.u64_ }),
        ElementType::Int64 => parse_i64(unsafe { (*elem).uni.i64_ }),
        ElementType::Double => parse_f64(unsafe { (*elem).uni.f64_ }),
        ElementType::Null => parse_none(),
        ElementType::True => parse_true(),
        ElementType::False => parse_false(),
        ElementType::Array => parse_yy_array(elem),
        ElementType::Object => parse_yy_object(elem),
    }
}
