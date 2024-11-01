use crate::opt::*;

use crate::serialize::buffer::SmallFixedBuffer;
use crate::serialize::error::SerializeError;
use crate::serialize::per_type::{
    DateTimeError, DateTimeLike, DefaultSerializer, Offset, ZeroListSerializer,
};
use crate::serialize::serializer::PyObjectSerializer;
use crate::typeref::{load_numpy_types, ARRAY_STRUCT_STR, DESCR_STR, DTYPE_STR, NUMPY_TYPES};
use core::ffi::{c_char, c_int, c_void};
use jiff::civil::DateTime;
use jiff::Timestamp;
use pyo3_ffi::*;
use serde::ser::{self, Serialize, SerializeSeq, Serializer};
use std::fmt;

#[repr(transparent)]
pub struct NumpySerializer<'a> {
    previous: &'a PyObjectSerializer,
}

impl<'a> NumpySerializer<'a> {
    pub fn new(previous: &'a PyObjectSerializer) -> Self {
        Self { previous: previous }
    }
}

impl<'a> Serialize for NumpySerializer<'a> {
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
            Err(PyArrayError::NotContiguous) | Err(PyArrayError::UnsupportedDataType)
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
pub fn is_numpy_scalar(ob_type: *mut PyTypeObject) -> bool {
    let numpy_types = unsafe { NUMPY_TYPES.get_or_init(load_numpy_types) };
    if numpy_types.is_none() {
        false
    } else {
        let scalar_types = unsafe { numpy_types.unwrap().as_ref() };
        ob_type == scalar_types.float64
            || ob_type == scalar_types.float32
            || ob_type == scalar_types.float16
            || ob_type == scalar_types.int64
            || ob_type == scalar_types.int16
            || ob_type == scalar_types.int32
            || ob_type == scalar_types.int8
            || ob_type == scalar_types.uint64
            || ob_type == scalar_types.uint32
            || ob_type == scalar_types.uint8
            || ob_type == scalar_types.uint16
            || ob_type == scalar_types.bool_
            || ob_type == scalar_types.datetime64
    }
}

#[cold]
pub fn is_numpy_array(ob_type: *mut PyTypeObject) -> bool {
    let numpy_types = unsafe { NUMPY_TYPES.get_or_init(load_numpy_types) };
    if numpy_types.is_none() {
        false
    } else {
        let scalar_types = unsafe { numpy_types.unwrap().as_ref() };
        unsafe { ob_type == scalar_types.array }
    }
}

#[repr(C)]
pub struct PyCapsule {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub pointer: *mut c_void,
    pub name: *const c_char,
    pub context: *mut c_void,
    pub destructor: *mut c_void, // should be typedef void (*PyCapsule_Destructor)(PyObject *);
}

// https://docs.scipy.org/doc/numpy/reference/arrays.interface.html#c.__array_struct__

const NPY_ARRAY_C_CONTIGUOUS: c_int = 0x1;
const NPY_ARRAY_NOTSWAPPED: c_int = 0x200;

#[repr(C)]
pub struct PyArrayInterface {
    pub two: c_int,
    pub nd: c_int,
    pub typekind: c_char,
    pub itemsize: c_int,
    pub flags: c_int,
    pub shape: *mut Py_intptr_t,
    pub strides: *mut Py_intptr_t,
    pub data: *mut c_void,
    pub descr: *mut PyObject,
}

#[derive(Clone, Copy)]
pub enum ItemType {
    BOOL,
    DATETIME64(NumpyDatetimeUnit),
    F16,
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl ItemType {
    fn find(array: *mut PyArrayInterface, ptr: *mut PyObject) -> Option<ItemType> {
        match unsafe { ((*array).typekind, (*array).itemsize) } {
            (098, 1) => Some(ItemType::BOOL),
            (077, 8) => {
                let unit = NumpyDatetimeUnit::from_pyobject(ptr);
                Some(ItemType::DATETIME64(unit))
            }
            (102, 2) => Some(ItemType::F16),
            (102, 4) => Some(ItemType::F32),
            (102, 8) => Some(ItemType::F64),
            (105, 1) => Some(ItemType::I8),
            (105, 2) => Some(ItemType::I16),
            (105, 4) => Some(ItemType::I32),
            (105, 8) => Some(ItemType::I64),
            (117, 1) => Some(ItemType::U8),
            (117, 2) => Some(ItemType::U16),
            (117, 4) => Some(ItemType::U32),
            (117, 8) => Some(ItemType::U64),
            _ => None,
        }
    }
}

pub enum PyArrayError {
    Malformed,
    NotContiguous,
    NotNativeEndian,
    UnsupportedDataType,
}

// >>> arr = numpy.array([[[1, 2], [3, 4]], [[5, 6], [7, 8]]], numpy.int32)
// >>> arr.ndim
// 3
// >>> arr.shape
// (2, 2, 2)
// >>> arr.strides
// (16, 8, 4)
pub struct NumpyArray {
    array: *mut PyArrayInterface,
    position: Vec<isize>,
    children: Vec<NumpyArray>,
    depth: usize,
    capsule: *mut PyCapsule,
    kind: ItemType,
    opts: Opt,
}

impl NumpyArray {
    #[cold]
    #[inline(never)]
    #[cfg_attr(feature = "optimize", optimize(size))]
    pub fn new(ptr: *mut PyObject, opts: Opt) -> Result<Self, PyArrayError> {
        let capsule = ffi!(PyObject_GetAttr(ptr, ARRAY_STRUCT_STR));
        let array = unsafe { (*(capsule as *mut PyCapsule)).pointer as *mut PyArrayInterface };
        if unsafe { (*array).two != 2 } {
            ffi!(Py_DECREF(capsule));
            Err(PyArrayError::Malformed)
        } else if unsafe { (*array).flags } & NPY_ARRAY_C_CONTIGUOUS != NPY_ARRAY_C_CONTIGUOUS {
            ffi!(Py_DECREF(capsule));
            Err(PyArrayError::NotContiguous)
        } else if unsafe { (*array).flags } & NPY_ARRAY_NOTSWAPPED != NPY_ARRAY_NOTSWAPPED {
            ffi!(Py_DECREF(capsule));
            Err(PyArrayError::NotNativeEndian)
        } else {
            let num_dimensions = unsafe { (*array).nd as usize };
            if num_dimensions == 0 {
                ffi!(Py_DECREF(capsule));
                return Err(PyArrayError::UnsupportedDataType);
            }
            match ItemType::find(array, ptr) {
                None => {
                    ffi!(Py_DECREF(capsule));
                    Err(PyArrayError::UnsupportedDataType)
                }
                Some(kind) => {
                    let mut pyarray = NumpyArray {
                        array: array,
                        position: vec![0; num_dimensions],
                        children: Vec::with_capacity(num_dimensions),
                        depth: 0,
                        capsule: capsule as *mut PyCapsule,
                        kind: kind,
                        opts,
                    };
                    if pyarray.dimensions() > 1 {
                        pyarray.build();
                    }
                    Ok(pyarray)
                }
            }
        }
    }

