// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::ptr::NonNull;

pub struct PyListIter {
    obj: *mut pyo3_ffi::PyListObject,
    len: usize,
    pos: usize,
}

impl PyListIter {
    #[inline]
    pub fn from_pyobject(obj: *mut pyo3_ffi::PyObject) -> Self {
        unsafe {
            PyListIter {
                obj: obj as *mut pyo3_ffi::PyListObject,
                len: ffi!(Py_SIZE(obj)) as usize,
                pos: 0,
            }
        }
    }
}

impl Iterator for PyListIter {
    type Item = NonNull<pyo3_ffi::PyObject>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.len {
            None
        } else {
            let elem = unsafe { *((*self.obj).ob_item).add(self.pos) };
            self.pos += 1;
            Some(nonnull!(elem))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

#[cfg(feature = "trusted_len")]
unsafe impl std::iter::TrustedLen for PyListIter {}
