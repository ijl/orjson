// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref;
use pyo3::prelude::*;
use pyo3::types::*;
use pyo3::IntoPyPointer;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;
use std::os::raw::c_char;

import_exception!(json, JSONDecodeError);

pub fn deserialize(py: Python, obj: PyObject) -> PyResult<PyObject> {
    let obj_ref = obj.as_ref(py);
    let obj_ptr = obj_ref.get_type_ptr();
    let data: Cow<str>;
    if unsafe { obj_ptr == typeref::STR_PTR } {
        data = unsafe {
            Cow::Borrowed(std::str::from_utf8_unchecked(
                <PyUnicode as PyTryFrom>::try_from_unchecked(obj_ref).as_bytes(),
            ))
        };
    } else if unsafe { obj_ptr == typeref::BYTES_PTR } {
        data = String::from_utf8_lossy(unsafe {
            <PyBytes as PyTryFrom>::try_from_unchecked(obj_ref).as_bytes()
        });
    } else {
        return Err(pyo3::exceptions::TypeError::py_err(format!(
            "Input must be str or bytes, not: {}",
            obj_ref.get_type().name()
        )));
    }

    let seed = JsonValue::new(py);
    let mut deserializer = serde_json::Deserializer::from_str(&data);
    match seed.deserialize(&mut deserializer) {
        Ok(py_ptr) => {
            deserializer
                .end()
                .map_err(|e| JSONDecodeError::py_err((e.to_string(), "", 0)))?;
            Ok(unsafe { PyObject::from_owned_ptr(py, py_ptr) })
        }
        Err(e) => {
            return Err(JSONDecodeError::py_err((e.to_string(), "", 0)));
        }
    }
}

#[derive(Clone, Copy)]
struct JsonValue<'a> {
    py: Python<'a>,
}

impl<'a> JsonValue<'a> {
    fn new(py: Python<'a>) -> JsonValue<'a> {
        JsonValue { py }
    }
}

impl<'de, 'a> DeserializeSeed<'de> for JsonValue<'a> {
    type Value = *mut pyo3::ffi::PyObject;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'a> Visitor<'de> for JsonValue<'a> {
    type Value = *mut pyo3::ffi::PyObject;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("JSON")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(unsafe { typeref::NONE })
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            true => unsafe {
                pyo3::ffi::Py_INCREF(typeref::TRUE);
                Ok(typeref::TRUE)
            },
            false => unsafe {
                pyo3::ffi::Py_INCREF(typeref::FALSE);
                Ok(typeref::FALSE)
            },
        }
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(unsafe { pyo3::ffi::PyLong_FromLong(value) })
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(unsafe { pyo3::ffi::PyLong_FromLong(value as i64) })
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(unsafe { pyo3::ffi::PyFloat_FromDouble(value) })
    }

    fn visit_borrowed_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(unsafe {
            pyo3::ffi::PyUnicode_FromStringAndSize(
                value.as_ptr() as *const c_char,
                value.len() as pyo3::ffi::Py_ssize_t,
            )
        })
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(unsafe {
            pyo3::ffi::PyUnicode_FromStringAndSize(
                value.as_ptr() as *const c_char,
                value.len() as pyo3::ffi::Py_ssize_t,
            )
        })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut elements: SmallVec<[*mut pyo3::ffi::PyObject; 8]> = SmallVec::new();
        while let Some(elem) = seq.next_element_seed(self)? {
            elements.push(elem);
        }
        let ptr = unsafe { pyo3::ffi::PyList_New(elements.len() as pyo3::ffi::Py_ssize_t) };
        for (i, obj) in elements.iter().enumerate() {
            unsafe { pyo3::ffi::PyList_SetItem(ptr, i as pyo3::ffi::Py_ssize_t, *obj) };
        }
        Ok(ptr)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let dict_ptr = PyDict::new(self.py).into_ptr();
        while let Some((key, value)) = map.next_entry_seed(PhantomData::<Cow<str>>, self)? {
            let _ = unsafe {
                pyo3::ffi::PyDict_SetItem(
                    dict_ptr,
                    pyo3::ffi::PyUnicode_FromStringAndSize(
                        key.as_ptr() as *const c_char,
                        key.len() as pyo3::ffi::Py_ssize_t,
                    ),
                    value,
                )
            };
        }
        Ok(dict_ptr)
    }
}
