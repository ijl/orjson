// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2026)

mod array;
mod datetime;
mod scalar;

pub(crate) use array::{NPY_ARRAY_C_CONTIGUOUS, NPY_ARRAY_NOTSWAPPED, PyArrayInterface, PyCapsule};
pub(crate) use datetime::{NumpyDateTimeError, NumpyDatetime64Repr, NumpyDatetimeUnit};
pub(crate) use scalar::{
    NumpyBool, NumpyDatetime64, NumpyFloat16, NumpyFloat32, NumpyFloat64, NumpyInt8, NumpyInt16,
    NumpyInt32, NumpyInt64, NumpyUint8, NumpyUint16, NumpyUint32, NumpyUint64,
};
