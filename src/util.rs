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
        return Err(serde::ser::Error::custom($msg))
    };
}

macro_rules! opt_enabled {
    ($var:expr, $flag:expr) => {
        $var & $flag != 0
    };
}

macro_rules! opt_disabled {
    ($var:expr, $flag:expr) => {
        $var & $flag == 0
    };
}

#[cfg(feature = "intrinsics")]
macro_rules! unlikely {
    ($exp:expr) => {
        std::intrinsics::unlikely($exp)
    };
}

#[cfg(not(feature = "intrinsics"))]
macro_rules! unlikely {
    ($exp:expr) => {
        $exp
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

#[cfg(Py_3_12)]
macro_rules! py_decref_without_destroy {
    ($op:expr) => {
        unsafe {
            (*$op).ob_refcnt.ob_refcnt -= 1;
        }
    };
}

#[cfg(not(Py_3_12))]
macro_rules! py_decref_without_destroy {
    ($op:expr) => {
        unsafe {
            (*$op).ob_refcnt -= 1;
        }
    };
}

macro_rules! ffi {
    ($fn:ident()) => {
        unsafe { pyo3_ffi::$fn() }
    };

    ($fn:ident($obj1:expr)) => {
        unsafe { pyo3_ffi::$fn($obj1) }
    };

    ($fn:ident($obj1:expr, $obj2:expr)) => {
        unsafe { pyo3_ffi::$fn($obj1, $obj2) }
    };

    ($fn:ident($obj1:expr, $obj2:expr, $obj3:expr)) => {
        unsafe { pyo3_ffi::$fn($obj1, $obj2, $obj3) }
    };

    ($fn:ident($obj1:expr, $obj2:expr, $obj3:expr, $obj4:expr)) => {
        unsafe { pyo3_ffi::$fn($obj1, $obj2, $obj3, $obj4) }
    };
}

#[cfg(Py_3_9)]
macro_rules! call_method {
    ($obj1:expr, $obj2:expr) => {
        unsafe { pyo3_ffi::PyObject_CallMethodNoArgs($obj1, $obj2) }
    };
    ($obj1:expr, $obj2:expr, $obj3:expr) => {
        unsafe { pyo3_ffi::PyObject_CallMethodOneArg($obj1, $obj2, $obj3) }
    };
}

#[cfg(not(Py_3_9))]
macro_rules! call_method {
    ($obj1:expr, $obj2:expr) => {
        unsafe {
            pyo3_ffi::PyObject_CallMethodObjArgs(
                $obj1,
                $obj2,
                std::ptr::null_mut() as *mut pyo3_ffi::PyObject,
            )
        }
    };
    ($obj1:expr, $obj2:expr, $obj3:expr) => {
        unsafe {
            pyo3_ffi::PyObject_CallMethodObjArgs(
                $obj1,
                $obj2,
                $obj3,
                std::ptr::null_mut() as *mut pyo3_ffi::PyObject,
            )
        }
    };
}

macro_rules! str_hash {
    ($op:expr) => {
        unsafe { (*$op.cast::<pyo3_ffi::PyASCIIObject>()).hash }
    };
}

#[cfg(Py_3_12)]
macro_rules! pydict_contains {
    ($obj1:expr, $obj2:expr) => {
        unsafe {
            pyo3_ffi::_PyDict_Contains_KnownHash(
                pyo3_ffi::PyType_GetDict($obj1),
                $obj2,
                (*$obj2.cast::<pyo3_ffi::PyASCIIObject>()).hash,
            ) == 1
        }
    };
}

#[cfg(all(Py_3_10, not(Py_3_12)))]
macro_rules! pydict_contains {
    ($obj1:expr, $obj2:expr) => {
        unsafe {
            pyo3_ffi::_PyDict_Contains_KnownHash(
                (*$obj1).tp_dict,
                $obj2,
                (*$obj2.cast::<pyo3_ffi::PyASCIIObject>()).hash,
            ) == 1
        }
    };
}

#[cfg(not(Py_3_10))]
macro_rules! pydict_contains {
    ($obj1:expr, $obj2:expr) => {
        unsafe { pyo3_ffi::PyDict_Contains((*$obj1).tp_dict, $obj2) == 1 }
    };
}

#[cfg(Py_3_12)]
macro_rules! use_immortal {
    ($op:expr) => {
        unsafe { $op }
    };
}

#[cfg(not(Py_3_12))]
macro_rules! use_immortal {
    ($op:expr) => {
        unsafe {
            ffi!(Py_INCREF($op));
            $op
        }
    };
}
