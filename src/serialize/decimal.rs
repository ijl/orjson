// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use serde::ser::{Serialize, Serializer};
use core::{str::FromStr};
use crate::str::*;

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
        let uni = unicode_to_str(ffi!(PyObject_Str(self.ptr)));
        println!("uni: {uni:?}");
        let json_number_result =serde_json::Number::from_str(uni.unwrap());
        // let json_number=json_number_result.unwrap();
        // println!("json_number: {json_number:?}");
        json_number_result
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

