// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::hint::black_box;
use serde::ser::{Serialize, Serializer};
use pyo3::prelude::*;
use rust_decimal::Decimal;
use criterion::{black_box};

#[repr(transparent)]
pub struct DecimalSerializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl DecimalSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        DecimalSerializer { ptr: ptr }
    }
}

impl Serialize for DecimalSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let d:Decimal=black_box(Self.ptr).extract();
        serializer.serialize_(d)
    }
}