    #[cfg_attr(feature = "optimize", optimize(size))]
    fn child_from_parent(&self, position: Vec<isize>, num_children: usize) -> Self {
        let mut arr = NumpyArray {
            array: self.array,
            position: position,
            children: Vec::with_capacity(num_children),
            depth: self.depth + 1,
            capsule: self.capsule,
            kind: self.kind,
            opts: self.opts,
        };
        arr.build();
        arr
    }

    #[cfg_attr(feature = "optimize", optimize(size))]
    fn build(&mut self) {
        if self.depth < self.dimensions() - 1 {
            for i in 0..self.shape()[self.depth] {
                let mut position: Vec<isize> = self.position.to_vec();
                position[self.depth] = i;
                let num_children: usize = if self.depth < self.dimensions() - 2 {
                    self.shape()[self.depth + 1] as usize
                } else {
                    0
                };
                self.children
                    .push(self.child_from_parent(position, num_children))
            }
        }
    }

    #[inline(always)]
    fn data(&self) -> *const c_void {
        let offset = self
            .strides()
            .iter()
            .zip(self.position.iter().copied())
            .take(self.depth)
            .map(|(a, b)| a * b)
            .sum::<isize>();
        unsafe { (*self.array).data.offset(offset) }
    }

    fn num_items(&self) -> usize {
        self.shape()[self.shape().len() - 1] as usize
    }

    fn dimensions(&self) -> usize {
        unsafe { (*self.array).nd as usize }
    }

    fn shape(&self) -> &[isize] {
        slice!((*self.array).shape as *const isize, self.dimensions())
    }

