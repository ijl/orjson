// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::ptr::NonNull;

pub struct PyTupleIterator {
    list: *mut pyo3::ffi::PyObject,
    len: isize,
    idx: isize,
}

impl PyTupleIterator {
    pub fn new(list: *mut pyo3::ffi::PyObject) -> Self {
        PyTupleIterator {
            list: list,
            len: ffi!(PyTuple_GET_SIZE(list)),
            idx: 0,
        }
    }
}

impl Iterator for PyTupleIterator {
    type Item = NonNull<pyo3::ffi::PyObject>;

    #[inline]
    fn next(&mut self) -> Option<NonNull<pyo3::ffi::PyObject>> {
        if self.len == self.idx {
            None
        } else {
            let item = unsafe {
                NonNull::new_unchecked(ffi!(PyTuple_GET_ITEM(self.list, self.idx as isize)))
            };
            self.idx += 1;
            Some(item)
        }
    }
}
