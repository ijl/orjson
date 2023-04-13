// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::cell::RefCell;
use crate::opt::*;
use crate::serialize::serializer::*;

use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::NonNull;
use std::rc::Rc;
use crate::ffi::SuspendGIL;

pub struct TupleSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
    gil: Rc<RefCell<SuspendGIL>>,
}

impl TupleSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
        gil: Rc<RefCell<SuspendGIL>>,
    ) -> Self {
        TupleSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
            gil: gil,
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
                let value = PyObjectSerializer::new(
                    elem,
                    self.opts,
                    self.default_calls,
                    self.recursion,
                    self.default,
                    Rc::clone(&self.gil),
                );
                seq.serialize_element(&value)?;
            }
            seq.end()
        }
    }
}
