// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// Copyright ijl (2018-2026), Nazar Kostetskyi (2022), Aviram Hassan (2020-2021), Ben Sully (2021)

use crate::ffi::{
    NumpyBool, NumpyDatetime64, NumpyDatetime64Repr, NumpyDatetimeUnit, NumpyFloat16, NumpyFloat32,
    NumpyFloat64, NumpyInt8, NumpyInt16, NumpyInt32, NumpyInt64, NumpyUint8, NumpyUint16,
    NumpyUint32, NumpyUint64, PyTypeObject,
};
use crate::serialize::error::SerializeError;
use crate::serialize::numpy::{
    ItemType, NumpyArray, NumpyBoolArray, NumpyDatetime64Array, NumpyF16Array, NumpyF32Array,
    NumpyF64Array, NumpyI8Array, NumpyI16Array, NumpyI32Array, NumpyI64Array, NumpyScalar,
    NumpyU8Array, NumpyU16Array, NumpyU32Array, NumpyU64Array, PyArrayError, datetime_into_error,
    write_numpy_datetime,
};
use crate::serialize::per_type::{DefaultSerializer, ZeroListSerializer};
use crate::serialize::serializer::PyObjectSerializer;
use crate::serialize::writer::SmallFixedBuffer;
use crate::serialize::writer::f16_to_f32;
use crate::typeref::{NUMPY_TYPES, load_numpy_types};
use serde::ser::{Serialize, SerializeSeq, Serializer};

#[repr(transparent)]
pub(crate) struct NumpySerializer<'a> {
    previous: &'a PyObjectSerializer,
}

impl<'a> NumpySerializer<'a> {
    pub fn new(previous: &'a PyObjectSerializer) -> Self {
        Self { previous: previous }
    }
}

impl Serialize for NumpySerializer<'_> {
    #[cold]
    #[inline(never)]
    #[cfg_attr(feature = "optimize", optimize(size))]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match NumpyArray::new(self.previous.ptr, self.previous.state.opts()) {
            Ok(val) => val.serialize(serializer),
            Err(PyArrayError::Malformed) => err!(SerializeError::NumpyMalformed),
            Err(PyArrayError::NotContiguous | PyArrayError::UnsupportedDataType)
                if self.previous.default.is_some() =>
            {
                DefaultSerializer::new(self.previous).serialize(serializer)
            }
            Err(PyArrayError::NotContiguous) => {
                err!(SerializeError::NumpyNotCContiguous)
            }
            Err(PyArrayError::NotNativeEndian) => {
                err!(SerializeError::NumpyNotNativeEndian)
            }
            Err(PyArrayError::UnsupportedDataType) => {
                err!(SerializeError::NumpyUnsupportedDatatype)
            }
        }
    }
}

macro_rules! slice {
    ($ptr:expr, $size:expr) => {
        unsafe { core::slice::from_raw_parts($ptr, $size) }
    };
}

#[cold]
pub(crate) fn is_numpy_scalar(ob_type: *mut PyTypeObject) -> bool {
    let numpy_types = unsafe { NUMPY_TYPES.get_or_init(load_numpy_types) };
    if numpy_types.is_none() {
        false
    } else {
        let scalar_types = unsafe { numpy_types.unwrap().as_ref() };
        core::ptr::eq(ob_type, scalar_types.float64)
            || core::ptr::eq(ob_type, scalar_types.float32)
            || core::ptr::eq(ob_type, scalar_types.float16)
            || core::ptr::eq(ob_type, scalar_types.int64)
            || core::ptr::eq(ob_type, scalar_types.int16)
            || core::ptr::eq(ob_type, scalar_types.int32)
            || core::ptr::eq(ob_type, scalar_types.int8)
            || core::ptr::eq(ob_type, scalar_types.uint64)
            || core::ptr::eq(ob_type, scalar_types.uint32)
            || core::ptr::eq(ob_type, scalar_types.uint8)
            || core::ptr::eq(ob_type, scalar_types.uint16)
            || core::ptr::eq(ob_type, scalar_types.bool_)
            || core::ptr::eq(ob_type, scalar_types.datetime64)
    }
}

