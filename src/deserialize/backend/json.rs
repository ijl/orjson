// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::deserialize::pyobject::*;
use crate::deserialize::DeserializeError;
use crate::str::unicode_from_str;
use core::ptr::NonNull;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt;

pub(crate) fn deserialize(
    data: &'static str,
) -> Result<NonNull<pyo3_ffi::PyObject>, DeserializeError<'static>> {
    let mut deserializer = serde_json::Deserializer::from_str(data);
    let seed = JsonValue {};
    match seed.deserialize(&mut deserializer) {
        Ok(obj) => {
            deserializer.end().map_err(|e| {
                DeserializeError::from_json(Cow::Owned(e.to_string()), e.line(), e.column(), data)
            })?;
            Ok(obj)
        }
        Err(e) => Err(DeserializeError::from_json(
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

    fn expecting(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
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
        while let Some(key) = map.next_key::<Cow<str>>()? {
            let pykey = get_unicode_key(&key);
            let pyval = map.next_value_seed(self)?;
            let _ = unsafe {
                pyo3_ffi::_PyDict_SetItem_KnownHash(
                    dict_ptr,
                    pykey,
                    pyval.as_ptr(),
                    str_hash!(pykey),
                )
            };
            reverse_pydict_incref!(pykey);
            reverse_pydict_incref!(pyval.as_ptr());
        }
        Ok(nonnull!(dict_ptr))
    }
}
