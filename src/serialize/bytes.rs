use serde::ser::{Serialize, Serializer};

pub struct BytesSerializer {
    ptr: *mut pyo3::ffi::PyObject
}

impl BytesSerializer {
    pub fn new(ptr: *mut pyo3::ffi::PyObject) -> Self {
        BytesSerializer { ptr }
    }
}

impl<'p> Serialize for BytesSerializer {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let bytes_size = ffi!(PyBytes_Size(self.ptr));
        let bytes = unsafe {
            std::slice::from_raw_parts(
                ffi!(PyBytes_AsString(self.ptr)) as *const u8,
                bytes_size as usize
            )
        };
        serializer.serialize_bytes(bytes)
    }
}

