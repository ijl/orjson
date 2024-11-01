// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::*;
use crate::serialize::buffer::SmallFixedBuffer;
use crate::serialize::error::SerializeError;
use crate::serialize::obtype::{pyobject_to_obtype, ObType};
use crate::serialize::per_type::datetimelike::DateTimeLike;
use crate::serialize::per_type::{
    BoolSerializer, DataclassGenericSerializer, Date, DateTime, DefaultSerializer, EnumSerializer,
    FloatSerializer, FragmentSerializer, IntSerializer, ListTupleSerializer, NoneSerializer,
    NumpyScalar, NumpySerializer, StrSerializer, StrSubclassSerializer, Time, ZeroListSerializer,
    UUID,
};
use crate::serialize::serializer::PyObjectSerializer;
use crate::serialize::state::SerializerState;
use crate::str::{unicode_to_str, unicode_to_str_via_ffi};
use crate::typeref::{STR_TYPE, TRUE, VALUE_STR};
use compact_str::CompactString;
use core::ptr::NonNull;
use serde::ser::{Serialize, SerializeMap, Serializer};
use smallvec::SmallVec;

pub struct ZeroDictSerializer;

impl ZeroDictSerializer {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Serialize for ZeroDictSerializer {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(b"{}")
    }
}

pub struct DictGenericSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    state: SerializerState,
    #[allow(dead_code)]
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl DictGenericSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        state: SerializerState,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        DictGenericSerializer {
            ptr: ptr,
            state: state.copy_for_recursive_call(),
            default: default,
        }
    }
}

impl Serialize for DictGenericSerializer {
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if unlikely!(self.state.recursion_limit()) {
            err!(SerializeError::RecursionLimit)
        }

        if unlikely!(ffi!(Py_SIZE(self.ptr)) == 0) {
            ZeroDictSerializer::new().serialize(serializer)
        } else if likely!(opt_disabled!(self.state.opts(), SORT_OR_NON_STR_KEYS)) {
            unsafe {
                core::mem::transmute::<&DictGenericSerializer, &Dict>(self).serialize(serializer)
            }
        } else if opt_enabled!(self.state.opts(), NON_STR_KEYS) {
            unsafe {
                core::mem::transmute::<&DictGenericSerializer, &DictNonStrKey>(self)
                    .serialize(serializer)
            }
        } else {
            unsafe {
                core::mem::transmute::<&DictGenericSerializer, &DictSortedKey>(self)
                    .serialize(serializer)
            }
        }
    }
}

macro_rules! impl_serialize_entry {
    ($map:expr, $self:expr, $key:expr, $value:expr) => {
        match pyobject_to_obtype($value, $self.state.opts()) {
            ObType::Str => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&StrSerializer::new($value))?;
            }
            ObType::StrSubclass => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&StrSubclassSerializer::new($value))?;
            }
            ObType::Int => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&IntSerializer::new($value, $self.state.opts()))?;
            }
            ObType::None => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&NoneSerializer::new()).unwrap();
            }
            ObType::Float => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&FloatSerializer::new($value))?;
            }
            ObType::Bool => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&BoolSerializer::new($value)).unwrap();
            }
            ObType::Datetime => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&DateTime::new($value, $self.state.opts()))?;
            }
            ObType::Date => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&Date::new($value))?;
            }
            ObType::Time => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&Time::new($value, $self.state.opts()))?;
            }
            ObType::Uuid => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&UUID::new($value)).unwrap();
            }
            ObType::Dict => {
                let pyvalue = DictGenericSerializer::new($value, $self.state, $self.default);
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&pyvalue)?;
            }
            ObType::List => {
                if ffi!(Py_SIZE($value)) == 0 {
                    $map.serialize_key($key).unwrap();
                    $map.serialize_value(&ZeroListSerializer::new()).unwrap();
                } else {
                    let pyvalue =
                        ListTupleSerializer::from_list($value, $self.state, $self.default);
                    $map.serialize_key($key).unwrap();
                    $map.serialize_value(&pyvalue)?;
                }
            }
            ObType::Tuple => {
                if ffi!(Py_SIZE($value)) == 0 {
                    $map.serialize_key($key).unwrap();
                    $map.serialize_value(&ZeroListSerializer::new()).unwrap();
                } else {
                    let pyvalue =
                        ListTupleSerializer::from_tuple($value, $self.state, $self.default);
                    $map.serialize_key($key).unwrap();
                    $map.serialize_value(&pyvalue)?;
                }
            }
            ObType::Dataclass => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&DataclassGenericSerializer::new(&PyObjectSerializer::new(
                    $value,
                    $self.state,
                    $self.default,
                )))?;
            }
            ObType::Enum => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&EnumSerializer::new(&PyObjectSerializer::new(
                    $value,
                    $self.state,
                    $self.default,
                )))?;
            }
            ObType::NumpyArray => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&NumpySerializer::new(&PyObjectSerializer::new(
                    $value,
                    $self.state,
                    $self.default,
                )))?;
            }
            ObType::NumpyScalar => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&NumpyScalar::new($value, $self.state.opts()))?;
            }
            ObType::Fragment => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&FragmentSerializer::new($value))?;
            }
            ObType::Unknown => {
                $map.serialize_key($key).unwrap();
                $map.serialize_value(&DefaultSerializer::new(&PyObjectSerializer::new(
                    $value,
                    $self.state,
                    $self.default,
                )))?;
            }
        }
    };
}

