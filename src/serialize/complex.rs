// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use serde::ser::{Serialize, SerializeSeq, Serializer};

#[repr(transparent)]
pub struct ComplexSerializer {
    ptr: *mut pyo3_ffi::PyObject,
}

impl ComplexSerializer {
    pub fn new(ptr: *mut pyo3_ffi::PyObject) -> Self {
        ComplexSerializer { ptr: ptr }
    }
}

impl Serialize for ComplexSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        let real = ffi!(PyComplex_RealAsDouble(self.ptr));
        let imag = ffi!(PyComplex_ImagAsDouble(self.ptr));
        seq.serialize_element(&real)?;
        seq.serialize_element(&imag)?;
        seq.end()
    }
}
