// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![feature(custom_attribute)]

#[macro_use]
extern crate pyo3;

extern crate serde;
extern crate serde_json;
extern crate smallvec;

use pyo3::prelude::*;
use pyo3::types::*;

mod encode;
mod decode;

#[pymodule]
fn orjson(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_wrapped(wrap_function!(dumps))?;
    m.add_wrapped(wrap_function!(loads))?;
    m.add("JSONDecodeError", py.get_type::<decode::JSONDecodeError>())?;
    Ok(())
}

#[pyfunction]
pub fn loads(py: Python, obj: PyObject) -> PyResult<PyObject> {
    let obj_ref = obj.as_ref(py);
    if let Ok(val) = <PyUnicode as PyTryFrom>::try_from(obj_ref) {
        decode::deserialize(py, val.as_bytes())
    } else if let Ok(val) = <PyBytes as PyTryFrom>::try_from(obj_ref) {
        decode::deserialize(py, val.as_bytes())
    }
    else {
        return Err(
            pyo3::exceptions::TypeError::py_err(
                format!("Input must be unicode or bytes, not: {}", obj_ref.get_type().name())
            )
        );
    }
}

#[pyfunction]
pub fn dumps(py: Python, obj: PyObject) -> PyResult<PyObject> {
    encode::serialize(py, obj)
}
