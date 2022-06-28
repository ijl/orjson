// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3_ffi::{PyObject, Py_hash_t, Py_ssize_t};
use std::os::raw::{c_char, c_void};

#[allow(non_snake_case)]
#[inline(always)]
pub unsafe fn PyDict_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    (*op.cast::<PyDictObject>()).ma_used
}

// dictobject.h
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyDictObject {
    pub ob_refcnt: pyo3_ffi::Py_ssize_t,
    pub ob_type: *mut pyo3_ffi::PyTypeObject,
    pub ma_used: pyo3_ffi::Py_ssize_t,
    pub ma_version_tag: u64,
    pub ma_keys: *mut PyDictKeysObject,
    pub ma_values: *mut *mut PyObject,
}

// dict-common.h
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyDictKeyEntry {
    pub me_hash: Py_hash_t,
    pub me_key: *mut PyObject,
    pub me_value: *mut PyObject,
}

// dict-common.h
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PyDictKeysObject {
    pub dk_refcnt: Py_ssize_t,
    pub dk_size: Py_ssize_t,
    pub dk_lookup: *mut c_void, // dict_lookup_func
    pub dk_usable: Py_ssize_t,
    pub dk_nentries: Py_ssize_t,
    pub dk_indices: [c_char; 1],
}

// dictobject.c
#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
fn DK_IXSIZE(dk: *mut PyDictKeysObject) -> isize {
    unsafe {
        if (*dk).dk_size <= 0xff {
            1
        } else if (*dk).dk_size <= 0xffff {
            2
        } else if (*dk).dk_size <= 0xffffffff {
            4
        } else {
            8
        }
    }
}

// dictobject.c
#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
fn DK_IXSIZE(dk: *mut PyDictKeysObject) -> isize {
    unsafe {
        if (*dk).dk_size <= 0xff {
            1
        } else if (*dk).dk_size <= 0xffff {
            2
        } else {
            4
        }
    }
}

pub struct PyDictIter {
    dict_ptr: *mut PyDictObject,
    indices_ptr: *mut PyDictKeyEntry,
    idx: usize,
    len: usize,
}

impl PyDictIter {
    pub fn new(dict_ptr: *mut PyDictObject) -> Self {
        unsafe {
            let dk = (*dict_ptr).ma_keys;
            let offset = (*dk).dk_size * DK_IXSIZE(dk);
            let indices_ptr = std::mem::transmute::<*mut [c_char; 1], *mut u8>(
                std::ptr::addr_of_mut!((*dk).dk_indices),
            )
            .offset(offset) as *mut PyDictKeyEntry;
            let len = PyDict_GET_SIZE(dict_ptr as *mut pyo3_ffi::PyObject) as usize;
            PyDictIter {
                dict_ptr: dict_ptr,
                indices_ptr: indices_ptr,
                idx: 0,
                len: len,
            }
        }
    }
}

impl Iterator for PyDictIter {
    type Item = (*mut pyo3_ffi::PyObject, *mut pyo3_ffi::PyObject);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if unlikely!(self.idx == self.len) {
                None
            } else if !(*self.dict_ptr).ma_values.is_null() {
                let entry_ptr: *mut PyDictKeyEntry = self.indices_ptr.add(self.idx);
                let value = (*(*self.dict_ptr).ma_values).add(self.idx);
                self.idx += 1;
                Some(((*entry_ptr).me_key, value))
            } else {
                let mut entry_ptr: *mut PyDictKeyEntry = self.indices_ptr.add(self.idx);
                while self.idx < self.len && (*entry_ptr).me_value.is_null() {
                    entry_ptr = entry_ptr.add(1);
                }
                let value = (*entry_ptr).me_value;
                self.idx += 1;
                Some(((*entry_ptr).me_key, value))
            }
        }
    }
}

impl ExactSizeIterator for PyDictIter {
    fn len(&self) -> usize {
        self.idx - self.len
    }
}
