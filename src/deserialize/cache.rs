// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use associative_cache::replacement::RoundRobinReplacement;
use associative_cache::*;
use once_cell::unsync::OnceCell;
use std::os::raw::c_void;

#[derive(Clone)]
pub struct CachedKey {
    ptr: *mut c_void,
    hash: pyo3::ffi::Py_hash_t,
}

unsafe impl Send for CachedKey {}
unsafe impl Sync for CachedKey {}

impl CachedKey {
    pub fn new(ptr: *mut pyo3::ffi::PyObject, hash: pyo3::ffi::Py_hash_t) -> CachedKey {
        CachedKey {
            ptr: ptr as *mut c_void,
            hash: hash,
        }
    }

    pub fn get(&mut self) -> (*mut pyo3::ffi::PyObject, pyo3::ffi::Py_hash_t) {
        let ptr = self.ptr as *mut pyo3::ffi::PyObject;
        ffi!(Py_INCREF(ptr));
        (ptr, self.hash)
    }
}

impl Drop for CachedKey {
    fn drop(&mut self) {
        ffi!(Py_DECREF(self.ptr as *mut pyo3::ffi::PyObject));
    }
}

pub type KeyMap =
    AssociativeCache<u64, CachedKey, Capacity512, HashDirectMapped, RoundRobinReplacement>;

pub static mut KEY_MAP: OnceCell<KeyMap> = OnceCell::new();
