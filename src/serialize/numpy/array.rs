// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// Copyright ijl (2020-2026), Ben Sully (2021)

use super::item::ItemType;
use crate::ffi::{
    NPY_ARRAY_C_CONTIGUOUS, NPY_ARRAY_NOTSWAPPED, NumpyDatetimeUnit, Py_DECREF, PyArrayInterface,
    PyCapsule, PyObject, PyObject_GetAttr,
};
use crate::opt::Opt;
use crate::typeref::ARRAY_STRUCT_STR;
use crate::util::isize_to_usize;
use core::ffi::c_void;

macro_rules! slice {
    ($ptr:expr, $size:expr) => {
        unsafe { core::slice::from_raw_parts($ptr, $size) }
    };
}

pub(crate) enum PyArrayError {
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
pub(crate) struct NumpyArray {
    array: *mut PyArrayInterface,
    unit: Option<NumpyDatetimeUnit>,
    position: Vec<isize>,
    pub children: Vec<NumpyArray>,
    pub depth: usize,
    capsule: *mut PyCapsule,
    pub kind: ItemType,
    pub opts: Opt,
}

impl NumpyArray {
    #[cold]
    #[inline(never)]
    #[cfg_attr(feature = "optimize", optimize(size))]
    pub fn new(ptr: *mut PyObject, opts: Opt) -> Result<Self, PyArrayError> {
        let capsule = unsafe { PyObject_GetAttr(ptr, ARRAY_STRUCT_STR) };
        debug_assert!(!capsule.is_null());
        let array = unsafe {
            (*capsule.cast::<PyCapsule>())
                .pointer
                .cast::<PyArrayInterface>()
        };
        debug_assert!(!array.is_null());
        if unsafe { (*array).two != 2 } {
            unsafe {
                Py_DECREF(capsule);
            }
            Err(PyArrayError::Malformed)
        } else if unsafe { (*array).flags } & NPY_ARRAY_C_CONTIGUOUS != NPY_ARRAY_C_CONTIGUOUS {
            unsafe {
                Py_DECREF(capsule);
            }
            Err(PyArrayError::NotContiguous)
        } else if unsafe { (*array).flags } & NPY_ARRAY_NOTSWAPPED != NPY_ARRAY_NOTSWAPPED {
            unsafe {
                Py_DECREF(capsule);
            }
            Err(PyArrayError::NotNativeEndian)
        } else {
            debug_assert!(unsafe { (*array).nd >= 0 });
            #[allow(clippy::cast_sign_loss)]
            let num_dimensions = unsafe { (*array).nd as usize };
            if num_dimensions == 0 {
                unsafe {
                    Py_DECREF(capsule);
                }
                return Err(PyArrayError::UnsupportedDataType);
            }
            match ItemType::find(array, ptr) {
                None => {
                    unsafe {
                        Py_DECREF(capsule);
                    }
                    Err(PyArrayError::UnsupportedDataType)
                }
                Some(kind) => {
                    let unit = match kind {
                        ItemType::DATETIME64(val) => Some(val),
                        _ => None,
                    };
                    let mut pyarray = NumpyArray {
                        array: array,
                        unit: unit,
                        position: vec![0; num_dimensions],
                        children: Vec::with_capacity(num_dimensions),
                        depth: 0,
                        capsule: capsule.cast::<PyCapsule>(),
                        kind: kind,
                        opts: opts,
                    };
                    if pyarray.dimensions() > 1 {
                        pyarray.build();
                    }
                    Ok(pyarray)
                }
            }
        }
    }

