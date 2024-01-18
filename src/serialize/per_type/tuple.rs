// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::serialize::serializer::PyObjectSerializer;
use crate::serialize::state::SerializerState;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::NonNull;

pub struct TupleSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    state: SerializerState,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl TupleSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        state: SerializerState,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        TupleSerializer {
            ptr: ptr,
            state: state.copy_for_recursive_call(),
            default: default,
        }
    }
}

impl Serialize for TupleSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if ffi!(Py_SIZE(self.ptr)) == 0 {
            serializer.serialize_seq(Some(0)).unwrap().end()
        } else {
            let mut seq = serializer.serialize_seq(None).unwrap();
            for i in 0..=ffi!(Py_SIZE(self.ptr)) as usize - 1 {
                let elem = ffi!(PyTuple_GET_ITEM(self.ptr, i as isize));
                let value = PyObjectSerializer::new(elem, self.state, self.default);
                seq.serialize_element(&value)?;
            }
            seq.end()
        }
    }
}
