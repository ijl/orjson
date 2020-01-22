// SPDX-License-Identifier: (Apache-2.0 OR MIT)

macro_rules! is_type {
    ($obj_ptr:ident, $type_ptr:ident) => {
        unsafe { $obj_ptr == $type_ptr }
    };
}

macro_rules! unlikely {
    ($exp:expr) => {
        unsafe { std::intrinsics::unlikely($exp) }
    };
}

macro_rules! str_from_slice {
    ($ptr:expr, $size:expr) => {
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts($ptr, $size as usize)) }
    };
}

macro_rules! str_to_pyobject {
    ($obj:expr) => {
        unsafe {
            pyo3::ffi::PyUnicode_DecodeUTF8Stateful(
                $obj.as_ptr() as *const c_char,
                $obj.len() as pyo3::ffi::Py_ssize_t,
                "ignore\0".as_ptr() as *const c_char,
                std::ptr::null_mut(),
            )
        };
    };
}

macro_rules! ffi {
    ($fn:ident()) => {
        unsafe { pyo3::ffi::$fn() }
    };

    ($fn:ident($obj1:expr)) => {
        unsafe { pyo3::ffi::$fn($obj1) }
    };

    ($fn:ident($obj1:expr, $obj2:expr)) => {
        unsafe { pyo3::ffi::$fn($obj1, $obj2) }
    };

    ($fn:ident($obj1:expr, $obj2:expr, $obj3:expr)) => {
        unsafe { pyo3::ffi::$fn($obj1, $obj2, $obj3) }
    };

    ($fn:ident($obj1:expr, $obj2:expr, $obj3:expr, $obj4:expr)) => {
        unsafe { pyo3::ffi::$fn($obj1, $obj2, $obj3, $obj4) }
    };
}
