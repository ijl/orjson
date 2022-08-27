// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref::*;
use associative_cache::replacement::RoundRobinReplacement;
use associative_cache::*;
use once_cell::unsync::OnceCell;
use std::hash::BuildHasher;
use std::hash::Hasher;
use std::os::raw::c_void;

#[repr(transparent)]
pub struct CachedKey {
    ptr: *mut c_void,
}

unsafe impl Send for CachedKey {}
unsafe impl Sync for CachedKey {}

impl CachedKey {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> CachedKey {
        CachedKey {
            ptr: ptr as *mut c_void,
        }
    }

    pub fn get(&mut self) -> *mut pyo3_ffi::PyObject {
        let ptr = self.ptr as *mut pyo3_ffi::PyObject;
        ffi!(Py_INCREF(ptr));
        ptr
    }
}

impl Drop for CachedKey {
    fn drop(&mut self) {
        ffi!(Py_DECREF(self.ptr as *mut pyo3_ffi::PyObject));
    }
}

pub type KeyMap =
    AssociativeCache<u64, CachedKey, Capacity1024, HashDirectMapped, RoundRobinReplacement>;

pub static mut KEY_MAP: OnceCell<KeyMap> = OnceCell::new();

pub fn cache_hash(key: &[u8]) -> u64 {
    let mut hasher = unsafe { (*HASH_BUILDER).build_hasher() };
    hasher.write(key);
    hasher.finish()
}
