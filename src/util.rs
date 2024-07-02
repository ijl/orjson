// SPDX-License-Identifier: (Apache-2.0 OR MIT)

pub const INVALID_STR: &str = "str is not valid UTF-8: surrogates not allowed";

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

macro_rules! is_class_by_type {
    ($ob_type:expr, $type_ptr:ident) => {
        unsafe { $ob_type == $type_ptr }
    };
}

macro_rules! is_subclass_by_flag {
    ($ob_type:expr, $flag:ident) => {
        unsafe { (((*$ob_type).tp_flags & pyo3_ffi::$flag) != 0) }
    };
}

macro_rules! is_subclass_by_type {
    ($ob_type:expr, $type:ident) => {
        unsafe {
            (*($ob_type as *mut pyo3_ffi::PyTypeObject))
                .ob_base
                .ob_base
                .ob_type
                == $type
        }
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
        core::intrinsics::unlikely($exp)
    };
}

#[cfg(not(feature = "intrinsics"))]
macro_rules! unlikely {
    ($exp:expr) => {
        $exp
    };
}

#[allow(unused_macros)]
#[cfg(feature = "intrinsics")]
macro_rules! likely {
    ($exp:expr) => {
        core::intrinsics::likely($exp)
    };
}

#[allow(unused_macros)]
#[cfg(not(feature = "intrinsics"))]
macro_rules! likely {
    ($exp:expr) => {
        $exp
    };
}

macro_rules! nonnull {
    ($exp:expr) => {
        unsafe { core::ptr::NonNull::new_unchecked($exp) }
    };
}

macro_rules! str_from_slice {
    ($ptr:expr, $size:expr) => {
        unsafe { std::str::from_utf8_unchecked(core::slice::from_raw_parts($ptr, $size as usize)) }
    };
}

#[cfg(Py_3_12)]
macro_rules! reverse_pydict_incref {
    ($op:expr) => {
        unsafe {
            if pyo3_ffi::_Py_IsImmortal($op) == 0 {
                debug_assert!(ffi!(Py_REFCNT($op)) >= 2);
                (*$op).ob_refcnt.ob_refcnt -= 1;
            }
        }
    };
}

#[cfg(not(Py_3_12))]
macro_rules! reverse_pydict_incref {
    ($op:expr) => {
        unsafe {
            debug_assert!(ffi!(Py_REFCNT($op)) >= 2);
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
                core::ptr::null_mut() as *mut pyo3_ffi::PyObject,
            )
        }
    };
    ($obj1:expr, $obj2:expr, $obj3:expr) => {
        unsafe {
            pyo3_ffi::PyObject_CallMethodObjArgs(
                $obj1,
                $obj2,
                $obj3,
                core::ptr::null_mut() as *mut pyo3_ffi::PyObject,
            )
        }
    };
}

macro_rules! str_hash {
    ($op:expr) => {
        unsafe { (*$op.cast::<pyo3_ffi::PyASCIIObject>()).hash }
    };
}

#[cfg(Py_3_13)]
macro_rules! pydict_contains {
    ($obj1:expr, $obj2:expr) => {
        unsafe { pyo3_ffi::PyDict_Contains(pyo3_ffi::PyType_GetDict($obj1), $obj2) == 1 }
    };
}

#[cfg(all(Py_3_12, not(Py_3_13)))]
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

#[cfg(not(Py_3_13))]
macro_rules! pydict_next {
    ($obj1:expr, $obj2:expr, $obj3:expr, $obj4:expr) => {
        unsafe { pyo3_ffi::_PyDict_Next($obj1, $obj2, $obj3, $obj4, core::ptr::null_mut()) }
    };
}

#[cfg(Py_3_13)]
macro_rules! pydict_next {
    ($obj1:expr, $obj2:expr, $obj3:expr, $obj4:expr) => {
        unsafe { pyo3_ffi::PyDict_Next($obj1, $obj2, $obj3, $obj4) }
    };
}

macro_rules! reserve_minimum {
    ($writer:expr) => {
        $writer.reserve(64);
    };
}

macro_rules! reserve_pretty {
    ($writer:expr, $val:expr) => {
        $writer.reserve($val + 16);
    };
}

macro_rules! assume {
    ($expr:expr) => {
        debug_assert!($expr);
        #[cfg(feature = "intrinsics")]
        unsafe {
            core::intrinsics::assume($expr);
        };
    };
}

#[allow(unused_macros)]
#[cfg(feature = "intrinsics")]
macro_rules! trailing_zeros {
    ($val:expr) => {
        core::intrinsics::cttz_nonzero($val) as usize
    };
}

#[allow(unused_macros)]
#[cfg(not(feature = "intrinsics"))]
macro_rules! trailing_zeros {
    ($val:expr) => {
        $val.trailing_zeros() as usize
    };
}

#[allow(unused_macros)]
#[cfg(feature = "intrinsics")]
macro_rules! popcnt {
    ($val:expr) => {
        core::intrinsics::ctpop(core::mem::transmute::<u32, i32>($val)) as usize
    };
}

#[allow(unused_macros)]
#[cfg(not(feature = "intrinsics"))]
macro_rules! popcnt {
    ($val:expr) => {
        core::mem::transmute::<u32, i32>($val).count_ones() as usize
    };
}
