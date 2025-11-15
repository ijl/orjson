use core::ptr::NonNull;
use pyo3_ffi::PyObject;
use std::io;

pub(crate) trait Writer {
    fn abort(&mut self);
    fn finish(&mut self, append: bool) -> io::Result<NonNull<PyObject>>;
}
