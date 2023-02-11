// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::error::INVALID_STR;
use pyo3_ffi::{PyErr_SetNone, PyErr_SetString, PyExc_RecursionError};
use std::ffi::CStr;
use std::ptr::NonNull;

pub enum SerializeError {
    DatetimeLibraryUnsupported,
    DefaultRecursionLimit,
    Integer53Bits,
    Integer64Bits,
    InvalidStr,
    KeyMustBeStr,
    RecursionLimit,
    TimeHasTzinfo,
    DictIntegerKey64Bit,
    DictKeyInvalidType,
    NumpyMalformed,
    NumpyNotCContiguous,
    NumpyUnsupportedDatatype,
    FrozenSetIterError,
    SetIterError,
    GeneratorError,
    GetIterError(NonNull<pyo3_ffi::PyObject>),
    UnsupportedType(NonNull<pyo3_ffi::PyObject>),
}

impl std::fmt::Display for SerializeError {
    #[cold]
    #[cfg_attr(feature = "optimize", optimize(size))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            SerializeError::DatetimeLibraryUnsupported => write!(f, "datetime's timezone library is not supported: use datetime.timezone.utc, pendulum, pytz, or dateutil"),
            SerializeError::DefaultRecursionLimit => {
                write!(f, "default serializer exceeds recursion limit")
            }
            SerializeError::Integer53Bits => write!(f, "Integer exceeds 53-bit range"),
            SerializeError::Integer64Bits => write!(f, "Integer exceeds 64-bit range"),
            SerializeError::InvalidStr => write!(f, "{}", INVALID_STR),
            SerializeError::KeyMustBeStr => write!(f, "Dict key must be str"),
            SerializeError::RecursionLimit => {
                unsafe { PyErr_SetNone(PyExc_RecursionError); }
                write!(f, "Recursion limit reached")
            },
            SerializeError::TimeHasTzinfo => write!(f, "datetime.time must not have tzinfo set"),
            SerializeError::DictIntegerKey64Bit => {
                write!(f, "Dict integer key must be within 64-bit range")
            }
            SerializeError::DictKeyInvalidType => {
                write!(f, "Dict key must a type serializable with OPT_NON_STR_KEYS")
            }
            SerializeError::NumpyMalformed => write!(f, "numpy array is malformed"),
            SerializeError::NumpyNotCContiguous => write!(
                f,
                "numpy array is not C contiguous; use ndarray.tolist() in default"
            ),
            SerializeError::NumpyUnsupportedDatatype => {
                write!(f, "unsupported datatype in numpy array")
            }
            SerializeError::FrozenSetIterError => {
                write!(f, "Error while serializing frozenset")
            }
            SerializeError::SetIterError => {
                write!(f, "Error while serializing set")
            }
            SerializeError::GeneratorError => {
                write!(f, "Error while serializing generator")
            }
            SerializeError::GetIterError(ptr) => {
                let name = unsafe { CStr::from_ptr((*ob_type!(ptr.as_ptr())).tp_name).to_string_lossy() };
                write!(f, "Failed to iterate over {}", name)
            }
            SerializeError::UnsupportedType(ptr) => {
                let name = unsafe { CStr::from_ptr((*ob_type!(ptr.as_ptr())).tp_name).to_string_lossy() };
                write!(f, "Type is not JSON serializable: {}", name)
            }
        }
    }
}