    fn child_from_parent(&self, position: Vec<isize>, num_children: usize) -> Self {
        let mut arr = NumpyArray {
            array: self.array,
            unit: self.unit,
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

    pub fn build(&mut self) {
        if self.depth < self.dimensions() - 1 {
            for i in 0..self.shape()[self.depth] {
                let mut position: Vec<isize> = self.position.clone();
                position[self.depth] = i;
                let num_children: usize = if self.depth < self.dimensions() - 2 {
                    isize_to_usize(self.shape()[self.depth + 1])
                } else {
                    0
                };
                self.children
                    .push(self.child_from_parent(position, num_children));
            }
        }
    }

    #[inline(always)]
    pub fn data(&self) -> *const c_void {
        let offset = self
            .strides()
            .iter()
            .zip(self.position.iter().copied())
            .take(self.depth)
            .map(|(a, b)| a * b)
            .sum::<isize>();
        unsafe { (*self.array).data.offset(offset) }
    }

    pub fn num_items(&self) -> usize {
        isize_to_usize(self.shape()[self.shape().len() - 1])
    }

    pub fn dimensions(&self) -> usize {
        #[allow(clippy::cast_sign_loss)]
        unsafe {
            (*self.array).nd as usize
        }
    }

    pub fn shape(&self) -> &[isize] {
        slice!((*self.array).shape.cast_const(), self.dimensions())
    }

    pub fn strides(&self) -> &[isize] {
        slice!((*self.array).strides.cast_const(), self.dimensions())
    }
}

impl Drop for NumpyArray {
    fn drop(&mut self) {
        if self.depth == 0 {
            unsafe {
                Py_DECREF(self.array.cast::<PyObject>());
                Py_DECREF(self.capsule.cast::<PyObject>());
            };
        }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyF64Array<'a> {
    pub data: &'a [f64],
}

impl<'a> NumpyF64Array<'a> {
    pub const fn new(data: &'a [f64]) -> Self {
        Self { data }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyF32Array<'a> {
    pub data: &'a [f32],
}

impl<'a> NumpyF32Array<'a> {
    pub const fn new(data: &'a [f32]) -> Self {
        Self { data }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyF16Array<'a> {
    pub data: &'a [u16],
}

impl<'a> NumpyF16Array<'a> {
    pub const fn new(data: &'a [u16]) -> Self {
        Self { data }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyU64Array<'a> {
    pub data: &'a [u64],
}

impl<'a> NumpyU64Array<'a> {
    pub const fn new(data: &'a [u64]) -> Self {
        Self { data }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyU32Array<'a> {
    pub data: &'a [u32],
}

impl<'a> NumpyU32Array<'a> {
    pub const fn new(data: &'a [u32]) -> Self {
        Self { data }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyU16Array<'a> {
    pub data: &'a [u16],
}

impl<'a> NumpyU16Array<'a> {
    pub const fn new(data: &'a [u16]) -> Self {
        Self { data }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyU8Array<'a> {
    pub data: &'a [u8],
}

impl<'a> NumpyU8Array<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyI64Array<'a> {
    pub data: &'a [i64],
}

impl<'a> NumpyI64Array<'a> {
    pub const fn new(data: &'a [i64]) -> Self {
        Self { data }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyI32Array<'a> {
    pub data: &'a [i32],
}

impl<'a> NumpyI32Array<'a> {
    pub const fn new(data: &'a [i32]) -> Self {
        Self { data }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyI16Array<'a> {
    pub data: &'a [i16],
}

impl<'a> NumpyI16Array<'a> {
    pub const fn new(data: &'a [i16]) -> Self {
        Self { data }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyI8Array<'a> {
    pub data: &'a [i8],
}

impl<'a> NumpyI8Array<'a> {
    pub const fn new(data: &'a [i8]) -> Self {
        Self { data }
    }
}

#[repr(transparent)]
pub(crate) struct NumpyBoolArray<'a> {
    pub data: &'a [u8],
}

impl<'a> NumpyBoolArray<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

pub(crate) struct NumpyDatetime64Array<'a> {
    pub data: &'a [i64],
    pub unit: NumpyDatetimeUnit,
    pub opts: Opt,
}

impl<'a> NumpyDatetime64Array<'a> {
    pub const fn new(data: &'a [i64], unit: NumpyDatetimeUnit, opts: Opt) -> Self {
        Self { data, unit, opts }
    }
}
