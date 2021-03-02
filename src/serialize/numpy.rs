use crate::typeref::{ARRAY_STRUCT_STR, NUMPY_TYPES};
use pyo3::ffi::*;
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ops::DerefMut;
use std::os::raw::{c_char, c_int, c_void};

macro_rules! slice {
    ($ptr:expr, $size:expr) => {
        unsafe { std::slice::from_raw_parts($ptr, $size) }
    };
}

pub fn is_numpy_scalar(ob_type: *mut PyTypeObject) -> bool {
    if unsafe { NUMPY_TYPES.is_none() } {
        false
    } else {
        let scalar_types = unsafe { NUMPY_TYPES.as_ref().unwrap() };
        ob_type == scalar_types.float64
            || ob_type == scalar_types.float32
            || ob_type == scalar_types.int64
            || ob_type == scalar_types.int32
            || ob_type == scalar_types.int8
            || ob_type == scalar_types.uint64
            || ob_type == scalar_types.uint32
            || ob_type == scalar_types.uint8
    }
}

pub fn is_numpy_array(ob_type: *mut PyTypeObject) -> bool {
    if unsafe { NUMPY_TYPES.is_none() } {
        false
    } else {
        unsafe { ob_type == NUMPY_TYPES.as_ref().unwrap().array }
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
    F32,
    F64,
    I8,
    I32,
    I64,
    U8,
    U32,
    U64,
}

impl ItemType {
    fn find(array: *mut PyArrayInterface) -> Option<ItemType> {
        match unsafe { ((*array).typekind, (*array).itemsize) } {
            (098, 1) => Some(ItemType::BOOL),
            (102, 4) => Some(ItemType::F32),
            (102, 8) => Some(ItemType::F64),
            (105, 1) => Some(ItemType::I8),
            (105, 4) => Some(ItemType::I32),
            (105, 8) => Some(ItemType::I64),
            (117, 1) => Some(ItemType::U8),
            (117, 4) => Some(ItemType::U32),
            (117, 8) => Some(ItemType::U64),
            _ => None,
        }
    }
}
pub enum PyArrayError {
    Malformed,
    NotContiguous,
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
}

impl<'a> NumpyArray {
    #[inline(never)]
    pub fn new(ptr: *mut PyObject) -> Result<Self, PyArrayError> {
        let capsule = ffi!(PyObject_GetAttr(ptr, ARRAY_STRUCT_STR));
        let array = unsafe { (*(capsule as *mut PyCapsule)).pointer as *mut PyArrayInterface };
        if unsafe { (*array).two != 2 } {
            ffi!(Py_DECREF(capsule));
            Err(PyArrayError::Malformed)
        } else if unsafe { (*array).flags } & 0x1 != 0x1 {
            ffi!(Py_DECREF(capsule));
            Err(PyArrayError::NotContiguous)
        } else {
            let num_dimensions = unsafe { (*array).nd as usize };
            if num_dimensions == 0 {
                return Err(PyArrayError::UnsupportedDataType);
            }
            match ItemType::find(array) {
                None => Err(PyArrayError::UnsupportedDataType),
                Some(kind) => {
                    let mut pyarray = NumpyArray {
                        array: array,
                        position: vec![0; num_dimensions],
                        children: Vec::with_capacity(num_dimensions),
                        depth: 0,
                        capsule: capsule as *mut PyCapsule,
                        kind: kind,
                    };
                    if pyarray.dimensions() > 1 {
                        pyarray.build();
                    }
                    Ok(pyarray)
                }
            }
        }
    }

    fn from_parent(&self, position: Vec<isize>, num_children: usize) -> Self {
        let mut arr = NumpyArray {
            array: self.array,
            position: position,
            children: Vec::with_capacity(num_children),
            depth: self.depth + 1,
            capsule: self.capsule,
            kind: self.kind,
        };
        arr.build();
        arr
    }

    fn build(&mut self) {
        if self.depth < self.dimensions() - 1 {
            for i in 0..=self.shape()[self.depth] - 1 {
                let mut position: Vec<isize> = self.position.iter().copied().collect();
                position[self.depth] = i;
                let num_children: usize;
                if self.depth < self.dimensions() - 2 {
                    num_children = self.shape()[self.depth + 1] as usize;
                } else {
                    num_children = 0;
                }
                self.children.push(self.from_parent(position, num_children))
            }
        }
    }

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
            ffi!(Py_XDECREF(self.capsule as *mut pyo3::ffi::PyObject))
        }
    }
}

