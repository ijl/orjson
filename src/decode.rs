// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::prelude::*;
use pyo3::types::*;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;

import_exception!(json, JSONDecodeError);

pub fn deserialize(py: Python, data: &str) -> PyResult<PyObject> {
    let seed = JsonValue::new(py);
    let mut deserializer = serde_json::Deserializer::from_str(data);
    match seed.deserialize(&mut deserializer) {
        Ok(py_object) => {
            deserializer
                .end()
                .map_err(|e| JSONDecodeError::py_err((e.to_string(), "", 0)))?;
            Ok(py_object)
        }
        Err(e) => {
            return Err(JSONDecodeError::py_err((e.to_string(), "", 0)));
        }
    }
}

#[derive(Clone)]
struct JsonValue<'a> {
    py: Python<'a>,
}

impl<'a> JsonValue<'a> {
    fn new(py: Python<'a>) -> JsonValue<'a> {
        JsonValue { py }
    }
}

impl<'de, 'a> DeserializeSeed<'de> for JsonValue<'a> {
    type Value = PyObject;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'a> Visitor<'de> for JsonValue<'a> {
    type Value = PyObject;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("JSON")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(self.py.None())
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.to_object(self.py))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.to_object(self.py))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.to_object(self.py))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.to_object(self.py))
    }

    fn visit_borrowed_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.to_object(self.py))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value.to_object(self.py))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut elements: SmallVec<[PyObject; 8]> = SmallVec::new();
        while let Some(elem) = seq.next_element_seed(self.clone())? {
            elements.push(elem);
        }
        Ok(elements.as_slice().to_object(self.py))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut elements: SmallVec<[(PyObject, PyObject); 8]> = SmallVec::new();
        while let Some((key, value)) = map.next_entry_seed(PhantomData::<Cow<str>>, self.clone())? {
            elements.push((key.to_object(self.py), value));
        }
        let dict = PyDict::new(self.py);
        for (key, value) in elements.iter() {
            dict.set_item(key, value).unwrap()
        }
        Ok(dict.into())
    }
}
