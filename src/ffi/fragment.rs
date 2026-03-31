// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2020-2026)

use crate::ffi::{
    Py_DECREF, Py_INCREF, Py_TPFLAGS_DEFAULT, Py_TPFLAGS_IMMUTABLETYPE, Py_tp_dealloc, Py_tp_new,
    PyErr_SetObject, PyExc_TypeError, PyObject, PyTupleRef, PyType_FromSpec, PyType_Slot,
    PyType_Spec, PyTypeObject, PyUnicode_FromStringAndSize,
};
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;

#[cfg(Py_GIL_DISABLED)]
use core::sync::atomic::{AtomicIsize, AtomicU32};

#[cfg(Py_GIL_DISABLED)]
macro_rules! pymutex_new {
    () => {
        unsafe { core::mem::zeroed() }
    };
}

#[repr(C)]
pub(crate) struct Fragment {
    #[cfg(Py_GIL_DISABLED)]
    pub ob_tid: usize,
    #[cfg(all(Py_GIL_DISABLED, Py_3_14))]
    pub ob_flags: u16,
    #[cfg(all(Py_GIL_DISABLED, not(Py_3_14)))]
    pub _padding: u16,
    #[cfg(Py_GIL_DISABLED)]
    pub ob_mutex: pyo3_ffi::PyMutex,
    #[cfg(Py_GIL_DISABLED)]
    pub ob_gc_bits: u8,
    #[cfg(Py_GIL_DISABLED)]
    pub ob_ref_local: AtomicU32,
    #[cfg(Py_GIL_DISABLED)]
    pub ob_ref_shared: AtomicIsize,
    #[cfg(not(Py_GIL_DISABLED))]
    pub ob_refcnt: pyo3_ffi::Py_ssize_t,
    #[cfg(PyPy)]
    pub ob_pypy_link: pyo3_ffi::Py_ssize_t,
    pub ob_type: *mut pyo3_ffi::PyTypeObject,
    pub contents: *mut pyo3_ffi::PyObject,
}

#[cold]
#[inline(never)]
#[cfg_attr(feature = "optimize", optimize(size))]
fn raise_args_exception() {
    unsafe {
        let msg = "orjson.Fragment() takes exactly 1 positional argument";
        let err_msg =
            PyUnicode_FromStringAndSize(msg.as_ptr().cast::<c_char>(), msg.len().cast_signed());
        PyErr_SetObject(PyExc_TypeError, err_msg);
        Py_DECREF(err_msg);
    };
}

#[unsafe(no_mangle)]
#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
pub(crate) unsafe extern "C" fn orjson_fragment_tp_new(
    _subtype: *mut PyTypeObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject {
    unsafe {
        let argsob = PyTupleRef::from_ptr_unchecked(args);
        if argsob.len() != 1 || !kwds.is_null() {
            raise_args_exception();
            null_mut()
        } else {
            let contents = argsob.get(0);
            Py_INCREF(contents);
            let obj = Box::new(Fragment {
                #[cfg(Py_GIL_DISABLED)]
                ob_tid: 0,
                #[cfg(all(Py_GIL_DISABLED, Py_3_14))]
                ob_flags: 0,
                #[cfg(all(Py_GIL_DISABLED, not(Py_3_14)))]
                _padding: 0,
                #[cfg(Py_GIL_DISABLED)]
                ob_mutex: pymutex_new!(),
                #[cfg(Py_GIL_DISABLED)]
                ob_gc_bits: 0,
                #[cfg(Py_GIL_DISABLED)]
                ob_ref_local: AtomicU32::new(0),
                #[cfg(Py_GIL_DISABLED)]
                ob_ref_shared: AtomicIsize::new(0),
                #[cfg(not(Py_GIL_DISABLED))]
                ob_refcnt: 1,
                #[cfg(PyPy)]
                ob_pypy_link: 0,
                ob_type: crate::typeref::FRAGMENT_TYPE,
                contents: contents,
            });
            Box::into_raw(obj).cast::<PyObject>()
        }
    }
}

#[unsafe(no_mangle)]
#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
pub(crate) unsafe extern "C" fn orjson_fragment_dealloc(object: *mut PyObject) {
    unsafe {
        Py_DECREF((*object.cast::<Fragment>()).contents);
        crate::ffi::PyMem_Free(object.cast::<core::ffi::c_void>());
    }
}

#[unsafe(no_mangle)]
#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
pub(crate) unsafe extern "C" fn orjson_fragmenttype_new() -> *mut PyTypeObject {
    unsafe {
        let mut slots = [
            PyType_Slot {
                slot: Py_tp_new,
                pfunc: orjson_fragment_tp_new as *mut c_void,
            },
            PyType_Slot {
                slot: Py_tp_dealloc,
                pfunc: orjson_fragment_dealloc as *mut c_void,
            },
            PyType_Slot {
                slot: 0,
                pfunc: null_mut(),
            },
        ];
        let mut spec = PyType_Spec {
            name: c"orjson.Fragment".as_ptr(),
            basicsize: core::mem::size_of::<Fragment>().cast_signed() as i32,
            itemsize: 0,
            flags: (Py_TPFLAGS_DEFAULT | Py_TPFLAGS_IMMUTABLETYPE) as u32,
            slots: &raw mut slots[0],
        };
        PyType_FromSpec(&raw mut spec).cast::<PyTypeObject>()
    }
}
