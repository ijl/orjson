// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![feature(custom_attribute)]

#[macro_use]
extern crate pyo3;

use pyo3::prelude::*;

#[pymodule]
fn orjson(_: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
