use crate::opt::*;
use crate::serialize::datetimelike::{DateTimeBuffer, DateTimeError, DateTimeLike, Offset};
use crate::typeref::{ARRAY_STRUCT_STR, DESCR_STR, DTYPE_STR, NUMPY_TYPES};
use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use pyo3_ffi::*;
use serde::ser::{self, Serialize, SerializeSeq, Serializer};
use std::convert::TryInto;
use std::fmt;
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
            || ob_type == scalar_types.bool_
            || ob_type == scalar_types.datetime64
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
    DATETIME64(NumpyDatetimeUnit),
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
    fn find(array: *mut PyArrayInterface, ptr: *mut PyObject) -> Option<ItemType> {
        match unsafe { ((*array).typekind, (*array).itemsize) } {
            (098, 1) => Some(ItemType::BOOL),
            (077, 8) => {
                let unit = NumpyDatetimeUnit::from_pyobject(ptr);
                Some(ItemType::DATETIME64(unit))
            }
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
    opts: Opt,
}

impl<'a> NumpyArray {
    #[inline(never)]
    pub fn new(ptr: *mut PyObject, opts: Opt) -> Result<Self, PyArrayError> {
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
                ffi!(Py_DECREF(capsule));
                return Err(PyArrayError::UnsupportedDataType);
            }
            match ItemType::find(array, ptr) {
                None => Err(PyArrayError::UnsupportedDataType),
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

    fn from_parent(&self, position: Vec<isize>, num_children: usize) -> Self {
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
            ffi!(Py_DECREF(self.array as *mut pyo3_ffi::PyObject));
            ffi!(Py_DECREF(self.capsule as *mut pyo3_ffi::PyObject));
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
                    ItemType::DATETIME64(unit) => {
                        let slice: &[i64] = slice!(data_ptr as *const i64, num_items);
                        for &each in slice.iter() {
                            let dt = unit
                                .datetime(each, self.opts)
                                .map_err(NumpyDateTimeError::into_serde_err)?;
                            seq.serialize_element(&dt).unwrap();
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
    obj: f32,
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
    obj: f64,
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
    obj: i8,
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
    obj: i32,
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
    obj: i64,
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
    obj: u8,
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
    obj: u32,
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
    obj: u64,
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
    obj: u8,
}

impl<'p> Serialize for DataTypeBOOL {
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

impl<'p> Serialize for NumpyScalar {
    #[inline(never)]
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
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: i32,
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
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: i64,
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
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: u8,
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
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: u32,
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
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: u64,
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
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: f32,
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
    ob_refcnt: Py_ssize_t,
    ob_type: *mut PyTypeObject,
    value: f64,
}

impl<'p> Serialize for NumpyFloat64 {
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

impl<'p> Serialize for NumpyBool {
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
#[derive(Clone, Copy, Debug)]
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
    #[cfg_attr(feature = "unstable-simd", optimize(size))]
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

#[derive(Clone, Copy, Debug)]
enum NumpyDateTimeError {
    UnsupportedUnit(NumpyDatetimeUnit),
    Unrepresentable { unit: NumpyDatetimeUnit, val: i64 },
}

impl NumpyDateTimeError {
    #[cold]
    #[cfg_attr(feature = "unstable-simd", optimize(size))]
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
    fn from_pyobject(ptr: *mut PyObject) -> Self {
        let dtype = ffi!(PyObject_GetAttr(ptr, DTYPE_STR));
        let descr = ffi!(PyObject_GetAttr(dtype, DESCR_STR));
        ffi!(Py_DECREF(dtype));
        let el0 = ffi!(PyList_GET_ITEM(descr, 0));
        ffi!(Py_DECREF(descr));
        let descr_str = ffi!(PyTuple_GET_ITEM(el0, 1));
        let mut str_size: pyo3_ffi::Py_ssize_t = 0;
        let uni = crate::unicode::read_utf8_from_str(descr_str, &mut str_size);
        if str_size < 5 {
            return Self::NaT;
        }
        let fmt = str_from_slice!(uni, str_size);
        // unit descriptions are found at
        // https://github.com/numpy/numpy/blob/b235f9e701e14ed6f6f6dcba885f7986a833743f/numpy/core/src/multiarray/datetime.c#L79-L96.
        match &fmt[4..fmt.len() - 1] {
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
        }
    }

    /// Return a `NumpyDatetime64Repr` for a value in array with this unit.
    ///
    /// Returns an `Err(NumpyDateTimeError)` if the value is invalid for this unit.
    fn datetime(&self, val: i64, opts: Opt) -> Result<NumpyDatetime64Repr, NumpyDateTimeError> {
        match self {
            Self::Years => Ok(NaiveDate::from_ymd(
                (val + 1970)
                    .try_into()
                    .map_err(|_| NumpyDateTimeError::Unrepresentable { unit: *self, val })?,
                1,
                1,
            )
            .and_hms(0, 0, 0)),
            Self::Months => Ok(NaiveDate::from_ymd(
                (val / 12 + 1970)
                    .try_into()
                    .map_err(|_| NumpyDateTimeError::Unrepresentable { unit: *self, val })?,
                (val % 12 + 1)
                    .try_into()
                    .map_err(|_| NumpyDateTimeError::Unrepresentable { unit: *self, val })?,
                1,
            )
            .and_hms(0, 0, 0)),
            Self::Weeks => Ok(NaiveDateTime::from_timestamp(val * 7 * 24 * 60 * 60, 0)),
            Self::Days => Ok(NaiveDateTime::from_timestamp(val * 24 * 60 * 60, 0)),
            Self::Hours => Ok(NaiveDateTime::from_timestamp(val * 60 * 60, 0)),
            Self::Minutes => Ok(NaiveDateTime::from_timestamp(val * 60, 0)),
            Self::Seconds => Ok(NaiveDateTime::from_timestamp(val, 0)),
            Self::Milliseconds => Ok(NaiveDateTime::from_timestamp(
                val / 1_000,
                (val % 1_000 * 1_000_000)
                    .try_into()
                    .map_err(|_| NumpyDateTimeError::Unrepresentable { unit: *self, val })?,
            )),
            Self::Microseconds => Ok(NaiveDateTime::from_timestamp(
                val / 1_000_000,
                (val % 1_000_000 * 1_000)
                    .try_into()
                    .map_err(|_| NumpyDateTimeError::Unrepresentable { unit: *self, val })?,
            )),
            Self::Nanoseconds => Ok(NaiveDateTime::from_timestamp(
                val / 1_000_000_000,
                (val % 1_000_000_000)
                    .try_into()
                    .map_err(|_| NumpyDateTimeError::Unrepresentable { unit: *self, val })?,
            )),
            _ => Err(NumpyDateTimeError::UnsupportedUnit(*self)),
        }
        .map(|dt| NumpyDatetime64Repr { dt, opts })
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
    dt: NaiveDateTime,
    opts: Opt,
}

impl DateTimeLike for NumpyDatetime64Repr {
    forward_inner!(year, i32);
    forward_inner!(month, u8);
    forward_inner!(day, u8);
    forward_inner!(hour, u8);
    forward_inner!(minute, u8);
    forward_inner!(second, u8);
    forward_inner!(nanosecond, u32);

    fn millisecond(&self) -> u32 {
        self.nanosecond() / 1_000_000
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buf = DateTimeBuffer::new();
        self.write_buf(&mut buf, self.opts).unwrap();
        serializer.collect_str(str_from_slice!(buf.as_ptr(), buf.len()))
    }
}
