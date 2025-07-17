// SPDX-License-Identifier: (Apache-2.0 OR MIT)

pub(crate) const INVALID_STR: &str = "str is not valid UTF-8: surrogates not allowed";

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

#[cfg(not(Py_GIL_DISABLED))]
macro_rules! tp_flags {
    ($ob_type:expr) => {
        unsafe { (*$ob_type).tp_flags }
    };
}

#[cfg(Py_GIL_DISABLED)]
macro_rules! tp_flags {
    ($ob_type:expr) => {
        unsafe {
            (*$ob_type)
                .tp_flags
                .load(std::sync::atomic::Ordering::Relaxed)
        }
    };
}

macro_rules! is_subclass_by_flag {
    ($tp_flags:expr, $flag:ident) => {
        unsafe { (($tp_flags & pyo3_ffi::$flag) != 0) }
    };
}

macro_rules! is_subclass_by_type {
    ($ob_type:expr, $type:ident) => {
        unsafe {
            (*($ob_type.cast::<pyo3_ffi::PyTypeObject>()))
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
        unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts($ptr, $size as usize)) }
    };
}

#[cfg(all(Py_3_12, not(Py_GIL_DISABLED)))]
macro_rules! reverse_pydict_incref {
    ($op:expr) => {
        unsafe {
            if crate::ffi::_Py_IsImmortal($op) == 0 {
                debug_assert!(ffi!(Py_REFCNT($op)) >= 2);
                (*$op).ob_refcnt.ob_refcnt -= 1;
            }
        }
    };
}

#[cfg(Py_GIL_DISABLED)]
macro_rules! reverse_pydict_incref {
    ($op:expr) => {
        debug_assert!(ffi!(Py_REFCNT($op)) >= 2);
        ffi!(Py_DECREF($op))
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

macro_rules! call_method {
    ($obj1:expr, $obj2:expr) => {
        unsafe { pyo3_ffi::PyObject_CallMethodNoArgs($obj1, $obj2) }
    };
    ($obj1:expr, $obj2:expr, $obj3:expr) => {
        unsafe { pyo3_ffi::PyObject_CallMethodOneArg($obj1, $obj2, $obj3) }
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
            crate::ffi::_PyDict_Contains_KnownHash(
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
            crate::ffi::_PyDict_Contains_KnownHash(
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
        unsafe { crate::ffi::_PyDict_Next($obj1, $obj2, $obj3, $obj4, core::ptr::null_mut()) }
    };
}

#[cfg(Py_3_13)]
macro_rules! pydict_next {
    ($obj1:expr, $obj2:expr, $obj3:expr, $obj4:expr) => {
        unsafe { pyo3_ffi::PyDict_Next($obj1, $obj2, $obj3, $obj4) }
    };
}

macro_rules! pydict_setitem {
    ($dict:expr, $pykey:expr, $pyval:expr) => {
        debug_assert!(ffi!(Py_REFCNT($dict)) == 1);
        debug_assert!(str_hash!($pykey) != -1);
        #[cfg(not(Py_3_13))]
        unsafe {
            let _ = crate::ffi::_PyDict_SetItem_KnownHash($dict, $pykey, $pyval, str_hash!($pykey));
        }
        #[cfg(Py_3_13)]
        unsafe {
            let _ = crate::ffi::_PyDict_SetItem_KnownHash_LockHeld(
                $dict.cast::<pyo3_ffi::PyDictObject>(),
                $pykey,
                $pyval,
                str_hash!($pykey),
            );
        }
        #[cfg(not(Py_GIL_DISABLED))]
        reverse_pydict_incref!($pykey);
        reverse_pydict_incref!($pyval);
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
        unsafe {
            core::hint::assert_unchecked($expr);
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

macro_rules! unreachable_unchecked {
    () => {
        unsafe { core::hint::unreachable_unchecked() }
    };
}

#[inline(always)]
#[allow(clippy::cast_possible_wrap)]
pub(crate) fn usize_to_isize(val: usize) -> isize {
    debug_assert!(val < (isize::MAX as usize));
    val as isize
}

#[inline(always)]
#[allow(clippy::cast_sign_loss)]
pub(crate) fn isize_to_usize(val: isize) -> usize {
    debug_assert!(val >= 0);
    val as usize
}