pub struct Dict {
    ptr: *mut pyo3_ffi::PyObject,
    state: SerializerState,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl Serialize for Dict {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut pos = 0;
        let mut next_key: *mut pyo3_ffi::PyObject = core::ptr::null_mut();
        let mut next_value: *mut pyo3_ffi::PyObject = core::ptr::null_mut();

        pydict_next!(self.ptr, &mut pos, &mut next_key, &mut next_value);

        let mut map = serializer.serialize_map(None).unwrap();

        let len = ffi!(Py_SIZE(self.ptr)) as usize;
        assume!(len > 0);

        for _ in 0..len {
            let key = next_key;
            let value = next_value;

            pydict_next!(self.ptr, &mut pos, &mut next_key, &mut next_value);

            // key
            let key_as_str = {
                let key_ob_type = ob_type!(key);
                if unlikely!(!is_class_by_type!(key_ob_type, STR_TYPE)) {
                    err!(SerializeError::KeyMustBeStr)
                }
                let tmp = unicode_to_str(key);
                if unlikely!(tmp.is_none()) {
                    err!(SerializeError::InvalidStr)
                };
                tmp.unwrap()
            };

            // value
            impl_serialize_entry!(map, self, key_as_str, value);
        }

        map.end()
    }
}

pub struct DictSortedKey {
    ptr: *mut pyo3_ffi::PyObject,
    state: SerializerState,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl Serialize for DictSortedKey {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut pos = 0;
        let mut next_key: *mut pyo3_ffi::PyObject = core::ptr::null_mut();
        let mut next_value: *mut pyo3_ffi::PyObject = core::ptr::null_mut();

        pydict_next!(self.ptr, &mut pos, &mut next_key, &mut next_value);

        let len = ffi!(Py_SIZE(self.ptr)) as usize;
        assume!(len > 0);

        let mut items: SmallVec<[(&str, *mut pyo3_ffi::PyObject); 8]> =
            SmallVec::with_capacity(len);

        for _ in 0..len as usize {
            let key = next_key;
            let value = next_value;

            pydict_next!(self.ptr, &mut pos, &mut next_key, &mut next_value);

            if unlikely!(unsafe { ob_type!(key) != STR_TYPE }) {
                err!(SerializeError::KeyMustBeStr)
            }
            let data = unicode_to_str(key);
            if unlikely!(data.is_none()) {
                err!(SerializeError::InvalidStr)
            }
            items.push((data.unwrap(), value));
        }

        items.sort_unstable_by(|a, b| a.0.cmp(b.0));

        let mut map = serializer.serialize_map(None).unwrap();
        for (key, val) in items.iter() {
            let pyvalue = PyObjectSerializer::new(*val, self.state, self.default);
            map.serialize_key(key).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }
}

#[inline(never)]
fn non_str_str(key: *mut pyo3_ffi::PyObject) -> Result<CompactString, SerializeError> {
    // because of ObType::Enum
    let uni = unicode_to_str(key);
    if unlikely!(uni.is_none()) {
        Err(SerializeError::InvalidStr)
    } else {
        Ok(CompactString::from(uni.unwrap()))
    }
}

#[cold]
#[inline(never)]
fn non_str_str_subclass(key: *mut pyo3_ffi::PyObject) -> Result<CompactString, SerializeError> {
    let uni = unicode_to_str_via_ffi(key);
    if unlikely!(uni.is_none()) {
        Err(SerializeError::InvalidStr)
    } else {
        Ok(CompactString::from(uni.unwrap()))
    }
}

#[inline(never)]
fn non_str_date(key: *mut pyo3_ffi::PyObject) -> Result<CompactString, SerializeError> {
    let mut buf = SmallFixedBuffer::new();
    Date::new(key).write_buf(&mut buf);
    let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
    Ok(CompactString::from(key_as_str))
}

#[inline(never)]
fn non_str_datetime(
    key: *mut pyo3_ffi::PyObject,
    opts: crate::opt::Opt,
) -> Result<CompactString, SerializeError> {
    let mut buf = SmallFixedBuffer::new();
    let dt = DateTime::new(key, opts);
    if dt.write_buf(&mut buf, opts).is_err() {
        return Err(SerializeError::DatetimeLibraryUnsupported);
    }
    let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
    Ok(CompactString::from(key_as_str))
}

#[cold]
#[inline(never)]
fn non_str_time(
    key: *mut pyo3_ffi::PyObject,
    opts: crate::opt::Opt,
) -> Result<CompactString, SerializeError> {
    let mut buf = SmallFixedBuffer::new();
    let time = Time::new(key, opts);
    if time.write_buf(&mut buf).is_err() {
        return Err(SerializeError::TimeHasTzinfo);
    }
    let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
    Ok(CompactString::from(key_as_str))
}

#[inline(never)]
fn non_str_uuid(key: *mut pyo3_ffi::PyObject) -> Result<CompactString, SerializeError> {
    let mut buf = SmallFixedBuffer::new();
    UUID::new(key).write_buf(&mut buf);
    let key_as_str = str_from_slice!(buf.as_ptr(), buf.len());
    Ok(CompactString::from(key_as_str))
}

#[cold]
#[inline(never)]
fn non_str_float(key: *mut pyo3_ffi::PyObject) -> Result<CompactString, SerializeError> {
    let val = ffi!(PyFloat_AS_DOUBLE(key));
    if !val.is_finite() {
        Ok(CompactString::const_new("null"))
    } else {
        Ok(CompactString::from(ryu::Buffer::new().format_finite(val)))
    }
}

#[inline(never)]
fn non_str_int(key: *mut pyo3_ffi::PyObject) -> Result<CompactString, SerializeError> {
    let ival = ffi!(PyLong_AsLongLong(key));
    if unlikely!(ival == -1 && !ffi!(PyErr_Occurred()).is_null()) {
        ffi!(PyErr_Clear());
        let uval = ffi!(PyLong_AsUnsignedLongLong(key));
        if unlikely!(uval == u64::MAX && !ffi!(PyErr_Occurred()).is_null()) {
            return Err(SerializeError::DictIntegerKey64Bit);
        }
        Ok(CompactString::from(itoa::Buffer::new().format(uval)))
    } else {
        Ok(CompactString::from(itoa::Buffer::new().format(ival)))
    }
}

#[inline(never)]
fn sort_non_str_dict_items(items: &mut SmallVec<[(CompactString, *mut pyo3_ffi::PyObject); 8]>) {
    items.sort_unstable_by(|a, b| a.0.cmp(&b.0));
}

pub struct DictNonStrKey {
    ptr: *mut pyo3_ffi::PyObject,
    state: SerializerState,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl DictNonStrKey {
    fn pyobject_to_string(
        key: *mut pyo3_ffi::PyObject,
        opts: crate::opt::Opt,
    ) -> Result<CompactString, SerializeError> {
        match pyobject_to_obtype(key, opts) {
            ObType::None => Ok(CompactString::const_new("null")),
            ObType::Bool => {
                if unsafe { key == TRUE } {
                    Ok(CompactString::const_new("true"))
                } else {
                    Ok(CompactString::const_new("false"))
                }
            }
            ObType::Int => non_str_int(key),
            ObType::Float => non_str_float(key),
            ObType::Datetime => non_str_datetime(key, opts),
            ObType::Date => non_str_date(key),
            ObType::Time => non_str_time(key, opts),
            ObType::Uuid => non_str_uuid(key),
            ObType::Enum => {
                let value = ffi!(PyObject_GetAttr(key, VALUE_STR));
                debug_assert!(ffi!(Py_REFCNT(value)) >= 2);
                let ret = Self::pyobject_to_string(value, opts);
                ffi!(Py_DECREF(value));
                ret
            }
            ObType::Str => non_str_str(key),
            ObType::StrSubclass => non_str_str_subclass(key),
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

impl Serialize for DictNonStrKey {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut pos = 0;
        let mut next_key: *mut pyo3_ffi::PyObject = core::ptr::null_mut();
        let mut next_value: *mut pyo3_ffi::PyObject = core::ptr::null_mut();

        pydict_next!(self.ptr, &mut pos, &mut next_key, &mut next_value);

        let opts = self.state.opts() & NOT_PASSTHROUGH;

        let len = ffi!(Py_SIZE(self.ptr)) as usize;
        assume!(len > 0);

        let mut items: SmallVec<[(CompactString, *mut pyo3_ffi::PyObject); 8]> =
            SmallVec::with_capacity(len);

        for _ in 0..len {
            let key = next_key;
            let value = next_value;

            pydict_next!(self.ptr, &mut pos, &mut next_key, &mut next_value);

            if is_type!(ob_type!(key), STR_TYPE) {
                let uni = unicode_to_str(key);
                if unlikely!(uni.is_none()) {
                    err!(SerializeError::InvalidStr)
                }
                items.push((CompactString::from(uni.unwrap()), value));
            } else {
                match Self::pyobject_to_string(key, opts) {
                    Ok(key_as_str) => items.push((key_as_str, value)),
                    Err(err) => err!(err),
                }
            }
        }

        if opt_enabled!(opts, SORT_KEYS) {
            sort_non_str_dict_items(&mut items);
        }

        let mut map = serializer.serialize_map(None).unwrap();
        for (key, val) in items.iter() {
            let pyvalue = PyObjectSerializer::new(*val, self.state, self.default);
            map.serialize_key(key).unwrap();
            map.serialize_value(&pyvalue)?;
        }
        map.end()
    }
}
