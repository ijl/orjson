// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![feature(custom_attribute)]

#[macro_use]
extern crate pyo3;

extern crate serde;
extern crate serde_json;
extern crate smallvec;

use pyo3::prelude::*;
use pyo3::types::*;

mod decode;
mod encode;
mod typeref;

#[pymodule]
fn orjson(py: Python, m: &PyModule) -> PyResult<()> {
    typeref::init_typerefs(py);
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_wrapped(wrap_function!(dumps))?;
    m.add_wrapped(wrap_function!(loads))?;
    m.add("JSONDecodeError", py.get_type::<decode::JSONDecodeError>())?;
    Ok(())
}

#[pyfunction]
pub fn loads(py: Python, obj: PyObject) -> PyResult<PyObject> {
    let obj_ref = obj.as_ref(py);
    let obj_ptr = obj_ref.get_type_ptr();
    let val: &[u8];
    if unsafe { obj_ptr == typeref::STR_PTR } {
        val = unsafe { <PyUnicode as PyTryFrom>::try_from_unchecked(obj_ref).as_bytes() };
    } else if unsafe { obj_ptr == typeref::BYTES_PTR } {
        val = unsafe { <PyBytes as PyTryFrom>::try_from_unchecked(obj_ref).as_bytes() };
    } else {
        return Err(pyo3::exceptions::TypeError::py_err(format!(
            "Input must be str or bytes, not: {}",
            obj_ref.get_type().name()
        )));
    }
    decode::deserialize(py, val)
}

#[pyfunction]
pub fn dumps(py: Python, obj: PyObject) -> PyResult<PyObject> {
    encode::serialize(py, obj)
}
