// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use core::ptr::NonNull;
use std::collections::HashMap;
use std::ptr;

use pyo3_ffi::{Py_ssize_t, PyDict_Check, PyDict_Next, PyObject, PyUnicode_AsUTF8, PyUnicode_Check};

use crate::deserialize::DeserializeError;
use crate::deserialize::utf8::read_input_to_buf;
use crate::typeref::EMPTY_UNICODE;

pub type Callable = Box<dyn Fn(*mut PyObject) -> *mut PyObject>;

pub fn deserialize(
    ptr: *mut pyo3_ffi::PyObject,
    default: Option<NonNull<pyo3_ffi::PyObject>>
) -> Result<NonNull<pyo3_ffi::PyObject>, DeserializeError<'static>> {
    debug_assert!(ffi!(Py_REFCNT(ptr)) >= 1);
    let buffer = read_input_to_buf(ptr)?;

    if unlikely!(buffer.len() == 2) {
        if buffer == b"[]" {
            return Ok(nonnull!(ffi!(PyList_New(0))));
        } else if buffer == b"{}" {
            return Ok(nonnull!(ffi!(PyDict_New())));
        } else if buffer == b"\"\"" {
            unsafe { return Ok(nonnull!(use_immortal!(EMPTY_UNICODE))) }
        }
    }

    let buffer_str = unsafe { std::str::from_utf8_unchecked(buffer) };

    #[cfg(feature = "yyjson")]
    {
        if default.is_some() {
            unsafe fn py_dict_to_hashmap(py_dict: *mut PyObject) -> Option<HashMap<String, Callable>> {
                let mut rust_hashmap = HashMap::new();
                let mut pos: Py_ssize_t = 0;

                // Check if py_dict is a PyDictObject
                if PyDict_Check(py_dict) == 0 {
                    return None;
                }

                // Iterate over the dictionary
                loop {
                    let mut key_ptr: *mut PyObject = ptr::null_mut();
                    let mut value_ptr: *mut PyObject = ptr::null_mut();

                    // Get the next key-value pair
                    let res = PyDict_Next(py_dict, &mut pos, &mut key_ptr, &mut value_ptr);
                    if res == 0 {
                        // End of dictionary
                        break;
                    }
                    // Check if key is a string
                    if PyUnicode_Check(key_ptr) != 0 {
                        // Convert key to Rust string
                        let key_str = py_str_to_string(key_ptr);
                        if let Some(str) = key_str {
                            // Insert key-value pair into Rust hashmap
                            rust_hashmap.insert(str, Box::new(move |item_ptr| {
                                #[cfg(not(Py_3_10))]
                                let default_obj = ffi!(PyObject_CallFunctionObjArgs(
                                    value_ptr,
                                    item_ptr,
                                    core::ptr::null_mut() as *mut pyo3_ffi::PyObject
                                ));
                                #[cfg(Py_3_10)]
                                let default_obj = unsafe {
                                    pyo3_ffi::PyObject_Vectorcall(
                                        value_ptr,
                                        core::ptr::addr_of!(item_ptr),
                                        pyo3_ffi::PyVectorcall_NARGS(1) as usize,
                                        core::ptr::null_mut(),
                                    )
                                };
                                default_obj
                            }) as Callable);
                        } else {
                            // Error converting key to string
                            return None;
                        }
                    }
                }
                Some(rust_hashmap)
            }

            unsafe fn py_str_to_string(py_str: *mut PyObject) -> Option<String> {
                let c_str = PyUnicode_AsUTF8(py_str);
                if c_str.is_null() {
                    return None;
                }
                let c_str = std::ffi::CStr::from_ptr(c_str);
                match c_str.to_str() {
                    Ok(s) => Some(s.to_string()),
                    Err(_) => None,
                }
            }

            let hm = unsafe { py_dict_to_hashmap(default.unwrap_unchecked().as_ptr()) };
            crate::deserialize::yyjson::deserialize_yyjson::<true>(buffer_str, &hm)
        } else {
            crate::deserialize::yyjson::deserialize_yyjson::<false>(buffer_str, &None)
        }
    }

    #[cfg(not(feature = "yyjson"))]
    {
        crate::deserialize::json::deserialize_json(buffer_str)
    }
}
