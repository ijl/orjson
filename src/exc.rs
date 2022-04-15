// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::ffi::CStr;
use std::ptr::NonNull;

pub const INVALID_STR: &str = "str is not valid UTF-8: surrogates not allowed";

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
    UnsupportedType(NonNull<pyo3_ffi::PyObject>),
}

impl std::fmt::Display for SerializeError {
    #[cold]
    #[cfg_attr(feature = "unstable-simd", optimize(size))]
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
            SerializeError::RecursionLimit => write!(f, "Recursion limit reached"),
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
            SerializeError::UnsupportedType(ptr) => {
                let name = unsafe { CStr::from_ptr((*ob_type!(ptr.as_ptr())).tp_name).to_string_lossy() };
                write!(f, "Type is not JSON serializable: {}", name)
            }
        }
    }
}
