// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::bytes::*;
use crate::exc::*;
use crate::typeref::*;
use crate::unicode::*;
use associative_cache::replacement::RoundRobinReplacement;
use associative_cache::*;
use lazy_static::lazy_static;
use pyo3::prelude::*;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt;
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::ptr::NonNull;
use wyhash::wyhash;

#[derive(Clone)]
#[repr(transparent)]
struct CachedKey(*mut c_void);

unsafe impl Send for CachedKey {}
unsafe impl Sync for CachedKey {}

impl CachedKey {
    fn new(ptr: *mut pyo3::ffi::PyObject) -> CachedKey {
        CachedKey {
            0: ptr as *mut c_void,
        }
    }

    fn get(&mut self) -> *mut pyo3::ffi::PyObject {
        let ptr = self.0 as *mut pyo3::ffi::PyObject;
        ffi!(Py_INCREF(ptr));
        ptr
    }
}

impl Drop for CachedKey {
    fn drop(&mut self) {
        ffi!(Py_DECREF(self.0 as *mut pyo3::ffi::PyObject));
    }
}

type KeyMap =
    AssociativeCache<u64, CachedKey, Capacity512, HashDirectMapped, RoundRobinReplacement>;

lazy_static! {
    static ref KEY_MAP: parking_lot::Mutex<KeyMap> = { parking_lot::Mutex::new(KeyMap::default()) };
}

pub fn deserialize(ptr: *mut pyo3::ffi::PyObject) -> PyResult<NonNull<pyo3::ffi::PyObject>> {
    let data: &str;
    let obj_type_ptr = unsafe { (*ptr).ob_type };
    if is_type!(obj_type_ptr, STR_TYPE) {
        let mut str_size: pyo3::ffi::Py_ssize_t = 0;
        let uni = read_utf8_from_str(ptr, &mut str_size);
        if unlikely!(uni.is_null()) {
            return Err(JSONDecodeError::py_err((INVALID_STR, "", 0)));
        }
        data = str_from_slice!(uni, str_size);
    } else if is_type!(obj_type_ptr, BYTES_TYPE) {
        let buffer = unsafe { PyBytes_AS_STRING(ptr) as *const u8 };
        let length = unsafe { PyBytes_GET_SIZE(ptr) as usize };
        let slice = unsafe { std::slice::from_raw_parts(buffer, length) };
        if encoding_rs::Encoding::utf8_valid_up_to(slice) != length {
            return Err(JSONDecodeError::py_err((INVALID_STR, "", 0)));
        }
        data = unsafe { std::str::from_utf8_unchecked(slice) };
    } else if is_type!(obj_type_ptr, BYTEARRAY_TYPE) {
        let buffer = ffi!(PyByteArray_AsString(ptr)) as *const u8;
        let length = ffi!(PyByteArray_Size(ptr)) as usize;
        let slice = unsafe { std::slice::from_raw_parts(buffer, length) };
        if encoding_rs::Encoding::utf8_valid_up_to(slice) != length {
            return Err(JSONDecodeError::py_err((INVALID_STR, "", 0)));
        }
        data = unsafe { std::str::from_utf8_unchecked(slice) };
    } else {
        return Err(JSONDecodeError::py_err((
            "Input must be str or bytes",
            "",
            0,
        )));
    }

    let seed = JsonValue {};
    let mut deserializer = serde_json::Deserializer::from_str(data);
    match seed.deserialize(&mut deserializer) {
        Ok(obj) => {
            deserializer
                .end()
                .map_err(|e| JSONDecodeError::py_err((e.to_string(), "", 0)))?;
            Ok(unsafe { NonNull::new_unchecked(obj) })
        }
        Err(e) => Err(JSONDecodeError::py_err((e.to_string(), "", 0))),
    }
}

#[derive(Clone, Copy)]
struct JsonValue;

impl<'de, 'a> DeserializeSeed<'de> for JsonValue {
    type Value = *mut pyo3::ffi::PyObject;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'a> Visitor<'de> for JsonValue {
    type Value = *mut pyo3::ffi::PyObject;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("JSON")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        ffi!(Py_INCREF(NONE));
        Ok(unsafe { NONE })
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value {
            ffi!(Py_INCREF(TRUE));
            Ok(unsafe { TRUE })
        } else {
            ffi!(Py_INCREF(FALSE));
            Ok(unsafe { FALSE })
        }
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        #[cfg(target_os = "windows")]
        {
            Ok(ffi!(PyLong_FromLongLong(value)))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(ffi!(PyLong_FromLong(value)))
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        #[cfg(target_os = "windows")]
        {
            Ok(ffi!(PyLong_FromUnsignedLongLong(value)))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(ffi!(PyLong_FromUnsignedLong(value)))
        }
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ffi!(PyFloat_FromDouble(value)))
    }

    fn visit_borrowed_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(str_to_pyobject!(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(str_to_pyobject!(value))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut elements: SmallVec<[*mut pyo3::ffi::PyObject; 8]> = SmallVec::with_capacity(8);
        while let Some(elem) = seq.next_element_seed(self)? {
            elements.push(elem);
        }
        let ptr = ffi!(PyList_New(elements.len() as pyo3::ffi::Py_ssize_t));
        for (i, obj) in elements.iter().enumerate() {
            ffi!(PyList_SET_ITEM(ptr, i as pyo3::ffi::Py_ssize_t, *obj));
        }
        Ok(ptr)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let dict_ptr = ffi!(PyDict_New());
        while let Some(key) = map.next_key::<Cow<str>>()? {
            let pykey: *mut pyo3::ffi::PyObject;
            if unlikely!(key.len() > 64) {
                pykey = str_to_pyobject!(key);
            } else {
                let hash = unsafe { wyhash(key.as_bytes(), HASH_SEED) };
                {
                    let mut map = KEY_MAP.lock();
                    let entry = map
                        .entry(&hash)
                        .or_insert_with(|| hash, || CachedKey::new(str_to_pyobject!(key)));
                    pykey = entry.get();
                }
            };
            let hash = hash_str(pykey);
            let value = map.next_value_seed(self)?;
            let _ = ffi!(_PyDict_SetItem_KnownHash(dict_ptr, pykey, value, hash));
            // counter Py_INCREF in insertdict
            ffi!(Py_DECREF(pykey));
            ffi!(Py_DECREF(value));
        }
        Ok(dict_ptr)
    }
}
