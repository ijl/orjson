// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::typeref::{EMPTY_UNICODE, STR_TYPE};
use crate::util::isize_to_usize;

use core::ffi::c_void;
use core::ptr::NonNull;
use pyo3_ffi::{PyASCIIObject, PyCompactUnicodeObject, PyObject};

fn to_str_via_ffi(op: *mut PyObject) -> Option<&'static str> {
    let mut str_size: pyo3_ffi::Py_ssize_t = 0;
    let ptr = ffi!(PyUnicode_AsUTF8AndSize(op, &mut str_size)).cast::<u8>();
    if unlikely!(ptr.is_null()) {
        None
    } else {
        Some(str_from_slice!(ptr, str_size as usize))
    }
}

#[cfg(feature = "avx512")]
pub type StrDeserializer = unsafe fn(&str) -> *mut pyo3_ffi::PyObject;

#[cfg(feature = "avx512")]
static mut STR_CREATE_FN: StrDeserializer = super::scalar::str_impl_kind_scalar;

pub fn set_str_create_fn() {
    unsafe {
        #[cfg(feature = "avx512")]
        if std::is_x86_feature_detected!("avx512vl") {
            STR_CREATE_FN = crate::str::avx512::create_str_impl_avx512vl;
        }
    }
}

#[cfg(all(Py_3_14, Py_GIL_DISABLED))]
const STATE_COMPACT_ASCII: u32 = u32::from_le(0b000000000000000000_0_1_1_001_00000000);

#[cfg(not(all(Py_3_14, Py_GIL_DISABLED)))]
const STATE_COMPACT_ASCII: u32 = u32::from_le(0b000000000000000000000000_0_1_1_001_00);

#[cfg(all(Py_3_14, Py_GIL_DISABLED))]
const STATE_COMPACT: u32 = u32::from_le(0b000000000000000000_0_0_1_000_00000000);

#[cfg(not(all(Py_3_14, Py_GIL_DISABLED)))]
const STATE_COMPACT: u32 = u32::from_le(0b000000000000000000000000_0_0_1_000_00);

#[cfg(all(Py_3_14, Py_GIL_DISABLED))]
const STATE_KIND_MASK: u32 = u32::from_le(0b111_00000000);

#[cfg(all(Py_3_14, Py_GIL_DISABLED))]
const STATE_KIND_SHIFT: usize = 8;

#[cfg(not(all(Py_3_14, Py_GIL_DISABLED)))]
const STATE_KIND_MASK: u32 = u32::from_le(0b111_00);

#[cfg(not(all(Py_3_14, Py_GIL_DISABLED)))]
const STATE_KIND_SHIFT: usize = 2;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub(crate) struct PyStr {
    ptr: NonNull<PyObject>,
}

unsafe impl Send for PyStr {}
unsafe impl Sync for PyStr {}

impl PyStr {
    pub unsafe fn from_ptr_unchecked(ptr: *mut PyObject) -> PyStr {
        debug_assert!(!ptr.is_null());
        debug_assert!(is_class_by_type!(ob_type!(ptr), STR_TYPE));
        PyStr { ptr: nonnull!(ptr) }
    }

    #[inline(always)]
    pub fn from_str_with_hash(buf: &str) -> PyStr {
        let mut obj = PyStr::from_str(buf);
        obj.hash();
        obj
    }

    #[inline(always)]
    pub fn from_str(buf: &str) -> PyStr {
        if unlikely!(buf.is_empty()) {
            return PyStr {
                ptr: nonnull!(use_immortal!(EMPTY_UNICODE)),
            };
        }
        #[cfg(not(feature = "avx512"))]
        let str_ptr = unsafe { super::scalar::str_impl_kind_scalar(buf) };
        #[cfg(feature = "avx512")]
        let str_ptr = unsafe { STR_CREATE_FN(buf) };
        debug_assert!(!str_ptr.is_null());
        PyStr {
            ptr: nonnull!(str_ptr),
        }
    }

    pub fn hash(&mut self) {
        unsafe {
            let ptr = self.ptr.as_ptr().cast::<PyASCIIObject>();
            let data_ptr: *mut c_void = if (*ptr).state & STATE_COMPACT_ASCII == STATE_COMPACT_ASCII
            {
                ptr.offset(1).cast::<c_void>()
            } else {
                ptr.cast::<PyCompactUnicodeObject>()
                    .offset(1)
                    .cast::<c_void>()
            };
            let num_bytes =
                (*ptr).length * (((*ptr).state & STATE_KIND_MASK) >> STATE_KIND_SHIFT) as isize;
            #[cfg(Py_3_14)]
            let hash = pyo3_ffi::Py_HashBuffer(data_ptr, num_bytes);
            #[cfg(not(Py_3_14))]
            let hash = pyo3_ffi::_Py_HashBytes(data_ptr, num_bytes);
            (*ptr).hash = hash;
            debug_assert!((*ptr).hash != -1);
        }
    }

    #[inline(always)]
    pub fn to_str(self) -> Option<&'static str> {
        unsafe {
            let op = self.ptr.as_ptr();
            if unlikely!((*op.cast::<PyASCIIObject>()).state & STATE_COMPACT == 0) {
                to_str_via_ffi(op)
            } else if (*op.cast::<PyASCIIObject>()).state & STATE_COMPACT_ASCII
                == STATE_COMPACT_ASCII
            {
                let ptr = op.cast::<PyASCIIObject>().offset(1).cast::<u8>();
                let len = isize_to_usize((*op.cast::<PyASCIIObject>()).length);
                Some(str_from_slice!(ptr, len))
            } else if (*op.cast::<PyCompactUnicodeObject>()).utf8_length > 0 {
                let ptr = ((*op.cast::<PyCompactUnicodeObject>()).utf8).cast::<u8>();
                let len = isize_to_usize((*op.cast::<PyCompactUnicodeObject>()).utf8_length);
                Some(str_from_slice!(ptr, len))
            } else {
                to_str_via_ffi(op)
            }
        }
    }

    pub fn as_ptr(self) -> *mut PyObject {
        self.ptr.as_ptr()
    }

    pub fn as_non_null_ptr(self) -> NonNull<PyObject> {
        self.ptr
    }
}

#[repr(transparent)]
pub(crate) struct PyStrSubclass {
    ptr: NonNull<PyObject>,
}

impl PyStrSubclass {
    pub unsafe fn from_ptr_unchecked(ptr: *mut PyObject) -> PyStrSubclass {
        let ob_type = ob_type!(ptr);
        let tp_flags = tp_flags!(ob_type);
        debug_assert!(!ptr.is_null());
        debug_assert!(!is_class_by_type!(ob_type, STR_TYPE));
        debug_assert!(is_subclass_by_flag!(tp_flags, Py_TPFLAGS_UNICODE_SUBCLASS));
        PyStrSubclass { ptr: nonnull!(ptr) }
    }

    #[inline(always)]
    pub fn to_str(&self) -> Option<&'static str> {
        to_str_via_ffi(self.ptr.as_ptr())
    }
}
