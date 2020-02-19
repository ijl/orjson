// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::encode::*;
use crate::exc::*;
use crate::typeref::*;
use crate::unicode::*;
use serde::ser::{Serialize, SerializeMap, Serializer};
use smallvec::SmallVec;
use std::ptr::NonNull;

pub struct DictSortedKey {
    ptr: *mut pyo3::ffi::PyObject,
    opts: u16,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3::ffi::PyObject>>,
}

impl DictSortedKey {
    pub fn new(
        ptr: *mut pyo3::ffi::PyObject,
        opts: u16,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3::ffi::PyObject>>,
    ) -> Self {
        DictSortedKey {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }
}

impl<'p> Serialize for DictSortedKey {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let len = ffi!(PyDict_Size(self.ptr)) as usize;
        if len == 0 {
            serializer.serialize_map(Some(0)).unwrap().end()
        } else {
            let mut items: SmallVec<[(&str, *mut pyo3::ffi::PyObject); 8]> =
                SmallVec::with_capacity(len);
            let mut pos = 0isize;
            let mut str_size: pyo3::ffi::Py_ssize_t = 0;
            let mut key: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
            let mut value: *mut pyo3::ffi::PyObject = std::ptr::null_mut();
            while unsafe { pyo3::ffi::PyDict_Next(self.ptr, &mut pos, &mut key, &mut value) != 0 } {
                if unlikely!((*key).ob_type != STR_TYPE) {
                    err!("Dict key must be str")
                }
                let data = read_utf8_from_str(key, &mut str_size);
                if unlikely!(data.is_null()) {
                    err!(INVALID_STR)
                }
                items.push((str_from_slice!(data, str_size), value));
            }

            items.sort_unstable_by(|a, b| a.0.cmp(b.0));

            let mut map = serializer.serialize_map(None).unwrap();
            for (key, val) in items.iter() {
                map.serialize_entry(
                    key,
                    &SerializePyObject::new(
                        *val,
                        None,
                        self.opts,
                        self.default_calls,
                        self.recursion + 1,
                        self.default,
                    ),
                )?;
            }
            map.end()
        }
    }
}
