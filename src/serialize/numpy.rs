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

#[repr(C)]
pub struct PyCapsule {
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub pointer: *mut c_void,
    pub name: *const c_char,
    pub context: *mut c_void,
    pub destructor: *mut c_void, // should be typedef void (*PyCapsule_Destructor)(PyObject *);
}

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

#[derive(Copy, Clone, PartialEq)]
pub enum ItemType {
    BOOL,
    F32,
    F64,
    I32,
    I64,
    U32,
    U64,
}

pub enum PyArrayError {
    Malformed,
    NotContiguous,
    UnsupportedDataType,
}

pub enum NumpyError {
    NotAvailable,
    InvalidType,
}

// >>> arr = numpy.array([[[1, 2], [3, 4]], [[5, 6], [7, 8]]], numpy.int32)
// >>> arr.ndim
// 3
// >>> arr.shape
// (2, 2, 2)
// >>> arr.strides
// (16, 8, 4)
pub struct PyArray {
    array: *mut PyArrayInterface,
    position: Vec<isize>,
    children: Vec<PyArray>,
    depth: usize,
    capsule: *mut PyCapsule,
}

impl<'a> PyArray {
    #[cold]
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
            let mut pyarray = PyArray {
                array: array,
                position: vec![0; num_dimensions],
                children: Vec::with_capacity(num_dimensions),
                depth: 0,
                capsule: capsule as *mut PyCapsule,
            };
            if pyarray.kind().is_none() {
                Err(PyArrayError::UnsupportedDataType)
            } else {
                if pyarray.dimensions() > 1 {
                    pyarray.build();
                }
                Ok(pyarray)
            }
        }
    }

    fn from_parent(&self, position: Vec<isize>, num_children: usize) -> Self {
        let mut arr = PyArray {
            array: self.array,
            position: position,
            children: Vec::with_capacity(num_children),
            depth: self.depth + 1,
            capsule: self.capsule,
        };
        arr.build();
        arr
    }

    fn kind(&self) -> Option<ItemType> {
        match unsafe { ((*self.array).typekind, (*self.array).itemsize) } {
            (098, 1) => Some(ItemType::BOOL),
            (102, 4) => Some(ItemType::F32),
            (102, 8) => Some(ItemType::F64),
            (105, 4) => Some(ItemType::I32),
            (105, 8) => Some(ItemType::I64),
            (117, 4) => Some(ItemType::U32),
            (117, 8) => Some(ItemType::U64),
            _ => None,
        }
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

    fn data(&self) -> *mut c_void {
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

impl Drop for PyArray {
    fn drop(&mut self) {
        if self.depth == 0 {
            ffi!(Py_XDECREF(self.capsule as *mut pyo3::ffi::PyObject))
        }
    }
}

impl<'p> Serialize for PyArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None).unwrap();
        if !self.children.is_empty() {
            for child in &self.children {
                seq.serialize_element(child).unwrap();
            }
        } else {
            let data_ptr = self.data();
            let num_items = self.num_items();
            match self.kind().unwrap() {
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
                ItemType::U64 => {
                    let slice: &[u64] = slice!(data_ptr as *const u64, num_items);
                    for &each in slice.iter() {
                        seq.serialize_element(&DataTypeU64 { obj: each }).unwrap();
                    }
                }
                ItemType::U32 => {
                    let slice: &[u32] = slice!(data_ptr as *const u32, num_items);
                    for &each in slice.iter() {
                        seq.serialize_element(&DataTypeU32 { obj: each }).unwrap();
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

#[repr(C)]
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
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

pub fn is_numpy_scalar(ob_type: *mut PyTypeObject) -> bool {
    let available_types;
    unsafe {
        match NUMPY_TYPES.deref_mut() {
            Some(v) => available_types = v,
            _ => return false,
        }
    }

    let numpy_scalars = [
        available_types.float32,
        available_types.float64,
        available_types.int32,
        available_types.int64,
        available_types.uint32,
        available_types.uint64,
    ];
    numpy_scalars.contains(&ob_type)
}

pub fn is_numpy_array(ob_type: *mut PyTypeObject) -> bool {
    let available_types;
    unsafe {
        match NUMPY_TYPES.deref_mut() {
            Some(v) => available_types = v,
            _ => return false,
        }
    }
    available_types.array == ob_type
}

// pub fn serialize_numpy_scalar<S>(obj: *mut pyo3::ffi::PyObject, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
//     {
//         let ob_type = ob_type!(obj);
//         let numpy = match ob_type {

//         }
//     }

pub enum NumpyObjects {
    Float32(NumpyFloat32),
    Float64(NumpyFloat64),
    Int32(NumpyInt32),
    Int64(NumpyInt64),
    Uint32(NumpyUint32),
    Uint64(NumpyUint64),
}

pub fn pyobj_to_numpy_obj(obj: *mut pyo3::ffi::PyObject) -> Result<NumpyObjects, NumpyError> {
    let available_types;
    unsafe {
        match NUMPY_TYPES.deref_mut() {
            Some(v) => available_types = v,
            _ => return Err(NumpyError::NotAvailable),
        }
    }

    let ob_type = ob_type!(obj);

    unsafe {
        if ob_type == available_types.float32 {
            return Ok(NumpyObjects::Float32(*(obj as *mut NumpyFloat32)));
        } else if ob_type == available_types.float64 {
            return Ok(NumpyObjects::Float64(*(obj as *mut NumpyFloat64)));
        } else if ob_type == available_types.int32 {
            return Ok(NumpyObjects::Int32(*(obj as *mut NumpyInt32)));
        } else if ob_type == available_types.int64 {
            return Ok(NumpyObjects::Int64(*(obj as *mut NumpyInt64)));
        } else if ob_type == available_types.uint32 {
            return Ok(NumpyObjects::Uint32(*(obj as *mut NumpyUint32)));
        } else if ob_type == available_types.uint64 {
            return Ok(NumpyObjects::Uint64(*(obj as *mut NumpyUint64)));
        } else {
            return Err(NumpyError::InvalidType);
        }
    }
}