    fn strides(&self) -> &[isize] {
        slice!((*self.array).strides as *const isize, self.dimensions())
    }
}

impl Drop for NumpyArray {
    fn drop(&mut self) {
        if self.depth == 0 {
            ffi!(Py_DECREF(self.array as *mut pyo3_ffi::PyObject));
            ffi!(Py_DECREF(self.capsule as *mut pyo3_ffi::PyObject));
        }
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
        if unlikely!(!(self.depth >= self.dimensions() || self.shape()[self.depth] != 0)) {
            ZeroListSerializer::new().serialize(serializer)
        } else if !self.children.is_empty() {
            let mut seq = serializer.serialize_seq(None).unwrap();
            for child in &self.children {
                seq.serialize_element(child).unwrap();
            }
            seq.end()
        } else {
            match self.kind {
                ItemType::F64 => {
                    NumpyF64Array::new(slice!(self.data() as *const f64, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::F32 => {
                    NumpyF32Array::new(slice!(self.data() as *const f32, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::F16 => {
                    NumpyF16Array::new(slice!(self.data() as *const u16, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::U64 => {
                    NumpyU64Array::new(slice!(self.data() as *const u64, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::U32 => {
                    NumpyU32Array::new(slice!(self.data() as *const u32, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::U16 => {
                    NumpyU16Array::new(slice!(self.data() as *const u16, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::U8 => {
                    NumpyU8Array::new(slice!(self.data() as *const u8, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::I64 => {
                    NumpyI64Array::new(slice!(self.data() as *const i64, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::I32 => {
                    NumpyI32Array::new(slice!(self.data() as *const i32, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::I16 => {
                    NumpyI16Array::new(slice!(self.data() as *const i16, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::I8 => {
                    NumpyI8Array::new(slice!(self.data() as *const i8, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::BOOL => {
                    NumpyBoolArray::new(slice!(self.data() as *const u8, self.num_items()))
                        .serialize(serializer)
                }
                ItemType::DATETIME64(unit) => NumpyDatetime64Array::new(
                    slice!(self.data() as *const i64, self.num_items()),
                    unit,
                    self.opts,
                )
                .serialize(serializer),
            }
        }
    }
}

#[repr(transparent)]
struct NumpyF64Array<'a> {
    data: &'a [f64],
}

impl<'a> NumpyF64Array<'a> {
    fn new(data: &'a [f64]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyF64Array<'a> {
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
pub struct DataTypeF64 {
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

#[repr(transparent)]
struct NumpyF32Array<'a> {
    data: &'a [f32],
}

impl<'a> NumpyF32Array<'a> {
    fn new(data: &'a [f32]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyF32Array<'a> {
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

#[repr(transparent)]
struct NumpyF16Array<'a> {
    data: &'a [u16],
}

impl<'a> NumpyF16Array<'a> {
    fn new(data: &'a [u16]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyF16Array<'a> {
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
        let as_f16 = half::f16::from_bits(self.obj);
        serializer.serialize_f32(as_f16.to_f32())
    }
}

#[repr(transparent)]
struct NumpyU64Array<'a> {
    data: &'a [u64],
}

impl<'a> NumpyU64Array<'a> {
    fn new(data: &'a [u64]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyU64Array<'a> {
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
pub struct DataTypeU64 {
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

#[repr(transparent)]
struct NumpyU32Array<'a> {
    data: &'a [u32],
}

impl<'a> NumpyU32Array<'a> {
    fn new(data: &'a [u32]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyU32Array<'a> {
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
pub struct DataTypeU32 {
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

#[repr(transparent)]
struct NumpyU16Array<'a> {
    data: &'a [u16],
}

impl<'a> NumpyU16Array<'a> {
    fn new(data: &'a [u16]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyU16Array<'a> {
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
pub struct DataTypeU16 {
    obj: u16,
}

impl Serialize for DataTypeU16 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.obj as u32)
    }
}

#[repr(transparent)]
struct NumpyI64Array<'a> {
    data: &'a [i64],
}

impl<'a> NumpyI64Array<'a> {
    fn new(data: &'a [i64]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyI64Array<'a> {
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
pub struct DataTypeI64 {
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

#[repr(transparent)]
struct NumpyI32Array<'a> {
    data: &'a [i32],
}

impl<'a> NumpyI32Array<'a> {
    fn new(data: &'a [i32]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyI32Array<'a> {
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
pub struct DataTypeI32 {
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

#[repr(transparent)]
struct NumpyI16Array<'a> {
    data: &'a [i16],
}

impl<'a> NumpyI16Array<'a> {
    fn new(data: &'a [i16]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyI16Array<'a> {
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
pub struct DataTypeI16 {
    obj: i16,
}

impl Serialize for DataTypeI16 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.obj as i32)
    }
}

#[repr(transparent)]
struct NumpyI8Array<'a> {
    data: &'a [i8],
}

impl<'a> NumpyI8Array<'a> {
    fn new(data: &'a [i8]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyI8Array<'a> {
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
pub struct DataTypeI8 {
    obj: i8,
}

impl Serialize for DataTypeI8 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.obj as i32)
    }
}

#[repr(transparent)]
struct NumpyU8Array<'a> {
    data: &'a [u8],
}

impl<'a> NumpyU8Array<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyU8Array<'a> {
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
pub struct DataTypeU8 {
    obj: u8,
}

impl Serialize for DataTypeU8 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.obj as u32)
    }
}

#[repr(transparent)]
struct NumpyBoolArray<'a> {
    data: &'a [u8],
}

impl<'a> NumpyBoolArray<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Serialize for NumpyBoolArray<'a> {
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
pub struct DataTypeBool {
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

pub struct NumpyScalar {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
}

impl NumpyScalar {
    pub fn new(ptr: *mut PyObject, opts: Opt) -> Self {
        NumpyScalar { ptr, opts }
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
            let ob_type = ob_type!(self.ptr);
            let scalar_types =
                unsafe { NUMPY_TYPES.get_or_init(load_numpy_types).unwrap().as_ref() };
            if ob_type == scalar_types.float64 {
                (*(self.ptr as *mut NumpyFloat64)).serialize(serializer)
            } else if ob_type == scalar_types.float32 {
                (*(self.ptr as *mut NumpyFloat32)).serialize(serializer)
            } else if ob_type == scalar_types.float16 {
                (*(self.ptr as *mut NumpyFloat16)).serialize(serializer)
            } else if ob_type == scalar_types.int64 {
                (*(self.ptr as *mut NumpyInt64)).serialize(serializer)
            } else if ob_type == scalar_types.int32 {
                (*(self.ptr as *mut NumpyInt32)).serialize(serializer)
            } else if ob_type == scalar_types.int16 {
                (*(self.ptr as *mut NumpyInt16)).serialize(serializer)
            } else if ob_type == scalar_types.int8 {
                (*(self.ptr as *mut NumpyInt8)).serialize(serializer)
            } else if ob_type == scalar_types.uint64 {
                (*(self.ptr as *mut NumpyUint64)).serialize(serializer)
            } else if ob_type == scalar_types.uint32 {
                (*(self.ptr as *mut NumpyUint32)).serialize(serializer)
            } else if ob_type == scalar_types.uint16 {
                (*(self.ptr as *mut NumpyUint16)).serialize(serializer)
            } else if ob_type == scalar_types.uint8 {
                (*(self.ptr as *mut NumpyUint8)).serialize(serializer)
            } else if ob_type == scalar_types.bool_ {
                (*(self.ptr as *mut NumpyBool)).serialize(serializer)
            } else if ob_type == scalar_types.datetime64 {
                let unit = NumpyDatetimeUnit::from_pyobject(self.ptr);
                let obj = &*(self.ptr as *mut NumpyDatetime64);
                let dt = unit
                    .datetime(obj.value, self.opts)
                    .map_err(NumpyDateTimeError::into_serde_err)?;
                dt.serialize(serializer)
            } else {
                unreachable!()
            }
        }
    }
}

#[repr(C)]
pub struct NumpyInt8 {
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: i8,
}

impl Serialize for NumpyInt8 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.value as i32)
    }
}

#[repr(C)]
pub struct NumpyInt16 {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub value: i16,
}

impl Serialize for NumpyInt16 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.value as i32)
    }
}

#[repr(C)]
pub struct NumpyInt32 {
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: i32,
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

#[repr(C)]
pub struct NumpyInt64 {
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: i64,
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

#[repr(C)]
pub struct NumpyUint8 {
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: u8,
}

impl Serialize for NumpyUint8 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.value as u32)
    }
}

#[repr(C)]
pub struct NumpyUint16 {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub value: u16,
}

impl Serialize for NumpyUint16 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.value as u32)
    }
}

#[repr(C)]
pub struct NumpyUint32 {
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: u32,
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

#[repr(C)]
pub struct NumpyUint64 {
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: u64,
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

#[repr(C)]
pub struct NumpyFloat16 {
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: u16,
}

impl Serialize for NumpyFloat16 {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let as_f16 = half::f16::from_bits(self.value);
        serializer.serialize_f32(as_f16.to_f32())
    }
}

#[repr(C)]
pub struct NumpyFloat32 {
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: f32,
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

#[repr(C)]
pub struct NumpyFloat64 {
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: f64,
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

#[repr(C)]
pub struct NumpyBool {
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: bool,
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

/// This mimicks the units supported by numpy's datetime64 type.
///
/// See
/// https://github.com/numpy/numpy/blob/fc8e3bbe419748ac5c6b7f3d0845e4bafa74644b/numpy/core/include/numpy/ndarraytypes.h#L268-L282.
#[derive(Clone, Copy)]
pub enum NumpyDatetimeUnit {
    NaT,
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
    Picoseconds,
    Femtoseconds,
    Attoseconds,
    Generic,
}

impl fmt::Display for NumpyDatetimeUnit {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unit = match self {
            Self::NaT => "NaT",
            Self::Years => "years",
            Self::Months => "months",
            Self::Weeks => "weeks",
            Self::Days => "days",
            Self::Hours => "hours",
            Self::Minutes => "minutes",
            Self::Seconds => "seconds",
            Self::Milliseconds => "milliseconds",
            Self::Microseconds => "microseconds",
            Self::Nanoseconds => "nanoseconds",
            Self::Picoseconds => "picoseconds",
            Self::Femtoseconds => "femtoseconds",
            Self::Attoseconds => "attoseconds",
            Self::Generic => "generic",
        };
        write!(f, "{}", unit)
    }
}

#[derive(Clone, Copy)]
enum NumpyDateTimeError {
    UnsupportedUnit(NumpyDatetimeUnit),
    Unrepresentable { unit: NumpyDatetimeUnit, val: i64 },
}

impl NumpyDateTimeError {
    #[cold]
    fn into_serde_err<T: ser::Error>(self) -> T {
        let err = match self {
            Self::UnsupportedUnit(unit) => format!("unsupported numpy.datetime64 unit: {}", unit),
            Self::Unrepresentable { unit, val } => {
                format!("unrepresentable numpy.datetime64: {} {}", val, unit)
            }
        };
        ser::Error::custom(err)
    }
}

macro_rules! to_jiff_datetime {
    ($timestamp:expr, $self:expr, $val:expr) => {
        Ok(
            ($timestamp.map_err(|_| NumpyDateTimeError::Unrepresentable {
                unit: *$self,
                val: $val,
            })?)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .datetime(),
        )
    };
}

impl NumpyDatetimeUnit {
    /// Create a `NumpyDatetimeUnit` from a pointer to a Python object holding a
    /// numpy array.
    ///
    /// This function must only be called with pointers to numpy arrays.
    ///
    /// We need to look inside the `obj.dtype.descr` attribute of the Python
    /// object rather than using the `descr` field of the `__array_struct__`
    /// because that field isn't populated for datetime64 arrays; see
    /// https://github.com/numpy/numpy/issues/5350.
    #[cold]
    #[cfg_attr(feature = "optimize", optimize(size))]
    fn from_pyobject(ptr: *mut PyObject) -> Self {
        let dtype = ffi!(PyObject_GetAttr(ptr, DTYPE_STR));
        let descr = ffi!(PyObject_GetAttr(dtype, DESCR_STR));
        let el0 = ffi!(PyList_GET_ITEM(descr, 0));
        let descr_str = ffi!(PyTuple_GET_ITEM(el0, 1));
        let uni = crate::str::unicode_to_str(descr_str).unwrap();
        if uni.len() < 5 {
            return Self::NaT;
        }
        // unit descriptions are found at
        // https://github.com/numpy/numpy/blob/b235f9e701e14ed6f6f6dcba885f7986a833743f/numpy/core/src/multiarray/datetime.c#L79-L96.
        let ret = match &uni[4..uni.len() - 1] {
            "Y" => Self::Years,
            "M" => Self::Months,
            "W" => Self::Weeks,
            "D" => Self::Days,
            "h" => Self::Hours,
            "m" => Self::Minutes,
            "s" => Self::Seconds,
            "ms" => Self::Milliseconds,
            "us" => Self::Microseconds,
            "ns" => Self::Nanoseconds,
            "ps" => Self::Picoseconds,
            "fs" => Self::Femtoseconds,
            "as" => Self::Attoseconds,
            "generic" => Self::Generic,
            _ => unreachable!(),
        };
        ffi!(Py_DECREF(dtype));
        ffi!(Py_DECREF(descr));
        ret
    }

    /// Return a `NumpyDatetime64Repr` for a value in array with this unit.
    ///
    /// Returns an `Err(NumpyDateTimeError)` if the value is invalid for this unit.
    #[cold]
    #[cfg_attr(feature = "optimize", optimize(size))]
    fn datetime(&self, val: i64, opts: Opt) -> Result<NumpyDatetime64Repr, NumpyDateTimeError> {
        match self {
            Self::Years => Ok(DateTime::new(
                (val + 1970)
                    .try_into()
                    .map_err(|_| NumpyDateTimeError::Unrepresentable { unit: *self, val })?,
                1,
                1,
                0,
                0,
                0,
                0,
            )
            .unwrap()),
            Self::Months => Ok(DateTime::new(
                (val / 12 + 1970)
                    .try_into()
                    .map_err(|_| NumpyDateTimeError::Unrepresentable { unit: *self, val })?,
                (val % 12 + 1)
                    .try_into()
                    .map_err(|_| NumpyDateTimeError::Unrepresentable { unit: *self, val })?,
                1,
                0,
                0,
                0,
                0,
            )
            .unwrap()),
            Self::Weeks => {
                to_jiff_datetime!(Timestamp::from_second(val * 7 * 24 * 60 * 60), self, val)
            }
            Self::Days => to_jiff_datetime!(Timestamp::from_second(val * 24 * 60 * 60), self, val),
            Self::Hours => to_jiff_datetime!(Timestamp::from_second(val * 60 * 60), self, val),
            Self::Minutes => to_jiff_datetime!(Timestamp::from_second(val * 60), self, val),
            Self::Seconds => to_jiff_datetime!(Timestamp::from_second(val), self, val),
            Self::Milliseconds => to_jiff_datetime!(Timestamp::from_millisecond(val), self, val),
            Self::Microseconds => to_jiff_datetime!(Timestamp::from_microsecond(val), self, val),
            Self::Nanoseconds => {
                to_jiff_datetime!(Timestamp::from_nanosecond(val as i128), self, val)
            }
            _ => Err(NumpyDateTimeError::UnsupportedUnit(*self)),
        }
        .map(|dt| NumpyDatetime64Repr { dt, opts })
    }
}

struct NumpyDatetime64Array<'a> {
    data: &'a [i64],
    unit: NumpyDatetimeUnit,
    opts: Opt,
}

impl<'a> NumpyDatetime64Array<'a> {
    fn new(data: &'a [i64], unit: NumpyDatetimeUnit, opts: Opt) -> Self {
        Self { data, unit, opts }
    }
}

impl<'a> Serialize for NumpyDatetime64Array<'a> {
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
                .map_err(NumpyDateTimeError::into_serde_err)?;
            seq.serialize_element(&dt).unwrap();
        }
        seq.end()
    }
}

#[repr(C)]
pub struct NumpyDatetime64 {
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: i64,
}

macro_rules! forward_inner {
    ($meth: ident, $ty: ident) => {
        fn $meth(&self) -> $ty {
            self.dt.$meth() as $ty
        }
    };
}

struct NumpyDatetime64Repr {
    dt: DateTime,
    opts: Opt,
}

impl DateTimeLike for NumpyDatetime64Repr {
    forward_inner!(year, i32);
    forward_inner!(month, u8);
    forward_inner!(day, u8);
    forward_inner!(hour, u8);
    forward_inner!(minute, u8);
    forward_inner!(second, u8);

    fn nanosecond(&self) -> u32 {
        self.dt.subsec_nanosecond() as u32
    }

    fn microsecond(&self) -> u32 {
        self.nanosecond() / 1_000
    }

    fn has_tz(&self) -> bool {
        false
    }

    fn slow_offset(&self) -> Result<Offset, DateTimeError> {
        unreachable!()
    }

    fn offset(&self) -> Result<Offset, DateTimeError> {
        Ok(Offset::default())
    }
}

impl Serialize for NumpyDatetime64Repr {
    #[cold]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = SmallFixedBuffer::new();
        let _ = self.write_buf(&mut buf, self.opts);
        serializer.collect_str(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}
