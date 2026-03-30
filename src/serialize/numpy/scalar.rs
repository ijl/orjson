// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2026)

use crate::ffi::PyObject;
use crate::opt::Opt;

pub(crate) struct NumpyScalar {
    pub ptr: *mut PyObject,
    pub opts: Opt,
}

impl NumpyScalar {
    pub const fn new(ptr: *mut PyObject, opts: Opt) -> Self {
        NumpyScalar { ptr, opts }
    }
}
