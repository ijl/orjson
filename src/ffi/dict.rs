// SPDX-License-Identifier: (Apache-2.0 OR MIT)

pub struct PyDictIter {
    dict_ptr: *mut pyo3_ffi::PyObject,
    pos: isize,
}

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
