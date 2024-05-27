use std::ptr::NonNull;

use pyo3_ffi::PyObject;

use crate::deserialize::deserializer::Callable;

#[cold]
#[inline(always)]
pub fn deserialize_default(
    callable: &Callable,
    item: NonNull<PyObject>
) -> Result<*mut PyObject, ()>
{
    let default_obj = callable(item.as_ptr());
    if unlikely!(default_obj.is_null()) {
        Err(())
    } else {
        Ok(default_obj)
    }
}
