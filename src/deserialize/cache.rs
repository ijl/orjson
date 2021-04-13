// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref::*;
use ahash::CallHasher;
use associative_cache::replacement::RoundRobinReplacement;
use associative_cache::*;
use once_cell::unsync::OnceCell;
use std::os::raw::c_void;

#[repr(transparent)]
pub struct CachedKey {
    ptr: *mut c_void,
}

unsafe impl Send for CachedKey {}
unsafe impl Sync for CachedKey {}

impl CachedKey {
    pub fn new(ptr: *mut pyo3::ffi::PyObject) -> CachedKey {
        CachedKey {
            ptr: ptr as *mut c_void,
        }
    }

    pub fn get(&mut self) -> *mut pyo3::ffi::PyObject {
        let ptr = self.ptr as *mut pyo3::ffi::PyObject;
        ffi!(Py_INCREF(ptr));
        ptr
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

pub fn cache_hash(key: &[u8]) -> u64 {
    <[u8]>::get_hash(&key, unsafe { &*HASH_BUILDER })
}
