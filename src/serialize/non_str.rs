// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2024-2026)

use crate::ffi::{
    PyDateRef, PyDateTimeRef, PyFloatRef, PyStrRef, PyStrSubclassRef, PyTimeRef, PyUuidRef,
};
use crate::serialize::{
    datetime::{write_date, write_datetime, write_time},
    error::SerializeError,
    obtype::ObType,
    writer::{
        SmallFixedBuffer, pyobject_to_obtype, write_float64, write_integer_i64, write_integer_u64,
    },
};
use crate::typeref::{TRUE, VALUE_STR};

fn non_str_str(key: PyStrRef) -> Result<String, SerializeError> {
    // because of ObType::Enum
    match key.as_str() {
        Some(uni) => Ok(String::from(uni)),
        None => {
            cold_path!();
            Err(SerializeError::InvalidStr)
        }
    }
}

fn non_str_str_subclass(key: PyStrSubclassRef) -> Result<String, SerializeError> {
    match key.as_str() {
        Some(uni) => Ok(String::from(uni)),
        None => {
            cold_path!();
            Err(SerializeError::InvalidStr)
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn non_str_date(key: PyDateRef) -> Result<String, SerializeError> {
    let mut buf = SmallFixedBuffer::new();
    write_date(key, &mut buf);
    Ok(buf.to_string())
}

fn non_str_datetime(key: PyDateTimeRef, opts: crate::opt::Opt) -> Result<String, SerializeError> {
    let mut buf = SmallFixedBuffer::new();
    if write_datetime(key, opts, &mut buf).is_err() {
        return Err(SerializeError::DatetimeLibraryUnsupported);
    }
    Ok(buf.to_string())
}

fn non_str_time(key: PyTimeRef, opts: crate::opt::Opt) -> Result<String, SerializeError> {
    let mut buf = SmallFixedBuffer::new();
    write_time(key, opts, &mut buf)?;
    Ok(buf.to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn non_str_uuid(key: PyUuidRef) -> Result<String, SerializeError> {
    let mut buf = SmallFixedBuffer::new();
    UUID::new(key).write_buf(&mut buf);
    Ok(buf.to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn non_str_float(ob: PyFloatRef) -> Result<String, SerializeError> {
    let mut buf = SmallFixedBuffer::new();
    write_float64(&mut buf, ob.value());
    Ok(buf.to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn non_str_int(key: *mut crate::ffi::PyObject) -> Result<String, SerializeError> {
    let ival = unsafe { crate::ffi::PyLong_AsLongLong(key) };
    if ival == -1 && unsafe { !crate::ffi::PyErr_Occurred().is_null() } {
        cold_path!();
        unsafe { crate::ffi::PyErr_Clear() };
        let uval = unsafe { crate::ffi::PyLong_AsUnsignedLongLong(key) };
        if uval == u64::MAX && unsafe { !crate::ffi::PyErr_Occurred().is_null() } {
            cold_path!();
            return Err(SerializeError::DictIntegerKey64Bit);
        }
        let mut buf = SmallFixedBuffer::new();
        write_integer_u64(&mut buf, uval);
        Ok(buf.to_string())
    } else {
        let mut buf = SmallFixedBuffer::new();
        write_integer_i64(&mut buf, ival);
        Ok(buf.to_string())
    }
}

#[inline(never)]
#[cfg_attr(feature = "optimize", optimize(size))]
pub(crate) fn pyobject_to_string(
    key: *mut crate::ffi::PyObject,
    opts: crate::opt::Opt,
) -> Result<String, SerializeError> {
    unsafe {
        match pyobject_to_obtype(key, opts) {
            ObType::None => Ok(String::from("null")),
            ObType::Bool => {
                if unsafe { core::ptr::eq(key, TRUE) } {
                    Ok(String::from("true"))
                } else {
                    Ok(String::from("false"))
                }
            }
            ObType::Int => non_str_int(key),
            ObType::Float => non_str_float(PyFloatRef::from_ptr_unchecked(key)),
            ObType::Datetime => non_str_datetime(PyDateTimeRef::from_ptr_unchecked(key), opts),
            ObType::Date => non_str_date(PyDateRef::from_ptr_unchecked(key)),
            ObType::Time => non_str_time(PyTimeRef::from_ptr_unchecked(key), opts),
            ObType::Uuid => non_str_uuid(PyUuidRef::from_ptr_unchecked(key)),
            ObType::Enum => {
                let value = unsafe { crate::ffi::PyObject_GetAttr(key, VALUE_STR) };
                debug_assert!(unsafe { crate::ffi::Py_REFCNT(value) >= 2 });
                let ret = pyobject_to_string(value, opts);
                unsafe { crate::ffi::Py_DECREF(value) };
                ret
            }
            ObType::Str => non_str_str(PyStrRef::from_ptr_unchecked(key)),
            ObType::StrSubclass => non_str_str_subclass(PyStrSubclassRef::from_ptr_unchecked(key)),
            ObType::Tuple
            | ObType::NumpyScalar
            | ObType::NumpyArray
            | ObType::Dict
            | ObType::List
            | ObType::Dataclass
            | ObType::Fragment
            | ObType::Unknown => Err(SerializeError::DictKeyInvalidType),
        }
    }
}
