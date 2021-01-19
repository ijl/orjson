// SPDX-License-Identifier: (Apache-2.0 OR MIT)

macro_rules! is_type {
    ($obj_ptr:expr, $type_ptr:expr) => {
        unsafe { $obj_ptr == $type_ptr }
    };
}

macro_rules! ob_type {
    ($obj:expr) => {
        unsafe { (*$obj).ob_type }
    };
}

macro_rules! err {
    ($msg:expr) => {
        return Err(serde::ser::Error::custom($msg));
    };
}

macro_rules! unlikely {
    ($exp:expr) => {
        std::intrinsics::unlikely($exp)
    };
}

macro_rules! likely {
    ($exp:expr) => {
        std::intrinsics::likely($exp)
    };
}

macro_rules! nonnull {
    ($exp:expr) => {
        unsafe { std::ptr::NonNull::new_unchecked($exp) }
    };
}

macro_rules! str_from_slice {
    ($ptr:expr, $size:expr) => {
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts($ptr, $size as usize)) }
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

#[cfg(python39)]
macro_rules! call_method {
    ($obj1:expr, $obj2:expr) => {
        unsafe { pyo3::ffi::PyObject_CallMethodNoArgs($obj1, $obj2) }
    };
    ($obj1:expr, $obj2:expr, $obj3:expr) => {
        unsafe { pyo3::ffi::PyObject_CallMethodOneArg($obj1, $obj2, $obj3) }
    };
}

#[cfg(not(python39))]
macro_rules! call_method {
    ($obj1:expr, $obj2:expr) => {
        unsafe {
            pyo3::ffi::PyObject_CallMethodObjArgs(
                $obj1,
                $obj2,
                std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
            )
        }
    };
    ($obj1:expr, $obj2:expr, $obj3:expr) => {
        unsafe {
            pyo3::ffi::PyObject_CallMethodObjArgs(
                $obj1,
                $obj2,
                $obj3,
                std::ptr::null_mut() as *mut pyo3::ffi::PyObject,
            )
        }
    };
}
