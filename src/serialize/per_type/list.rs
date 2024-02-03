// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::{PASSTHROUGH_DATETIME, STRICT_INTEGER};
use crate::serialize::error::SerializeError;
use crate::serialize::per_type::{
    BoolSerializer, DateTime, DictGenericSerializer, FloatSerializer, Int53Serializer,
    IntSerializer, NoneSerializer, StrSerializer,
};
use crate::serialize::serializer::PyObjectSerializer;
use crate::serialize::state::SerializerState;
use crate::typeref::*;

use core::ptr::NonNull;
use serde::ser::{Serialize, SerializeSeq, Serializer};

pub struct ZeroListSerializer;

impl ZeroListSerializer {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Serialize for ZeroListSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(b"[]")
    }
}

pub struct ListTupleSerializer {
    data_ptr: *const *mut pyo3_ffi::PyObject,
    state: SerializerState,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
    len: usize,
}

impl ListTupleSerializer {
    pub fn from_list(
        ptr: *mut pyo3_ffi::PyObject,
        state: SerializerState,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        debug_assert!(
            is_type!(ob_type!(ptr), LIST_TYPE)
                || is_subclass_by_flag!(ob_type!(ptr), Py_TPFLAGS_LIST_SUBCLASS)
        );
        let data_ptr = unsafe { (*(ptr as *mut pyo3_ffi::PyListObject)).ob_item };
        let len = ffi!(Py_SIZE(ptr)) as usize;
        Self {
            data_ptr: data_ptr,
            len: len,
            state: state.copy_for_recursive_call(),
            default: default,
        }
    }

    pub fn from_tuple(
        ptr: *mut pyo3_ffi::PyObject,
        state: SerializerState,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        debug_assert!(
            is_type!(ob_type!(ptr), TUPLE_TYPE)
                || is_subclass_by_flag!(ob_type!(ptr), Py_TPFLAGS_TUPLE_SUBCLASS)
        );
        let data_ptr = unsafe { (*(ptr as *mut pyo3_ffi::PyTupleObject)).ob_item.as_ptr() };
        let len = ffi!(Py_SIZE(ptr)) as usize;
        Self {
            data_ptr: data_ptr,
            len: len,
            state: state.copy_for_recursive_call(),
            default: default,
        }
    }
}

impl Serialize for ListTupleSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if unlikely!(self.state.recursion_limit()) {
            err!(SerializeError::RecursionLimit)
        }
        debug_assert!(self.len >= 1);
        let mut seq = serializer.serialize_seq(None).unwrap();
        for idx in 0..=self.len - 1 {
            let value = unsafe { *((self.data_ptr).add(idx)) };
            let value_ob_type = ob_type!(value);
            if is_class_by_type!(value_ob_type, STR_TYPE) {
                seq.serialize_element(&StrSerializer::new(value))?;
            } else if is_class_by_type!(value_ob_type, INT_TYPE) {
                if unlikely!(opt_enabled!(self.state.opts(), STRICT_INTEGER)) {
                    seq.serialize_element(&Int53Serializer::new(value))?;
                } else {
                    seq.serialize_element(&IntSerializer::new(value))?;
                }
            } else if is_class_by_type!(value_ob_type, BOOL_TYPE) {
                seq.serialize_element(&BoolSerializer::new(value))?;
            } else if is_class_by_type!(value_ob_type, NONE_TYPE) {
                seq.serialize_element(&NoneSerializer::new())?;
            } else if is_class_by_type!(value_ob_type, FLOAT_TYPE) {
                seq.serialize_element(&FloatSerializer::new(value))?;
            } else if is_class_by_type!(value_ob_type, DICT_TYPE) {
                let pyvalue = DictGenericSerializer::new(value, self.state, self.default);
                seq.serialize_element(&pyvalue)?;
            } else if is_class_by_type!(value_ob_type, LIST_TYPE) {
                if ffi!(Py_SIZE(value)) == 0 {
                    seq.serialize_element(&ZeroListSerializer::new())?;
                } else {
                    let pyvalue = ListTupleSerializer::from_list(value, self.state, self.default);
                    seq.serialize_element(&pyvalue)?;
                }
            } else if is_class_by_type!(value_ob_type, DATETIME_TYPE)
                && opt_disabled!(self.state.opts(), PASSTHROUGH_DATETIME)
            {
                seq.serialize_element(&DateTime::new(value, self.state.opts()))?;
            } else {
                let pyvalue = PyObjectSerializer::new(value, self.state, self.default);
                seq.serialize_element(&pyvalue)?;
            }
        }
        seq.end()
    }
}
