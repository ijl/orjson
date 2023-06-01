// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::dataclass::*;
use crate::serialize::datetime::*;
use crate::serialize::default::*;
use crate::serialize::dict::*;
use crate::serialize::float::*;
use crate::serialize::fragment::*;
use crate::serialize::int::*;
use crate::serialize::list::*;
use crate::serialize::numpy::*;
use crate::serialize::pyenum::EnumSerializer;
use crate::serialize::str::*;
use crate::serialize::tuple::*;
use crate::serialize::uuid::*;
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
            ffi!(_Py_Dealloc(buf.finish().as_ptr()));
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
    unsafe {
        let ob_type = ob_type!(obj);
        if ob_type == STR_TYPE {
            ObType::Str
        } else if ob_type == INT_TYPE {
            ObType::Int
        } else if ob_type == BOOL_TYPE {
            ObType::Bool
        } else if ob_type == NONE_TYPE {
            ObType::None
        } else if ob_type == FLOAT_TYPE {
            ObType::Float
        } else if ob_type == LIST_TYPE {
            ObType::List
        } else if ob_type == DICT_TYPE {
            ObType::Dict
        } else if ob_type == DATETIME_TYPE && opt_disabled!(opts, PASSTHROUGH_DATETIME) {
            ObType::Datetime
        } else {
            pyobject_to_obtype_unlikely(obj, opts)
        }
    }
}

macro_rules! is_subclass_by_flag {
    ($ob_type:expr, $flag:ident) => {
        (((*$ob_type).tp_flags & pyo3_ffi::$flag) != 0)
    };
}

macro_rules! is_subclass_by_type {
    ($ob_type:expr, $type:ident) => {
        (*($ob_type as *mut pyo3_ffi::PyTypeObject))
            .ob_base
            .ob_base
            .ob_type
            == $type
    };
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
#[inline(never)]
pub fn pyobject_to_obtype_unlikely(obj: *mut pyo3_ffi::PyObject, opts: Opt) -> ObType {
    unsafe {
        let ob_type = ob_type!(obj);
        if ob_type == DATE_TYPE && opt_disabled!(opts, PASSTHROUGH_DATETIME) {
            ObType::Date
        } else if ob_type == TIME_TYPE && opt_disabled!(opts, PASSTHROUGH_DATETIME) {
            ObType::Time
        } else if ob_type == TUPLE_TYPE {
            ObType::Tuple
        } else if ob_type == UUID_TYPE {
            ObType::Uuid
        } else if ob_type == FRAGMENT_TYPE {
            ObType::Fragment
        } else if is_subclass_by_type!(ob_type, ENUM_TYPE) {
            ObType::Enum
        } else if opt_disabled!(opts, PASSTHROUGH_SUBCLASS)
            && is_subclass_by_flag!(ob_type, Py_TPFLAGS_UNICODE_SUBCLASS)
        {
            ObType::StrSubclass
        } else if opt_disabled!(opts, PASSTHROUGH_SUBCLASS)
            && is_subclass_by_flag!(ob_type, Py_TPFLAGS_LONG_SUBCLASS)
        {
            ObType::Int
        } else if opt_disabled!(opts, PASSTHROUGH_SUBCLASS)
            && is_subclass_by_flag!(ob_type, Py_TPFLAGS_LIST_SUBCLASS)
        {
            ObType::List
        } else if opt_disabled!(opts, PASSTHROUGH_SUBCLASS)
            && is_subclass_by_flag!(ob_type, Py_TPFLAGS_DICT_SUBCLASS)
        {
            ObType::Dict
        } else if opt_disabled!(opts, PASSTHROUGH_DATACLASS)
            && pydict_contains!(ob_type, DATACLASS_FIELDS_STR)
        {
            ObType::Dataclass
        } else if opt_enabled!(opts, SERIALIZE_NUMPY) && is_numpy_scalar(ob_type) {
            ObType::NumpyScalar
        } else if opt_enabled!(opts, SERIALIZE_NUMPY) && is_numpy_array(ob_type) {
            ObType::NumpyArray
        } else {
            ObType::Unknown
        }
    }
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
            ObType::None => serializer.serialize_unit(),
            ObType::Float => FloatSerializer::new(self.ptr).serialize(serializer),
            ObType::Bool => serializer.serialize_bool(unsafe { self.ptr == TRUE }),
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
