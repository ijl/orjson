// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![feature(custom_attribute)]
#![feature(core_intrinsics)]

#[macro_use]
extern crate pyo3;

extern crate serde;
extern crate serde_json;
extern crate smallvec;

use pyo3::prelude::*;
use pyo3::ToPyPointer;

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

/// dumps(obj, /)
/// --
///
/// Serialize Python objects to JSON.
#[pyfunction]
pub fn dumps(py: Python, obj: PyObject) -> PyResult<PyObject> {
    encode::serialize(py, obj.as_ptr())
}
