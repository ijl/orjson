// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::cache::*;
use crate::deserialize::pyobject::*;
use crate::deserialize::DeserializeError;
use crate::typeref::*;
use crate::unicode::*;
use crate::yyjson::*;
use std::borrow::Cow;
use std::os::raw::c_char;
use std::ptr::null;
use std::ptr::NonNull;

const YYJSON_TYPE_MASK: u8 = 0x07;
const YYJSON_TYPE_BIT: u8 = 3;
const YYJSON_SUBTYPE_MASK: u8 = 0x18;
const YYJSON_TAG_BIT: u8 = 8;

const YYJSON_SUBTYPE_UINT: u8 = 0 << 3;
const YYJSON_SUBTYPE_SINT: u8 = 1 << 3;
const YYJSON_SUBTYPE_REAL: u8 = 2 << 3;

const YYJSON_TYPE_NONE: u8 = 0;
const YYJSON_TYPE_RAW: u8 = 1;
const YYJSON_TYPE_NULL: u8 = 2;
const YYJSON_TYPE_BOOL: u8 = 3;
const YYJSON_TYPE_NUM: u8 = 4;
const YYJSON_TYPE_STR: u8 = 5;
const YYJSON_TYPE_ARR: u8 = 6;
const YYJSON_TYPE_OBJ: u8 = 7;

fn yyjson_doc_get_root(doc: *mut yyjson_doc) -> *mut yyjson_val {
    unsafe { (*doc).root }
}

fn unsafe_yyjson_get_len(val: *mut yyjson_val) -> usize {
    unsafe { ((*val).tag >> YYJSON_TAG_BIT) as usize }
}

fn unsafe_yyjson_get_bool(val: *mut yyjson_val) -> bool {
    unsafe { (((*val).tag as u8 & YYJSON_SUBTYPE_MASK) >> YYJSON_TYPE_BIT) != 0 }
}

fn yyjson_obj_iter_get_val(key: *mut yyjson_val) -> *mut yyjson_val {
    unsafe { key.add(1) }
}

fn unsafe_yyjson_get_first(ctn: *mut yyjson_val) -> *mut yyjson_val {
    unsafe { ctn.add(1) }
}

fn yyjson_read_max_memory_usage(len: u64) -> u64 {
    (12 * len) + 256
}

