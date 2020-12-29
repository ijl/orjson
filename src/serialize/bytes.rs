use serde::ser::{Serialize, Serializer};
use crate::opt::{Opt, SERIALIZE_BYTES_AS_JSON, SERIALIZE_BYTES_AS_STRING};
use std::ptr;

pub struct BytesSerializer {
    ptr: *mut pyo3::ffi::PyObject,
    opts: Opt,
}

impl BytesSerializer {
    pub fn new(ptr: *mut pyo3::ffi::PyObject, opts: Opt) -> Self {
        BytesSerializer { ptr, opts }
    }
}

impl<'p> Serialize for BytesSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.opts & SERIALIZE_BYTES_AS_JSON != 0 {
            let bytes_size = ffi!(PyBytes_Size(self.ptr));
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    ffi!(PyBytes_AsString(self.ptr)) as *const u8,
                    bytes_size as usize,
                )
            };
            serializer.serialize_bytes(bytes)
    } else if self.opts & SERIALIZE_BYTES_AS_STRING != 0 {
            let mut bytes_size: pyo3::ffi::Py_ssize_t = 0;
            let mut bytes: *mut i8 = ptr::null_mut();
            ffi!(PyBytes_AsStringAndSize(self.ptr, &mut bytes, &mut bytes_size));
            if unlikely!(bytes.is_null()) {
                err!("bytes could not be read")
            }
            serializer.serialize_str(str_from_slice!(bytes as *const u8, bytes_size))
        } else {
            err!("Type is not JSON serializable: bytes")
        }
    }
}
