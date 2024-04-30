// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::cache::*;
use crate::str::{hash_str, unicode_from_str};
use crate::typeref::{FALSE, NONE, TRUE};
use core::ptr::NonNull;

#[inline(always)]
pub fn get_unicode_key(key_str: &str) -> *mut pyo3_ffi::PyObject {
    if unlikely!(key_str.len() > 64) {
        let pyob = unicode_from_str(key_str);
        hash_str(pyob);
        pyob
    } else {
        let hash = cache_hash(key_str.as_bytes());
        unsafe {
            let entry = KEY_MAP
                .get_mut()
                .unwrap_or_else(|| unreachable!())
                .entry(&hash)
                .or_insert_with(
                    || hash,
                    || {
                        let pyob = unicode_from_str(key_str);
                        hash_str(pyob);
                        CachedKey::new(pyob)
                    },
                );
            entry.get()
        }
    }
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
    nonnull!(use_immortal!(TRUE))
}

#[inline(always)]
pub fn parse_false() -> NonNull<pyo3_ffi::PyObject> {
    nonnull!(use_immortal!(FALSE))
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
    nonnull!(use_immortal!(NONE))
}
