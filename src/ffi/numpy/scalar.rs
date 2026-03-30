// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2026)

use crate::ffi::PyObject;

#[repr(C)]
pub(crate) struct NumpyFloat64 {
    head: PyObject,
    pub value: f64,
}

#[repr(C)]
pub(crate) struct NumpyFloat32 {
    head: PyObject,
    pub value: f32,
}

#[repr(C)]
pub(crate) struct NumpyFloat16 {
    head: PyObject,
    pub value: u16,
}

#[repr(C)]
pub(crate) struct NumpyUint64 {
    head: PyObject,
    pub value: u64,
}

#[repr(C)]
pub(crate) struct NumpyUint32 {
    head: PyObject,
    pub value: u32,
}

#[repr(C)]
pub(crate) struct NumpyUint16 {
    head: PyObject,
    pub value: u16,
}

#[repr(C)]
pub(crate) struct NumpyUint8 {
    head: PyObject,
    pub value: u8,
}

#[repr(C)]
pub(crate) struct NumpyInt64 {
    head: PyObject,
    pub value: i64,
}

#[repr(C)]
pub(crate) struct NumpyInt32 {
    head: PyObject,
    pub value: i32,
}

#[repr(C)]
pub(crate) struct NumpyInt16 {
    head: PyObject,
    pub value: i16,
}

#[repr(C)]
pub(crate) struct NumpyInt8 {
    head: PyObject,
    pub value: i8,
}

#[repr(C)]
pub(crate) struct NumpyBool {
    head: PyObject,
    pub value: bool,
}

#[repr(C)]
pub(crate) struct NumpyDatetime64 {
    head: PyObject,
    pub value: i64,
}
