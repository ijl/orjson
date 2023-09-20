// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::per_type::*;
use crate::serialize::writer::*;
use crate::typeref::*;
use serde::ser::{Serialize, Serializer};
use std::io::Write;
use std::ptr::NonNull;

use crate::serialize::json::{to_writer, to_writer_pretty};

pub const RECURSION_LIMIT: u8 = 255;

pub fn serialize(
    ptr: *mut pyo3_ffi::PyObject,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
    opts: Opt,
) -> Result<NonNull<pyo3_ffi::PyObject>, String> {
    let mut buf = BytesWriter::default();
    let obj = PyObjectSerializer::new(ptr, opts, 0, 0, default);
    let res = if opt_disabled!(opts, INDENT_2) {
        to_writer(&mut buf, &obj)
    } else {
        to_writer_pretty(&mut buf, &obj)
    };
    match res {
        Ok(_) => {
            if opt_enabled!(opts, APPEND_NEWLINE) {
                let _ = buf.write(b"\n");
            }
            Ok(buf.finish())
        }
        Err(err) => {
            ffi!(_Py_Dealloc(buf.bytes_ptr().as_ptr()));
            Err(err.to_string())
        }
    }
}

#[repr(u32)]
pub enum ObType {
    Str,
    Int,
    Bool,
    None,
    Float,
    List,
    Dict,
    Datetime,
    Date,
    Time,
    Tuple,
    Uuid,
    Dataclass,
    NumpyScalar,
    NumpyArray,
    Enum,
    StrSubclass,
    Fragment,
    Unknown,
}

pub fn pyobject_to_obtype(obj: *mut pyo3_ffi::PyObject, opts: Opt) -> ObType {
    let ob_type = ob_type!(obj);
    if is_class_by_type!(ob_type, STR_TYPE) {
        ObType::Str
    } else if is_class_by_type!(ob_type, INT_TYPE) {
        ObType::Int
    } else if is_class_by_type!(ob_type, BOOL_TYPE) {
        ObType::Bool
    } else if is_class_by_type!(ob_type, NONE_TYPE) {
        ObType::None
    } else if is_class_by_type!(ob_type, FLOAT_TYPE) {
        ObType::Float
    } else if is_class_by_type!(ob_type, LIST_TYPE) {
        ObType::List
    } else if is_class_by_type!(ob_type, DICT_TYPE) {
        ObType::Dict
    } else if is_class_by_type!(ob_type, DATETIME_TYPE) && opt_disabled!(opts, PASSTHROUGH_DATETIME)
    {
        ObType::Datetime
    } else {
        pyobject_to_obtype_unlikely(ob_type, opts)
    }
}

#[cfg_attr(feature = "optimize", optimize(size))]
#[inline(never)]
pub fn pyobject_to_obtype_unlikely(ob_type: *mut pyo3_ffi::PyTypeObject, opts: Opt) -> ObType {
    if is_class_by_type!(ob_type, UUID_TYPE) {
        return ObType::Uuid;
    } else if is_class_by_type!(ob_type, TUPLE_TYPE) {
        return ObType::Tuple;
    } else if is_class_by_type!(ob_type, FRAGMENT_TYPE) {
        return ObType::Fragment;
    }

    if opt_disabled!(opts, PASSTHROUGH_DATETIME) {
        if is_class_by_type!(ob_type, DATE_TYPE) {
            return ObType::Date;
        } else if is_class_by_type!(ob_type, TIME_TYPE) {
            return ObType::Time;
        }
    }

    if opt_disabled!(opts, PASSTHROUGH_SUBCLASS) {
        if is_subclass_by_flag!(ob_type, Py_TPFLAGS_UNICODE_SUBCLASS) {
            return ObType::StrSubclass;
        } else if is_subclass_by_flag!(ob_type, Py_TPFLAGS_LONG_SUBCLASS) {
            return ObType::Int;
        } else if is_subclass_by_flag!(ob_type, Py_TPFLAGS_LIST_SUBCLASS) {
            return ObType::List;
        } else if is_subclass_by_flag!(ob_type, Py_TPFLAGS_DICT_SUBCLASS) {
            return ObType::Dict;
        }
    }

    if is_subclass_by_type!(ob_type, ENUM_TYPE) {
        return ObType::Enum;
    }

    if opt_disabled!(opts, PASSTHROUGH_DATACLASS) && pydict_contains!(ob_type, DATACLASS_FIELDS_STR)
    {
        return ObType::Dataclass;
    }

    if unlikely!(opt_enabled!(opts, SERIALIZE_NUMPY)) {
        if is_numpy_scalar(ob_type) {
            return ObType::NumpyScalar;
        } else if is_numpy_array(ob_type) {
            return ObType::NumpyArray;
        }
    }

    ObType::Unknown
}

pub struct PyObjectSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl PyObjectSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        PyObjectSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion,
            default: default,
        }
    }
}

impl Serialize for PyObjectSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match pyobject_to_obtype(self.ptr, self.opts) {
            ObType::Str => StrSerializer::new(self.ptr).serialize(serializer),
            ObType::StrSubclass => StrSubclassSerializer::new(self.ptr).serialize(serializer),
            ObType::Int => {
                if unlikely!(opt_enabled!(self.opts, STRICT_INTEGER)) {
                    Int53Serializer::new(self.ptr).serialize(serializer)
                } else {
                    IntSerializer::new(self.ptr).serialize(serializer)
                }
            }
            ObType::None => NoneSerializer::new().serialize(serializer),
            ObType::Float => FloatSerializer::new(self.ptr).serialize(serializer),
            ObType::Bool => BoolSerializer::new(self.ptr).serialize(serializer),
            ObType::Datetime => DateTime::new(self.ptr, self.opts).serialize(serializer),
            ObType::Date => Date::new(self.ptr).serialize(serializer),
            ObType::Time => Time::new(self.ptr, self.opts).serialize(serializer),
            ObType::Uuid => UUID::new(self.ptr).serialize(serializer),
            ObType::Dict => DictGenericSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
            ObType::List => ListSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
            ObType::Tuple => TupleSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
            ObType::Dataclass => DataclassGenericSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
            ObType::Enum => EnumSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
            ObType::NumpyArray => NumpySerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
            ObType::NumpyScalar => NumpyScalar::new(self.ptr, self.opts).serialize(serializer),
            ObType::Fragment => FragmentSerializer::new(self.ptr).serialize(serializer),
            ObType::Unknown => DefaultSerializer::new(
                self.ptr,
                self.opts,
                self.default_calls,
                self.recursion,
                self.default,
            )
            .serialize(serializer),
        }
    }
}
