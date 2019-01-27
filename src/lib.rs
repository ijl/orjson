// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![feature(custom_attribute)]
#![feature(core_intrinsics)]

#[macro_use]
extern crate pyo3;

extern crate encoding_rs;
extern crate itoa;
extern crate serde;
extern crate serde_json;
extern crate smallvec;

use pyo3::prelude::*;
use pyo3::ToPyPointer;
use std::ptr::NonNull;

mod decode;
mod encode;
mod exc;
mod typeref;

#[pymodule]
fn orjson(py: Python, m: &PyModule) -> PyResult<()> {
    typeref::init_typerefs();
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_wrapped(wrap_function!(dumps))?;
    m.add_wrapped(wrap_function!(loads))?;
    m.add("JSONDecodeError", py.get_type::<exc::JSONDecodeError>())?;
    m.add("JSONEncodeError", py.get_type::<exc::JSONEncodeError>())?;
    m.add("OPT_STRICT_INTEGER", encode::STRICT_INTEGER.into_object(py))?;
    m.add("OPT_NAIVE_UTC", encode::NAIVE_UTC.into_object(py))?;
    Ok(())
}

/// loads(obj, /)
/// --
///
/// Deserialize JSON to Python objects.
#[pyfunction]
pub fn loads(py: Python, obj: PyObject) -> PyResult<PyObject> {
    decode::deserialize(py, obj.as_ptr())
}

/// dumps(obj, default, option, /)
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
    encode::serialize(py, obj.as_ptr(), pydef, optsbits as u8)
}
