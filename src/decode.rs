// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::exc::*;
use crate::typeref;
use pyo3::prelude::*;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;
use std::os::raw::c_char;

pub fn deserialize(py: Python, ptr: *mut pyo3::ffi::PyObject) -> PyResult<PyObject> {
    let obj_type_ptr = unsafe { (*ptr).ob_type };
    let data: Cow<str>;
    if unsafe { obj_type_ptr == typeref::STR_PTR } {
        let mut str_size: pyo3::ffi::Py_ssize_t = unsafe { std::mem::uninitialized() };
        let uni = unsafe { pyo3::ffi::PyUnicode_AsUTF8AndSize(ptr, &mut str_size) as *const u8 };
        if unsafe { std::intrinsics::unlikely(uni.is_null()) } {
            return Err(JSONDecodeError::py_err((INVALID_STR, "", 0)));
        }
        data = unsafe {
            Cow::Borrowed(std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                uni,
                str_size as usize,
            )))
        };
    } else if unsafe { obj_type_ptr == typeref::BYTES_PTR } {
        let buffer = unsafe { pyo3::ffi::PyBytes_AsString(ptr) as *const u8 };
        let length = unsafe { pyo3::ffi::PyBytes_Size(ptr) as usize };
        let slice = unsafe { std::slice::from_raw_parts(buffer, length) };
        if encoding_rs::Encoding::utf8_valid_up_to(slice) == length {
            data = Cow::Borrowed(unsafe { std::str::from_utf8_unchecked(slice) });
        } else {
            return Err(JSONDecodeError::py_err((INVALID_STR, "", 0)));
        }
    } else {
        return Err(JSONDecodeError::py_err((
            "Input must be str or bytes",
            "",
            0,
        )));
    }

    let seed = JsonValue {};
    let mut deserializer = serde_json::Deserializer::from_str(&data);
    match seed.deserialize(&mut deserializer) {
        Ok(py_ptr) => {
            deserializer
                .end()
                .map_err(|e| JSONDecodeError::py_err((e.to_string(), "", 0)))?;
            Ok(unsafe { PyObject::from_owned_ptr(py, py_ptr) })
        }
        Err(e) => Err(JSONDecodeError::py_err((e.to_string(), "", 0))),
    }
}

#[derive(Clone, Copy)]
struct JsonValue;

impl<'de, 'a> DeserializeSeed<'de> for JsonValue {
    type Value = *mut pyo3::ffi::PyObject;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'a> Visitor<'de> for JsonValue {
    type Value = *mut pyo3::ffi::PyObject;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("JSON")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        unsafe { pyo3::ffi::Py_INCREF(typeref::NONE) };
        Ok(unsafe { typeref::NONE })
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value {
            unsafe {
                pyo3::ffi::Py_INCREF(typeref::TRUE);
                Ok(typeref::TRUE)
            }
        } else {
            unsafe {
                pyo3::ffi::Py_INCREF(typeref::FALSE);
                Ok(typeref::FALSE)
            }
        }
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(unsafe { pyo3::ffi::PyLong_FromLongLong(value) })
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(unsafe { pyo3::ffi::PyLong_FromLongLong(value as i64) })
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
        let mut elements: SmallVec<[*mut pyo3::ffi::PyObject; 6]> = SmallVec::new();
        while let Some(elem) = seq.next_element_seed(self)? {
            elements.push(elem);
        }
        let ptr = unsafe { pyo3::ffi::PyList_New(elements.len() as pyo3::ffi::Py_ssize_t) };
        for (i, obj) in elements.iter().enumerate() {
            unsafe { pyo3::ffi::PyList_SET_ITEM(ptr, i as pyo3::ffi::Py_ssize_t, *obj) };
        }
        Ok(ptr)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let dict_ptr = unsafe { pyo3::ffi::PyDict_New() };
        while let Some((key, value)) = map.next_entry_seed(PhantomData::<Cow<str>>, self)? {
            let pykey = unsafe {
                pyo3::ffi::PyUnicode_FromStringAndSize(
                    key.as_ptr() as *const c_char,
                    key.len() as pyo3::ffi::Py_ssize_t,
                )
            };
            let _ = unsafe { pyo3::ffi::PyDict_SetItem(dict_ptr, pykey, value) };
            // counter Py_INCREF in insertdict
            unsafe {
                pyo3::ffi::Py_DECREF(pykey);
                pyo3::ffi::Py_DECREF(value);
            };
        }
        Ok(dict_ptr)
    }
}
