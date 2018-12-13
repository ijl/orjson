// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref::*;
use pyo3::prelude::*;
use pyo3::types::*;
use serde::ser::{self, Serialize, SerializeMap, SerializeSeq, Serializer};

pub fn serialize(py: Python, obj: PyObject) -> PyResult<PyObject> {
    let s: Result<Vec<u8>, JsonError> = serde_json::to_vec(&SerializePyObject {
        py: py,
        obj: obj.as_ref(py),
    })
    .map_err(|error| JsonError::InvalidConversion { error });
    Ok(PyBytes::new(py, (s?).as_slice()).into())
}

pub enum JsonError {
    InvalidConversion { error: serde_json::Error },
}

impl From<JsonError> for PyErr {
    fn from(h: JsonError) -> PyErr {
        match h {
            JsonError::InvalidConversion { error } => {
                PyErr::new::<pyo3::exceptions::TypeError, _>(error.to_string())
            }
        }
    }
}

#[repr(transparent)]
pub struct SerializePyObject<'p, 'a> {
    pub py: Python<'p>,
    pub obj: &'a PyObjectRef,
}

impl<'p, 'a> Serialize for SerializePyObject<'p, 'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let obj_ptr = self.obj.get_type_ptr();
        if unsafe { obj_ptr == STR_PTR } {
            let val = unsafe { <PyUnicode as PyTryFrom>::try_from_unchecked(self.obj) };
            serializer.serialize_str(unsafe { std::str::from_utf8_unchecked(val.as_bytes()) })
        } else if unsafe { obj_ptr == FLOAT_PTR } {
            let val = unsafe { <PyFloat as PyTryFrom>::try_from_unchecked(self.obj) };
            serializer.serialize_f64(val.value())
        } else if unsafe { obj_ptr == INT_PTR } {
            if let Ok(val) = <i64 as FromPyObject>::extract(self.obj) {
                serializer.serialize_i64(val)
            } else {
                Err(ser::Error::custom(format_args!(
                    "Integer exceeds 64-bit max: {:?}",
                    self.obj
                )))
            }
        } else if unsafe { obj_ptr == BOOL_PTR } {
            let val = unsafe { <PyBool as PyTryFrom>::try_from_unchecked(self.obj) };
            serializer.serialize_bool(val.is_true())
        } else if unsafe { obj_ptr == NONE_PTR } {
            serializer.serialize_unit()
        } else if unsafe { obj_ptr == DICT_PTR } {
            let val = unsafe { <PyDict as PyTryFrom>::try_from_unchecked(self.obj) };
            let len = val.len();
            if len != 0 {
                let mut map = serializer.serialize_map(Some(len))?;
                for (key, value) in val.iter() {
                    if unsafe { std::intrinsics::unlikely(key.get_type_ptr() != STR_PTR) } {
                        return Err(ser::Error::custom(format_args!(
                            "Dict key must be str, not: {:?}",
                            key
                        )));
                    }
                    map.serialize_entry(
                        unsafe {
                            std::str::from_utf8_unchecked(
                                <PyUnicode as PyTryFrom>::try_from_unchecked(key).as_bytes(),
                            )
                        },
                        &SerializePyObject {
                            py: self.py,
                            obj: value,
                        },
                    )?;
                }
                map.end()
            } else {
                serializer.serialize_map(None).unwrap().end()
            }
        } else if unsafe { obj_ptr == LIST_PTR } {
            let val = unsafe { <PyList as PyTryFrom>::try_from_unchecked(self.obj) };
            let len = val.len();
            if len != 0 {
                let mut seq = serializer.serialize_seq(Some(len))?;
                for element in val {
                    seq.serialize_element(&SerializePyObject {
                        py: self.py,
                        obj: element,
                    })?
                }
                seq.end()
            } else {
                serializer.serialize_seq(None).unwrap().end()
            }
        } else if unsafe { obj_ptr == TUPLE_PTR } {
            let val = unsafe { <PyTuple as PyTryFrom>::try_from_unchecked(self.obj) };
            let len = val.len();
            if len != 0 {
                let mut seq = serializer.serialize_seq(Some(len))?;
                for element in val {
                    seq.serialize_element(&SerializePyObject {
                        py: self.py,
                        obj: element,
                    })?
                }
                seq.end()
            } else {
                serializer.serialize_seq(None).unwrap().end()
            }
        } else if unsafe { obj_ptr == BYTES_PTR } {
            let val = unsafe { <PyBytes as PyTryFrom>::try_from_unchecked(self.obj) };
            serializer.serialize_str(unsafe { std::str::from_utf8_unchecked(val.as_bytes()) })
        } else {
            Err(ser::Error::custom(format_args!(
                "Type is not JSON serializable: {}",
                self.obj.get_type().name(),
            )))
        }
    }
}
