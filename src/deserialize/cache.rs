// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use associative_cache::replacement::RoundRobinReplacement;
use associative_cache::*;
use core::ffi::c_void;
use once_cell::unsync::OnceCell;
use std::hash::Hasher;

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
        debug_assert!(ffi!(Py_REFCNT(ptr)) >= 1);
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
    // try to omit code for >64 path in ahash
    debug_assert!(key.len() <= 64);
    #[cfg(feature = "intrinsics")]
    unsafe {
        core::intrinsics::assume(key.len() <= 64);
    };
    let mut hasher = ahash::AHasher::default();
    hasher.write(key);
    hasher.finish()
}
