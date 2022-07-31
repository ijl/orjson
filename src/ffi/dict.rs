// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[cfg(feature = "pydictiter")]
use pyo3_ffi::{PyObject, Py_hash_t, Py_ssize_t};

#[cfg(feature = "pydictiter")]
use std::os::raw::{c_char, c_void};

// dictobject.h
#[cfg(feature = "pydictiter")]
#[repr(C)]
pub struct PyDictObject {
    pub ob_refcnt: pyo3_ffi::Py_ssize_t,
    pub ob_type: *mut pyo3_ffi::PyTypeObject,
    pub ma_used: pyo3_ffi::Py_ssize_t,
    pub ma_version_tag: u64,
    pub ma_keys: *mut PyDictKeysObject,
    pub ma_values: *mut *mut PyObject,
}

// dict-common.h
#[cfg(feature = "pydictiter")]
#[repr(C)]
pub struct PyDictKeyEntry {
    pub me_hash: Py_hash_t,
    pub me_key: *mut PyObject,
    pub me_value: *mut PyObject,
}

// dict-common.h
#[cfg(feature = "pydictiter")]
#[repr(C)]
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
#[cfg(feature = "pydictiter")]
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
#[cfg(feature = "pydictiter")]
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

// dictobject.c
#[allow(non_snake_case)]
#[cfg(feature = "pydictiter")]
fn DK_ENTRIES(dk: *mut PyDictKeysObject) -> *mut PyDictKeyEntry {
    unsafe {
        std::mem::transmute::<*mut [c_char; 1], *mut u8>(std::ptr::addr_of_mut!((*dk).dk_indices))
            .offset((*dk).dk_size * DK_IXSIZE(dk)) as *mut PyDictKeyEntry
    }
}

#[cfg(feature = "pydictiter")]
pub struct PyDictIter {
    idx: usize,
    len: usize,
    values_ptr: *mut *mut pyo3_ffi::PyObject,
    indices_ptr: *mut PyDictKeyEntry,
}

#[cfg(feature = "pydictiter")]
impl PyDictIter {
    #[inline]
    pub fn from_pyobject(obj: *mut pyo3_ffi::PyObject) -> Self {
        unsafe {
            let dict_ptr = obj as *mut PyDictObject;
            PyDictIter {
                indices_ptr: DK_ENTRIES((*dict_ptr).ma_keys),
                values_ptr: (*dict_ptr).ma_values,
                idx: 0,
                len: (*(*dict_ptr).ma_keys).dk_nentries as usize,
            }
        }
    }
}

#[cfg(feature = "pydictiter")]
impl Iterator for PyDictIter {
    type Item = (*mut pyo3_ffi::PyObject, *mut pyo3_ffi::PyObject);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if unlikely!(self.idx >= self.len) {
                None
            } else if !self.values_ptr.is_null() {
                let key = (*(self.indices_ptr.add(self.idx))).me_key;
                let value = (*self.values_ptr).add(self.idx);
                self.idx += 1;
                Some((key, value))
            } else {
                let mut entry_ptr = self.indices_ptr.add(self.idx);
                while self.idx < self.len {
                    self.idx += 1;
                    if !(*entry_ptr).me_value.is_null() {
                        return Some(((*entry_ptr).me_key, (*entry_ptr).me_value));
                    }
                    entry_ptr = entry_ptr.add(1);
                }
                None
            }
        }
    }
}

#[cfg(not(feature = "pydictiter"))]
pub struct PyDictIter {
    dict_ptr: *mut pyo3_ffi::PyObject,
    pos: isize,
}

#[cfg(not(feature = "pydictiter"))]
impl PyDictIter {
    #[inline]
    pub fn from_pyobject(obj: *mut pyo3_ffi::PyObject) -> Self {
        unsafe {
            PyDictIter {
                dict_ptr: obj,
                pos: 0,
            }
        }
    }
}

#[cfg(not(feature = "pydictiter"))]
impl Iterator for PyDictIter {
    type Item = (*mut pyo3_ffi::PyObject, *mut pyo3_ffi::PyObject);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut key: *mut pyo3_ffi::PyObject = std::ptr::null_mut();
        let mut value: *mut pyo3_ffi::PyObject = std::ptr::null_mut();
        unsafe {
            if pyo3_ffi::_PyDict_Next(
                self.dict_ptr,
                &mut self.pos,
                &mut key,
                &mut value,
                std::ptr::null_mut(),
            ) == 1
            {
                Some((key, value))
            } else {
                None
            }
        }
    }
}