#[cold]
pub(crate) fn is_numpy_array(ob_type: *mut PyTypeObject) -> bool {
    let numpy_types = unsafe { NUMPY_TYPES.get_or_init(load_numpy_types) };
    if numpy_types.is_none() {
        false
    } else {
        let scalar_types = unsafe { numpy_types.unwrap().as_ref() };
        unsafe { core::ptr::eq(ob_type, scalar_types.array) }
    }
}

impl Serialize for NumpyArray {
    #[cold]
    #[inline(never)]
    #[cfg_attr(feature = "optimize", optimize(size))]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !(self.depth >= self.dimensions() || self.shape()[self.depth] != 0) {
            cold_path!();
            ZeroListSerializer::new().serialize(serializer)
        } else if !self.children.is_empty() {
            cold_path!();
            let mut seq = serializer.serialize_seq(None).unwrap();
            for child in &self.children {
                seq.serialize_element(child).unwrap();
            }
            seq.end()
        } else {
            match self.kind {
                ItemType::F64 => {
                    NumpyF64Array::new(slice!(self.data().cast::<f64>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::F32 => {
                    NumpyF32Array::new(slice!(self.data().cast::<f32>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::F16 => {
                    NumpyF16Array::new(slice!(self.data().cast::<u16>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::U64 => {
                    NumpyU64Array::new(slice!(self.data().cast::<u64>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::U32 => {
                    NumpyU32Array::new(slice!(self.data().cast::<u32>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::U16 => {
                    NumpyU16Array::new(slice!(self.data().cast::<u16>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::U8 => {
                    NumpyU8Array::new(slice!(self.data().cast::<u8>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::I64 => {
                    NumpyI64Array::new(slice!(self.data().cast::<i64>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::I32 => {
                    NumpyI32Array::new(slice!(self.data().cast::<i32>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::I16 => {
                    NumpyI16Array::new(slice!(self.data().cast::<i16>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::I8 => {
                    NumpyI8Array::new(slice!(self.data().cast::<i8>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::BOOL => {
                    NumpyBoolArray::new(slice!(self.data().cast::<u8>(), self.num_items()))
                        .serialize(serializer)
                }
                ItemType::DATETIME64(unit) => NumpyDatetime64Array::new(
                    slice!(self.data().cast::<i64>(), self.num_items()),
                    unit,
                    self.opts,
                )
                .serialize(serializer),
            }
        }
    }
}

impl Serialize for NumpyF64Array<'_> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeF64 { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
pub(crate) struct DataTypeF64 {
    obj: f64,
}

impl Serialize for DataTypeF64 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.obj)
    }
}

impl Serialize for NumpyF32Array<'_> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeF32 { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
struct DataTypeF32 {
    obj: f32,
}

impl Serialize for DataTypeF32 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f32(self.obj)
    }
}

impl Serialize for NumpyF16Array<'_> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeF16 { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
struct DataTypeF16 {
    obj: u16,
}

impl Serialize for DataTypeF16 {
    #[cold]
    #[cfg_attr(feature = "optimize", optimize(size))]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f32(f16_to_f32(self.obj))
    }
}

impl Serialize for NumpyU64Array<'_> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeU64 { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
pub(crate) struct DataTypeU64 {
    obj: u64,
}

impl Serialize for DataTypeU64 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.obj)
    }
}

impl Serialize for NumpyU32Array<'_> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeU32 { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
pub(crate) struct DataTypeU32 {
    obj: u32,
}

impl Serialize for DataTypeU32 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.obj)
    }
}

impl Serialize for NumpyU16Array<'_> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeU16 { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
pub(crate) struct DataTypeU16 {
    obj: u16,
}

impl Serialize for DataTypeU16 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(u32::from(self.obj))
    }
}

impl Serialize for NumpyI64Array<'_> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeI64 { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
pub(crate) struct DataTypeI64 {
    obj: i64,
}

impl Serialize for DataTypeI64 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.obj)
    }
}

impl Serialize for NumpyI32Array<'_> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeI32 { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
pub(crate) struct DataTypeI32 {
    obj: i32,
}

impl Serialize for DataTypeI32 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.obj)
    }
}

impl Serialize for NumpyI16Array<'_> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeI16 { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
pub(crate) struct DataTypeI16 {
    obj: i16,
}

impl Serialize for DataTypeI16 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(i32::from(self.obj))
    }
}

impl Serialize for NumpyI8Array<'_> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeI8 { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
pub(crate) struct DataTypeI8 {
    obj: i8,
}

impl Serialize for DataTypeI8 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(i32::from(self.obj))
    }
}

impl Serialize for NumpyU8Array<'_> {
    #[cold]
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeU8 { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
pub(crate) struct DataTypeU8 {
    obj: u8,
}

impl Serialize for DataTypeU8 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(u32::from(self.obj))
    }
}

impl Serialize for NumpyBoolArray<'_> {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            seq.serialize_element(&DataTypeBool { obj: each }).unwrap();
        }
        seq.end()
    }
}

#[repr(transparent)]
pub(crate) struct DataTypeBool {
    obj: u8,
}

impl Serialize for DataTypeBool {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(self.obj == 1)
    }
}

impl Serialize for NumpyScalar {
    #[cold]
    #[inline(never)]
    #[cfg_attr(feature = "optimize", optimize(size))]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unsafe {
            let ob_type = crate::ffi::PyObject_Type(self.ptr);
            let scalar_types =
                unsafe { NUMPY_TYPES.get_or_init(load_numpy_types).unwrap().as_ref() };
            if core::ptr::eq(ob_type, scalar_types.float64) {
                (*(self.ptr.cast::<NumpyFloat64>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.float32) {
                (*(self.ptr.cast::<NumpyFloat32>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.float16) {
                (*(self.ptr.cast::<NumpyFloat16>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.int64) {
                (*(self.ptr.cast::<NumpyInt64>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.int32) {
                (*(self.ptr.cast::<NumpyInt32>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.int16) {
                (*(self.ptr.cast::<NumpyInt16>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.int8) {
                (*(self.ptr.cast::<NumpyInt8>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.uint64) {
                (*(self.ptr.cast::<NumpyUint64>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.uint32) {
                (*(self.ptr.cast::<NumpyUint32>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.uint16) {
                (*(self.ptr.cast::<NumpyUint16>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.uint8) {
                (*(self.ptr.cast::<NumpyUint8>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.bool_) {
                (*(self.ptr.cast::<NumpyBool>())).serialize(serializer)
            } else if core::ptr::eq(ob_type, scalar_types.datetime64) {
                let unit = NumpyDatetimeUnit::from_pyobject(self.ptr);
                let obj = &*self.ptr.cast::<NumpyDatetime64>();
                let dt = unit
                    .datetime(obj.value, self.opts)
                    .map_err(|e| serde::ser::Error::custom(datetime_into_error(e)))?;
                dt.serialize(serializer)
            } else {
                unreachable!()
            }
        }
    }
}
impl Serialize for NumpyInt8 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(i32::from(self.value))
    }
}

impl Serialize for NumpyInt16 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(i32::from(self.value))
    }
}

impl Serialize for NumpyInt32 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.value)
    }
}

impl Serialize for NumpyInt64 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.value)
    }
}

impl Serialize for NumpyUint8 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(u32::from(self.value))
    }
}

impl Serialize for NumpyUint16 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(u32::from(self.value))
    }
}
impl Serialize for NumpyUint32 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.value)
    }
}

impl Serialize for NumpyUint64 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.value)
    }
}

impl Serialize for NumpyFloat16 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f32(f16_to_f32(self.value))
    }
}

impl Serialize for NumpyFloat32 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f32(self.value)
    }
}

impl Serialize for NumpyFloat64 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.value)
    }
}

impl Serialize for NumpyBool {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(self.value)
    }
}

impl Serialize for NumpyDatetime64Array<'_> {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        for &each in self.data.iter() {
            let dt = self
                .unit
                .datetime(each, self.opts)
                .map_err(|e| serde::ser::Error::custom(datetime_into_error(e)))?;
            seq.serialize_element(&dt).unwrap();
        }
        seq.end()
    }
}

impl Serialize for NumpyDatetime64Repr {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = SmallFixedBuffer::new();
        write_numpy_datetime(self, &mut buf);
        serializer.collect_str(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}
