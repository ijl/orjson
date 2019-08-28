// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![feature(core_intrinsics)]

#[macro_use]
extern crate pyo3;

extern crate encoding_rs;
extern crate itoa;
extern crate serde;
extern crate serde_json;
extern crate smallvec;

use pyo3::prelude::*;
use pyo3::AsPyPointer;
use std::os::raw::c_char;
use std::ptr::NonNull;

mod decode;
mod encode;
mod exc;
mod typeref;

#[pymodule]
fn orjson(py: Python, m: &PyModule) -> PyResult<()> {
    typeref::init_typerefs();
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    m.add_wrapped(wrap_pyfunction!(dumps))?;

    // see pyo3 function_c_wrapper, impl_arg_params
    let wrapped_loads = pyo3::ffi::PyMethodDef {
        ml_name: "loads\0".as_ptr() as *const c_char,
        ml_meth: Some(loads),
        ml_flags: pyo3::ffi::METH_O,
        ml_doc: std::ptr::null(),
    };
    unsafe {
        pyo3::ffi::PyModule_AddObject(
            m.as_ptr(),
            "loads\0".as_ptr() as *const c_char,
            pyo3::ffi::PyCFunction_New(
                Box::into_raw(Box::new(wrapped_loads)),
                std::ptr::null_mut(),
            ),
        )
    };

    m.add("JSONDecodeError", py.get_type::<exc::JSONDecodeError>())?;
    m.add("JSONEncodeError", py.get_type::<exc::JSONEncodeError>())?;

    unsafe {
        pyo3::ffi::PyModule_AddObject(
            m.as_ptr(),
            "OPT_STRICT_INTEGER\0".as_ptr() as *const c_char,
            #[cfg(target_os = "windows")]
            pyo3::ffi::PyLong_FromUnsignedLongLong(encode::STRICT_INTEGER as u64),
            #[cfg(not(target_os = "windows"))]
            pyo3::ffi::PyLong_FromUnsignedLong(encode::STRICT_INTEGER as u64),
        )
    };
    unsafe {
        pyo3::ffi::PyModule_AddObject(
            m.as_ptr(),
            "OPT_NAIVE_UTC\0".as_ptr() as *const c_char,
            #[cfg(target_os = "windows")]
            pyo3::ffi::PyLong_FromUnsignedLongLong(encode::NAIVE_UTC as u64),
            #[cfg(not(target_os = "windows"))]
            pyo3::ffi::PyLong_FromUnsignedLong(encode::NAIVE_UTC as u64),
        )
    };
    Ok(())
}

/// loads(obj, /)
/// --
///
/// Deserialize JSON to Python objects.
pub unsafe extern "C" fn loads(
    _self: *mut pyo3::ffi::PyObject,
    obj: *mut pyo3::ffi::PyObject,
) -> *mut pyo3::ffi::PyObject {
    match decode::deserialize(obj) {
        Ok(val) => val.as_ptr(),
        Err(err) => {
            err.restore(pyo3::Python::assume_gil_acquired());
            std::ptr::null_mut()
        }
    }
}

/// dumps(obj, /, default, option)
/// --
///
/// Serialize Python objects to JSON.
#[pyfunction]
pub fn dumps(
    py: Python,
    obj: PyObject,
    default: Option<PyObject>,
    option: Option<PyObject>,
) -> PyResult<PyObject> {
    let pydef: Option<NonNull<pyo3::ffi::PyObject>>;
    if default.is_some() {
        pydef = Some(unsafe { NonNull::new_unchecked(default.unwrap().as_ptr()) });
    } else {
        pydef = None
    };
    let optsbits: i8;
    if option.is_some() {
        let optsptr = option.unwrap().as_ptr();
        if unsafe { (*optsptr).ob_type != typeref::INT_PTR } {
            return Err(exc::JSONEncodeError::py_err("Invalid opts"));
        } else {
            optsbits = unsafe { pyo3::ffi::PyLong_AsLong(optsptr) as i8 };
            if optsbits <= 0 || optsbits > encode::MAX_OPT {
                // -1
                return Err(exc::JSONEncodeError::py_err("Invalid opts"));
            }
        }
    } else {
        optsbits = 0
    };
    match encode::serialize(obj.as_ptr(), pydef, optsbits as u8) {
        Ok(val) => unsafe { Ok(PyObject::from_owned_ptr(py, val.as_ptr())) },
        Err(err) => Err(err),
    }
}