pub fn deserialize_yyjson(
    data: &'static str,
) -> Result<NonNull<pyo3_ffi::PyObject>, DeserializeError<'static>> {
    unsafe {
        let allocator: *mut yyjson_alc;
        if yyjson_read_max_memory_usage(data.as_bytes().len() as u64) < YYJSON_BUFFER_SIZE as u64 {
            allocator = std::ptr::addr_of_mut!(*YYJSON_ALLOC);
        } else {
            allocator = std::ptr::null_mut();
        }
        let mut err = yyjson_read_err {
            code: YYJSON_READ_SUCCESS,
            msg: null(),
            pos: 0,
        };
        let doc: *mut yyjson_doc = yyjson_read_opts(
            data.as_bytes().as_ptr() as *mut c_char,
            data.as_bytes().len() as u64,
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

#[repr(u8)]
enum ElementType {
    Null,
    Bool,
    Int64,
    Uint64,
    Double,
    String,
    Array,
    Object,
    Other,
}

impl ElementType {
    fn from_tag(elem: *mut yyjson_val) -> Self {
        let tag = unsafe { (*elem).tag as u8 };
        match tag & YYJSON_TYPE_MASK as u8 {
            YYJSON_TYPE_STR => Self::String,
            YYJSON_TYPE_ARR => Self::Array,
            YYJSON_TYPE_NULL => Self::Null,
            YYJSON_TYPE_BOOL => Self::Bool,
            YYJSON_TYPE_OBJ => Self::Object,
            YYJSON_TYPE_NUM => match tag & YYJSON_SUBTYPE_MASK {
                YYJSON_SUBTYPE_UINT => Self::Uint64,
                YYJSON_SUBTYPE_SINT => Self::Int64,
                YYJSON_SUBTYPE_REAL => Self::Double,
                _ => Self::Other,
            },
            YYJSON_TYPE_RAW => unreachable!(),
            YYJSON_TYPE_NONE => unreachable!(),
            _ => unreachable!(),
        }
    }
}

fn parse_yy_string(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    nonnull!(unicode_from_str(str_from_slice!(
        (*elem).uni.str_ as *const u8,
        unsafe_yyjson_get_len(elem)
    )))
}

fn parse_yy_array(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    unsafe {
        let len = unsafe_yyjson_get_len(elem);
        let list = ffi!(PyList_New(len as isize));
        if len == 0 {
            return nonnull!(list);
        }
        let mut iter: yyjson_arr_iter = yyjson_arr_iter {
            idx: 0,
            max: len as u64,
            cur: unsafe_yyjson_get_first(elem),
        };
        for idx in 0..=len.saturating_sub(1) {
            let val = yyjson_arr_iter_next(&mut iter);
            let each = parse_node(val);
            ffi!(PyList_SET_ITEM(list, idx as isize, each.as_ptr()));
        }
        nonnull!(list)
    }
}

fn parse_yy_object(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    unsafe {
        let len = unsafe_yyjson_get_len(elem);
        if len == 0 {
            return nonnull!(ffi!(PyDict_New()));
        }
        let dict = ffi!(_PyDict_NewPresized(len as isize));
        let mut iter = yyjson_obj_iter {
            idx: 0,
            max: len as u64,
            cur: unsafe_yyjson_get_first(elem),
            obj: elem,
        };
        for _ in 0..=len.saturating_sub(1) {
            let key = yyjson_obj_iter_next(&mut iter);
            let val = yyjson_obj_iter_get_val(key);
            let key_str = str_from_slice!((*key).uni.str_ as *const u8, unsafe_yyjson_get_len(key));
            let pyval = parse_node(val);
            let pykey: *mut pyo3_ffi::PyObject;
            let pyhash: pyo3_ffi::Py_hash_t;
            if unlikely!(key_str.len() > 64) {
                pykey = unicode_from_str(&key_str);
                pyhash = hash_str(pykey);
            } else {
                let hash = cache_hash(key_str.as_bytes());
                {
                    let map = unsafe {
                        KEY_MAP
                            .get_mut()
                            .unwrap_or_else(|| unsafe { std::hint::unreachable_unchecked() })
                    };
                    let entry = map.entry(&hash).or_insert_with(
                        || hash,
                        || {
                            let pyob = unicode_from_str(&key_str);
                            hash_str(pyob);
                            CachedKey::new(pyob)
                        },
                    );
                    pykey = entry.get();
                    pyhash = unsafe { (*pykey.cast::<PyASCIIObject>()).hash }
                }
            };
            let _ = ffi!(_PyDict_SetItem_KnownHash(
                dict,
                pykey,
                pyval.as_ptr(),
                pyhash
            ));
            ffi!(Py_DECREF(pykey));
            ffi!(Py_DECREF(pyval.as_ptr()));
        }
        nonnull!(dict)
    }
}

pub fn parse_node(elem: *mut yyjson_val) -> NonNull<pyo3_ffi::PyObject> {
    match ElementType::from_tag(elem) {
        ElementType::String => parse_yy_string(elem),
        ElementType::Double => parse_f64(unsafe { (*elem).uni.f64_ }),
        ElementType::Uint64 => parse_u64(unsafe { (*elem).uni.u64_ }),
        ElementType::Int64 => parse_i64(unsafe { (*elem).uni.i64_ }),
        ElementType::Bool => parse_bool(unsafe { unsafe_yyjson_get_bool(elem) }),
        ElementType::Null => parse_none(),
        ElementType::Array => parse_yy_array(elem),
        ElementType::Object => parse_yy_object(elem),
        ElementType::Other => unreachable!(),
    }
}
