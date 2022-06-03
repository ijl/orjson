// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::cache::*;
use crate::deserialize::pyobject::*;
use crate::deserialize::DeserializeError;
use crate::unicode::*;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt;
use std::ptr::NonNull;

pub fn deserialize_json(
    data: &'static str,
) -> Result<NonNull<pyo3_ffi::PyObject>, DeserializeError<'static>> {
    let mut deserializer = serde_json::Deserializer::from_str(data);
    let seed = JsonValue {};
    match seed.deserialize(&mut deserializer) {
        Ok(obj) => {
            deserializer.end().map_err(|e| {
                DeserializeError::new(Cow::Owned(e.to_string()), e.line(), e.column(), data)
            })?;
            Ok(obj)
        }
        Err(e) => Err(DeserializeError::new(
            Cow::Owned(e.to_string()),
            e.line(),
            e.column(),
            data,
        )),
    }
}

#[derive(Clone, Copy)]
struct JsonValue;

impl<'de> DeserializeSeed<'de> for JsonValue {
    type Value = NonNull<pyo3_ffi::PyObject>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for JsonValue {
    type Value = NonNull<pyo3_ffi::PyObject>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("JSON")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(parse_none())
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(parse_bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(parse_i64(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(parse_u64(value))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(parse_f64(value))
    }

    fn visit_borrowed_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(nonnull!(unicode_from_str(value)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(nonnull!(unicode_from_str(value)))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        match seq.next_element_seed(self) {
            Ok(None) => Ok(nonnull!(ffi!(PyList_New(0)))),
            Ok(Some(elem)) => {
                let mut elements: SmallVec<[*mut pyo3_ffi::PyObject; 8]> =
                    SmallVec::with_capacity(8);
                elements.push(elem.as_ptr());
                while let Some(elem) = seq.next_element_seed(self)? {
                    elements.push(elem.as_ptr());
                }
                let ptr = ffi!(PyList_New(elements.len() as isize));
                for (i, &obj) in elements.iter().enumerate() {
                    ffi!(PyList_SET_ITEM(ptr, i as isize, obj));
                }
                Ok(nonnull!(ptr))
            }
            Err(err) => Result::Err(err),
        }
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let dict_ptr = ffi!(PyDict_New());
        while let Some(key) = map.next_key::<beef::lean::Cow<str>>()? {
            let value = map.next_value_seed(self)?;
            let pykey: *mut pyo3_ffi::PyObject;
            let pyhash: pyo3_ffi::Py_hash_t;
            if unlikely!(key.len() > 64) {
                pykey = unicode_from_str(&key);
                pyhash = hash_str(pykey);
            } else {
                let hash = cache_hash(key.as_bytes());
                {
                    let map = unsafe {
                        KEY_MAP
                            .get_mut()
                            .unwrap_or_else(|| unsafe { std::hint::unreachable_unchecked() })
                    };
                    let entry = map.entry(&hash).or_insert_with(
                        || hash,
                        || {
                            let pyob = unicode_from_str(&key);
                            hash_str(pyob);
                            CachedKey::new(pyob)
                        },
                    );
                    pykey = entry.get();
                    pyhash = unsafe { (*pykey.cast::<PyASCIIObject>()).hash }
                }
            }
            let _ = ffi!(_PyDict_SetItem_KnownHash(
                dict_ptr,
                pykey,
                value.as_ptr(),
                pyhash
            ));
            // counter Py_INCREF in insertdict
            ffi!(Py_DECREF(pykey));
            ffi!(Py_DECREF(value.as_ptr()));
        }
        Ok(nonnull!(dict_ptr))
    }
}
