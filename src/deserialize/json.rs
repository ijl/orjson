// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use core::ptr::NonNull;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use serde::de::{self, Deserializer, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use smallvec::SmallVec;

use crate::deserialize::default::deserialize_default;
use crate::deserialize::DeserializeError;
use crate::deserialize::deserializer::Callable;
use crate::deserialize::pyobject::*;
use crate::str::unicode_from_str;

pub static mut DEFAULT_FUNC: Option<HashMap<String, Callable>> = None;

pub fn deserialize_json<const WITH_DEFAULT: bool>(
    data: &'static str
) -> Result<NonNull<pyo3_ffi::PyObject>, DeserializeError<'static>> {
    let mut deserializer = serde_json::Deserializer::from_str(data);
    let deserialized = if WITH_DEFAULT {
        JsonValueWithDefault(JsonValue {}).deserialize(&mut deserializer)
    } else { JsonValue{}.deserialize(&mut deserializer) };
    match deserialized {
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
        visit_seq_generic(self, &mut seq)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let dict_ptr = ffi!(PyDict_New());
        while let Some(key) = map.next_key::<beef::lean::Cow<str>>()? {
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

#[derive(Clone, Copy)]
struct JsonValueWithDefault(JsonValue);


impl<'de> DeserializeSeed<'de> for JsonValueWithDefault {
    type Value = NonNull<pyo3_ffi::PyObject>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for JsonValueWithDefault {
    type Value = NonNull<pyo3_ffi::PyObject>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.expecting(formatter)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        self.0.visit_unit()
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        self.0.visit_bool(value)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        self.0.visit_i64(value)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        self.0.visit_u64(value)
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        self.0.visit_f64(value)
    }

    fn visit_borrowed_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        self.0.visit_borrowed_str(value)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        self.0.visit_str(value)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
    {
        visit_seq_generic(self, &mut seq)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
    {
        let dict_ptr = ffi!(PyDict_New());
        while let Some(key) = map.next_key::<beef::lean::Cow<str>>()? {
            let pykey = get_unicode_key(&key);
            let pyval = map.next_value_seed(self)?.as_ptr();
            unsafe {
                let default_func = DEFAULT_FUNC.as_ref().unwrap_unchecked();
                if let Some(callable) = default_func.get(&*key) {
                    if let Ok(deserialized_obj) = deserialize_default(callable, NonNull::new_unchecked(pyval)) {
                        return Ok(NonNull::new_unchecked(deserialized_obj));
                    }
                }
            }
            let _ = unsafe {
                pyo3_ffi::_PyDict_SetItem_KnownHash(
                    dict_ptr,
                    pykey,
                    pyval,
                    str_hash!(pykey),
                )
            };
            reverse_pydict_incref!(pykey);
            reverse_pydict_incref!(pyval);
        }
        Ok(nonnull!(dict_ptr))
    }
}


fn visit_seq_generic<'de, A, D>(this: D, seq: &mut A) -> Result<NonNull<pyo3_ffi::PyObject>, A::Error>
    where
        A: SeqAccess<'de>,
        D: DeserializeSeed<'de, Value=NonNull<pyo3_ffi::PyObject>> + Copy
{
    match seq.next_element_seed(this) {
        Ok(None) => Ok(nonnull!(ffi!(PyList_New(0)))),
        Ok(Some(elem)) => {
            let mut elements: SmallVec<[*mut pyo3_ffi::PyObject; 8]> =
                SmallVec::with_capacity(8);
            elements.push(elem.as_ptr());
            while let Some(elem) = seq.next_element_seed(this)? {
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

