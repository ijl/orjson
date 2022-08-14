// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::cache::*;
use crate::typeref::*;
use crate::unicode::*;
use std::ptr::NonNull;

pub fn get_unicode_key(key_str: &str) -> (*mut pyo3_ffi::PyObject, pyo3_ffi::Py_hash_t) {
    let pykey: *mut pyo3_ffi::PyObject;
    let pyhash: pyo3_ffi::Py_hash_t;
    if unlikely!(key_str.len() > 64) {
        pykey = unicode_from_str(key_str);
        pyhash = hash_str(pykey);
    } else {
        let hash = cache_hash(key_str.as_bytes());
        let map = unsafe {
            KEY_MAP
                .get_mut()
                .unwrap_or_else(|| unsafe { std::hint::unreachable_unchecked() })
        };
        let entry = map.entry(&hash).or_insert_with(
            || hash,
            || {
                let pyob = unicode_from_str(key_str);
                hash_str(pyob);
                CachedKey::new(pyob)
            },
        );
        pykey = entry.get();
        pyhash = unsafe { (*pykey.cast::<PyASCIIObject>()).hash }
    }
    (pykey, pyhash)
}

#[allow(dead_code)]
#[inline(always)]
pub fn parse_bool(val: bool) -> NonNull<pyo3_ffi::PyObject> {
    if val {
        parse_true()
    } else {
        parse_false()
    }
}

#[inline(always)]
pub fn parse_true() -> NonNull<pyo3_ffi::PyObject> {
    ffi!(Py_INCREF(TRUE));
    nonnull!(TRUE)
}

#[inline(always)]
pub fn parse_false() -> NonNull<pyo3_ffi::PyObject> {
    ffi!(Py_INCREF(FALSE));
    nonnull!(FALSE)
}
#[inline(always)]
pub fn parse_i64(val: i64) -> NonNull<pyo3_ffi::PyObject> {
    nonnull!(ffi!(PyLong_FromLongLong(val)))
}

#[inline(always)]
pub fn parse_u64(val: u64) -> NonNull<pyo3_ffi::PyObject> {
    nonnull!(ffi!(PyLong_FromUnsignedLongLong(val)))
}

#[inline(always)]
pub fn parse_f64(val: f64) -> NonNull<pyo3_ffi::PyObject> {
    nonnull!(ffi!(PyFloat_FromDouble(val)))
}

#[inline(always)]
pub fn parse_none() -> NonNull<pyo3_ffi::PyObject> {
    ffi!(Py_INCREF(NONE));
    nonnull!(NONE)
}
