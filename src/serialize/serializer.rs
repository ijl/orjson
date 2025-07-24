// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::{Opt, APPEND_NEWLINE, ENSURE_ASCII, INDENT_2};
use crate::serialize::obtype::{pyobject_to_obtype, ObType};
use crate::serialize::per_type::{
    BoolSerializer, DataclassGenericSerializer, Date, DateTime, DefaultSerializer,
    DictGenericSerializer, EnumSerializer, FloatSerializer, FragmentSerializer, IntSerializer,
    ListTupleSerializer, NoneSerializer, NumpyScalar, NumpySerializer, StrSerializer,
    StrSubclassSerializer, Time, ZeroListSerializer, UUID,
};
use crate::serialize::state::SerializerState;
use crate::serialize::writer::{
    to_writer, to_writer_ascii, to_writer_pretty, to_writer_pretty_ascii, BytesWriter,
};
use core::ptr::NonNull;
use serde::ser::{Serialize, Serializer};

pub(crate) fn serialize(
    ptr: *mut pyo3_ffi::PyObject,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
    opts: Opt,
) -> Result<NonNull<pyo3_ffi::PyObject>, String> {
    let mut buf = BytesWriter::default();
    let obj = PyObjectSerializer::new(ptr, SerializerState::new(opts), default);
    let res = match (
        opt_enabled!(opts, INDENT_2),
        opt_enabled!(opts, ENSURE_ASCII),
    ) {
        (false, false) => to_writer(&mut buf, &obj),
        (true, false) => to_writer_pretty(&mut buf, &obj),
        (false, true) => to_writer_ascii(&mut buf, &obj),
        (true, true) => to_writer_pretty_ascii(&mut buf, &obj),
    };

    match res {
        Ok(()) => Ok(buf.finish(opt_enabled!(opts, APPEND_NEWLINE))),
        Err(err) => {
            ffi!(Py_DECREF(buf.bytes_ptr().as_ptr()));
            Err(err.to_string())
        }
    }
}

pub(crate) struct PyObjectSerializer {
    pub ptr: *mut pyo3_ffi::PyObject,
    pub state: SerializerState,
    pub default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl PyObjectSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        state: SerializerState,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        PyObjectSerializer {
            ptr: ptr,
            state: state,
            default: default,
        }
    }
}

impl Serialize for PyObjectSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match pyobject_to_obtype(self.ptr, self.state.opts()) {
            ObType::Str => StrSerializer::new(self.ptr).serialize(serializer),
            ObType::StrSubclass => StrSubclassSerializer::new(self.ptr).serialize(serializer),
            ObType::Int => IntSerializer::new(self.ptr, self.state.opts()).serialize(serializer),
            ObType::None => NoneSerializer::new().serialize(serializer),
            ObType::Float => FloatSerializer::new(self.ptr).serialize(serializer),
            ObType::Bool => BoolSerializer::new(self.ptr).serialize(serializer),
            ObType::Datetime => DateTime::new(self.ptr, self.state.opts()).serialize(serializer),
            ObType::Date => Date::new(self.ptr).serialize(serializer),
            ObType::Time => Time::new(self.ptr, self.state.opts()).serialize(serializer),
            ObType::Uuid => UUID::new(self.ptr).serialize(serializer),
            ObType::Dict => {
                DictGenericSerializer::new(self.ptr, self.state, self.default).serialize(serializer)
            }
            ObType::List => {
                if ffi!(Py_SIZE(self.ptr)) == 0 {
                    ZeroListSerializer::new().serialize(serializer)
                } else {
                    ListTupleSerializer::from_list(self.ptr, self.state, self.default)
                        .serialize(serializer)
                }
            }
            ObType::Tuple => {
                if ffi!(Py_SIZE(self.ptr)) == 0 {
                    ZeroListSerializer::new().serialize(serializer)
                } else {
                    ListTupleSerializer::from_tuple(self.ptr, self.state, self.default)
                        .serialize(serializer)
                }
            }
            ObType::Dataclass => DataclassGenericSerializer::new(self).serialize(serializer),
            ObType::Enum => EnumSerializer::new(self).serialize(serializer),
            ObType::NumpyArray => NumpySerializer::new(self).serialize(serializer),
            ObType::NumpyScalar => {
                NumpyScalar::new(self.ptr, self.state.opts()).serialize(serializer)
            }
            ObType::Fragment => FragmentSerializer::new(self.ptr).serialize(serializer),
            ObType::Unknown => DefaultSerializer::new(self).serialize(serializer),
        }
    }
}
