// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::prelude::*;
use pyo3::types::*;
use serde::ser::{self, Serialize, SerializeMap, SerializeSeq, Serializer};

pub fn serialize(py: Python, obj: PyObject) -> PyResult<PyObject> {
    let typerefs = TypeRefs::new(py);
    let s: Result<Vec<u8>, JsonError> = serde_json::to_vec(&SerializePyObject {
        py: py,
        refs: &typerefs,
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

#[derive(Clone)]
pub struct TypeRefs {
    pub str: *mut pyo3::ffi::PyTypeObject,
    pub bytes: *mut pyo3::ffi::PyTypeObject,
    pub dict: *mut pyo3::ffi::PyTypeObject,
    pub list: *mut pyo3::ffi::PyTypeObject,
    pub tuple: *mut pyo3::ffi::PyTypeObject,
    pub none: *mut pyo3::ffi::PyTypeObject,
    pub bool: *mut pyo3::ffi::PyTypeObject,
    pub int: *mut pyo3::ffi::PyTypeObject,
    pub float: *mut pyo3::ffi::PyTypeObject,
}

impl TypeRefs {
    pub fn new(py: Python) -> TypeRefs {
        TypeRefs {
            str: PyUnicode::new(py, "python").as_ref(py).get_type_ptr(),
            bytes: PyBytes::new(py, b"python").as_ref(py).get_type_ptr(),
            dict: PyDict::new(py).as_ref().get_type_ptr(),
            list: PyList::empty(py).as_ref().get_type_ptr(),
            tuple: PyTuple::empty(py).as_ref(py).get_type_ptr(),
            none: py.None().as_ref(py).get_type_ptr(),
            bool: true.to_object(py).as_ref(py).get_type_ptr(),
            int: 1.to_object(py).as_ref(py).get_type_ptr(),
            float: 1.0.to_object(py).as_ref(py).get_type_ptr(),
        }
    }
}

pub struct SerializePyObject<'p, 'a> {
    pub py: Python<'p>,
    pub refs: &'a TypeRefs,
    pub obj: &'a PyObjectRef,
}

impl<'p, 'a> Serialize for SerializePyObject<'p, 'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let obj_ptr = self.obj.get_type_ptr();
        if obj_ptr == self.refs.str {
            let val: &PyUnicode = self.obj.extract().unwrap();
            serializer.serialize_str(unsafe { std::str::from_utf8_unchecked(val.as_bytes()) })
        } else if obj_ptr == self.refs.bytes {
            let val: &PyBytes = self.obj.extract().unwrap();
            serializer.serialize_str(unsafe { std::str::from_utf8_unchecked(val.as_bytes()) })
        } else if obj_ptr == self.refs.dict {
            let val: &PyDict = self.obj.extract().unwrap();
            let len = val.len();
            if len != 0 {
                let mut map = serializer.serialize_map(Some(len))?;
                for (key, value) in val.iter() {
                    map.serialize_entry(
                        &SerializePyObject {
                            py: self.py,
                            refs: self.refs,
                            obj: key,
                        },
                        &SerializePyObject {
                            py: self.py,
                            refs: self.refs,
                            obj: value,
                        },
                    )?;
                }
                map.end()
            } else {
                serializer.serialize_map(None).unwrap().end()
            }
        } else if obj_ptr == self.refs.list {
            let val: &PyList = self.obj.extract().unwrap();
            let len = val.len();
            if len != 0 {
                let mut seq = serializer.serialize_seq(Some(len))?;
                for element in val {
                    seq.serialize_element(&SerializePyObject {
                        py: self.py,
                        refs: self.refs,
                        obj: element,
                    })?
                }
                seq.end()
            } else {
                serializer.serialize_seq(None).unwrap().end()
            }
        } else if obj_ptr == self.refs.tuple {
            let val: &PyTuple = self.obj.extract().unwrap();
            let len = val.len();
            if len != 0 {
                let mut seq = serializer.serialize_seq(Some(len))?;
                for element in val {
                    seq.serialize_element(&SerializePyObject {
                        py: self.py,
                        refs: self.refs,
                        obj: element,
                    })?
                }
                seq.end()
            } else {
                serializer.serialize_seq(None).unwrap().end()
            }
        } else if obj_ptr == self.refs.bool {
            let val: &PyBool = self.obj.extract().unwrap();
            serializer.serialize_bool(val.is_true())
        } else if obj_ptr == self.refs.int {
            if let Ok(val) = <i64 as FromPyObject>::extract(self.obj) {
                serializer.serialize_i64(val)
            } else {
                Err(ser::Error::custom(format_args!(
                    "Integer exceeds 64-bit max: {:?}",
                    self.obj
                )))
            }
        } else if obj_ptr == self.refs.float {
            let val: &PyFloat = self.obj.extract().unwrap();
            serializer.serialize_f64(val.value())
        } else if obj_ptr == self.refs.none {
            serializer.serialize_unit()
        } else {
            Err(ser::Error::custom(format_args!(
                "Type is not JSON serializable: {}",
                self.obj.get_type().name(),
            )))
        }
    }
}