impl<'p> Serialize for NumpyArray {
    #[inline(never)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();

        if self.depth >= self.shape().len() || self.shape()[self.depth] != 0 {
            if !self.children.is_empty() {
                for child in &self.children {
                    seq.serialize_element(child).unwrap();
                }
            } else {
                let data_ptr = self.data();
                let num_items = self.num_items();
                match self.kind {
                    ItemType::F64 => {
                        let slice: &[f64] = slice!(data_ptr as *const f64, num_items);
                        for &each in slice.iter() {
                            seq.serialize_element(&DataTypeF64 { obj: each }).unwrap();
                        }
                    }
                    ItemType::F32 => {
                        let slice: &[f32] = slice!(data_ptr as *const f32, num_items);
                        for &each in slice.iter() {
                            seq.serialize_element(&DataTypeF32 { obj: each }).unwrap();
                        }
                    }
                    ItemType::I64 => {
                        let slice: &[i64] = slice!(data_ptr as *const i64, num_items);
                        for &each in slice.iter() {
                            seq.serialize_element(&DataTypeI64 { obj: each }).unwrap();
                        }
                    }
                    ItemType::I32 => {
                        let slice: &[i32] = slice!(data_ptr as *const i32, num_items);
                        for &each in slice.iter() {
                            seq.serialize_element(&DataTypeI32 { obj: each }).unwrap();
                        }
                    }
                    ItemType::I8 => {
                        let slice: &[i8] = slice!(data_ptr as *const i8, num_items);
                        for &each in slice.iter() {
                            seq.serialize_element(&DataTypeI8 { obj: each }).unwrap();
                        }
                    }
                    ItemType::U8 => {
                        let slice: &[u8] = slice!(data_ptr as *const u8, num_items);
                        for &each in slice.iter() {
                            seq.serialize_element(&DataTypeU8 { obj: each }).unwrap();
                        }
                    }
                    ItemType::U32 => {
                        let slice: &[u32] = slice!(data_ptr as *const u32, num_items);
                        for &each in slice.iter() {
                            seq.serialize_element(&DataTypeU32 { obj: each }).unwrap();
                        }
                    }
                    ItemType::U64 => {
                        let slice: &[u64] = slice!(data_ptr as *const u64, num_items);
                        for &each in slice.iter() {
                            seq.serialize_element(&DataTypeU64 { obj: each }).unwrap();
                        }
                    }
                    ItemType::BOOL => {
                        let slice: &[u8] = slice!(data_ptr as *const u8, num_items);
                        for &each in slice.iter() {
                            seq.serialize_element(&DataTypeBOOL { obj: each }).unwrap();
                        }
                    }
                }
            }
        }
        seq.end()
    }
}

#[repr(transparent)]
struct DataTypeF32 {
    pub obj: f32,
}

impl<'p> Serialize for DataTypeF32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f32(self.obj)
    }
}

#[repr(transparent)]
pub struct DataTypeF64 {
    pub obj: f64,
}

impl<'p> Serialize for DataTypeF64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.obj)
    }
}

#[repr(transparent)]
pub struct DataTypeI8 {
    pub obj: i8,
}

impl<'p> Serialize for DataTypeI8 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i8(self.obj)
    }
}

#[repr(transparent)]
pub struct DataTypeI32 {
    pub obj: i32,
}

impl<'p> Serialize for DataTypeI32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.obj)
    }
}

#[repr(transparent)]
pub struct DataTypeI64 {
    pub obj: i64,
}

impl<'p> Serialize for DataTypeI64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.obj)
    }
}

#[repr(transparent)]
pub struct DataTypeU8 {
    pub obj: u8,
}

impl<'p> Serialize for DataTypeU8 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.obj)
    }
}

#[repr(transparent)]
pub struct DataTypeU32 {
    pub obj: u32,
}

impl<'p> Serialize for DataTypeU32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.obj)
    }
}

#[repr(transparent)]
pub struct DataTypeU64 {
    pub obj: u64,
}

impl<'p> Serialize for DataTypeU64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.obj)
    }
}

#[repr(transparent)]
pub struct DataTypeBOOL {
    pub obj: u8,
}

impl<'p> Serialize for DataTypeBOOL {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(self.obj == 1)
    }
}

#[repr(transparent)]
pub struct NumpyScalar {
    pub ptr: *mut pyo3::ffi::PyObject,
}

impl NumpyScalar {
    pub fn new(ptr: *mut PyObject) -> Self {
        NumpyScalar { ptr }
    }
}

impl<'p> Serialize for NumpyScalar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unsafe {
            let ob_type = ob_type!(self.ptr);
            let scalar_types = NUMPY_TYPES.deref_mut().as_ref().unwrap();
            if ob_type == scalar_types.float64 {
                (*(self.ptr as *mut NumpyFloat64)).serialize(serializer)
            } else if ob_type == scalar_types.float32 {
                (*(self.ptr as *mut NumpyFloat32)).serialize(serializer)
            } else if ob_type == scalar_types.int64 {
                (*(self.ptr as *mut NumpyInt64)).serialize(serializer)
            } else if ob_type == scalar_types.int32 {
                (*(self.ptr as *mut NumpyInt32)).serialize(serializer)
            } else if ob_type == scalar_types.int8 {
                (*(self.ptr as *mut NumpyInt8)).serialize(serializer)
            } else if ob_type == scalar_types.uint64 {
                (*(self.ptr as *mut NumpyUint64)).serialize(serializer)
            } else if ob_type == scalar_types.uint32 {
                (*(self.ptr as *mut NumpyUint32)).serialize(serializer)
            } else if ob_type == scalar_types.uint8 {
                (*(self.ptr as *mut NumpyUint8)).serialize(serializer)
            } else {
                unreachable!()
            }
        }
    }
}

#[repr(C)]
pub struct NumpyInt8 {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub value: i8,
}

impl<'p> Serialize for NumpyInt8 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i8(self.value)
    }
}

#[repr(C)]
pub struct NumpyInt32 {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub value: i32,
}

impl<'p> Serialize for NumpyInt32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.value)
    }
}

#[repr(C)]
pub struct NumpyInt64 {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub value: i64,
}

impl<'p> Serialize for NumpyInt64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.value)
    }
}

#[repr(C)]
pub struct NumpyUint8 {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub value: u8,
}

impl<'p> Serialize for NumpyUint8 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.value)
    }
}

#[repr(C)]
pub struct NumpyUint32 {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub value: u32,
}

impl<'p> Serialize for NumpyUint32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.value)
    }
}

#[repr(C)]
pub struct NumpyUint64 {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub value: u64,
}

impl<'p> Serialize for NumpyUint64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.value)
    }
}

#[repr(C)]
pub struct NumpyFloat32 {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub value: f32,
}

impl<'p> Serialize for NumpyFloat32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f32(self.value)
    }
}

#[repr(C)]
pub struct NumpyFloat64 {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub value: f64,
}

impl<'p> Serialize for NumpyFloat64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.value)
    }
}
