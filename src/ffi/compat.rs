// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[cfg(Py_GIL_DISABLED)]
#[allow(non_upper_case_globals)]
pub(crate) const _Py_IMMORTAL_REFCNT_LOCAL: u32 = u32::MAX;

#[cfg(all(Py_3_14, target_pointer_width = "32"))]
pub(crate) const _Py_IMMORTAL_MINIMUM_REFCNT: pyo3_ffi::Py_ssize_t =
    ((1 as core::ffi::c_long) << (30 as core::ffi::c_long)) as pyo3_ffi::Py_ssize_t;

#[cfg(all(Py_3_12, not(Py_3_14)))]
#[allow(non_upper_case_globals)]
pub(crate) const _Py_IMMORTAL_REFCNT: pyo3_ffi::Py_ssize_t = {
    if cfg!(target_pointer_width = "64") {
        core::ffi::c_uint::MAX as pyo3_ffi::Py_ssize_t
    } else {
        // for 32-bit systems, use the lower 30 bits (see comment in CPython's object.h)
        (core::ffi::c_uint::MAX >> 2) as pyo3_ffi::Py_ssize_t
    }
};
#[cfg(all(Py_3_12, not(Py_GIL_DISABLED)))]
#[inline(always)]
#[allow(non_snake_case)]
pub(crate) unsafe fn _Py_IsImmortal(op: *mut pyo3_ffi::PyObject) -> core::ffi::c_int {
    #[cfg(all(target_pointer_width = "64", not(Py_GIL_DISABLED)))]
    {
        (((*op).ob_refcnt.ob_refcnt as pyo3_ffi::PY_INT32_T) < 0) as core::ffi::c_int
    }

    #[cfg(all(target_pointer_width = "32", not(Py_GIL_DISABLED)))]
    {
        #[cfg(not(Py_3_14))]
        {
            ((*op).ob_refcnt.ob_refcnt == _Py_IMMORTAL_REFCNT) as core::ffi::c_int
        }

        #[cfg(Py_3_14)]
        {
            ((*op).ob_refcnt.ob_refcnt >= _Py_IMMORTAL_MINIMUM_REFCNT) as core::ffi::c_int
        }
    }

    #[cfg(Py_GIL_DISABLED)]
    {
        ((*op)
            .ob_ref_local
            .load(std::sync::atomic::Ordering::Relaxed)
            == _Py_IMMORTAL_REFCNT_LOCAL) as core::ffi::c_int
    }
}

unsafe extern "C" {
    pub fn _PyDict_Next(
        mp: *mut pyo3_ffi::PyObject,
        pos: *mut pyo3_ffi::Py_ssize_t,
        key: *mut *mut pyo3_ffi::PyObject,
        value: *mut *mut pyo3_ffi::PyObject,
        hash: *mut pyo3_ffi::Py_hash_t,
    ) -> core::ffi::c_int;

    #[cfg(Py_3_10)]
    pub fn _PyDict_Contains_KnownHash(
        op: *mut pyo3_ffi::PyObject,
        key: *mut pyo3_ffi::PyObject,
        hash: pyo3_ffi::Py_hash_t,
    ) -> core::ffi::c_int;

    #[cfg(not(Py_3_13))]
    pub(crate) fn _PyDict_SetItem_KnownHash(
        mp: *mut pyo3_ffi::PyObject,
        name: *mut pyo3_ffi::PyObject,
        value: *mut pyo3_ffi::PyObject,
        hash: pyo3_ffi::Py_hash_t,
    ) -> core::ffi::c_int;

    #[cfg(Py_3_13)]
    pub(crate) fn _PyDict_SetItem_KnownHash_LockHeld(
        mp: *mut pyo3_ffi::PyDictObject,
        name: *mut pyo3_ffi::PyObject,
        value: *mut pyo3_ffi::PyObject,
        hash: pyo3_ffi::Py_hash_t,
    ) -> core::ffi::c_int;

    #[cfg(Py_3_13)]
    pub(crate) fn _PyLong_AsByteArray(
        v: *mut pyo3_ffi::PyLongObject,
        bytes: *mut core::ffi::c_uchar,
        n: pyo3_ffi::Py_ssize_t,
        little_endian: core::ffi::c_int,
        is_signed: core::ffi::c_int,
        with_exceptions: core::ffi::c_int,
    ) -> core::ffi::c_int;

    #[cfg(not(Py_3_13))]
    pub(crate) fn _PyLong_AsByteArray(
        v: *mut pyo3_ffi::PyLongObject,
        bytes: *mut core::ffi::c_uchar,
        n: pyo3_ffi::Py_ssize_t,
        little_endian: core::ffi::c_int,
        is_signed: core::ffi::c_int,
    ) -> core::ffi::c_int;
}
