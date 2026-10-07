// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// Copyright ijl (2020-2026), Aviram Hassan (2020)

use crate::opt::{PASSTHROUGH_DATACLASS, PASSTHROUGH_SUBCLASS, SERIALIZE_NUMPY};
use crate::serialize::numpy::{is_numpy_array, is_numpy_scalar};
use crate::typeref::{
    BOOL_TYPE, DATACLASS_FIELDS_STR, DATE_TYPE, DATETIME_TYPE, DICT_TYPE, ENUM_TYPE, FLOAT_TYPE,
    FRAGMENT_TYPE, INT_TYPE, LIST_TYPE, NONE_TYPE, STR_TYPE, TIME_TYPE, TUPLE_TYPE, UUID_TYPE,
};

use crate::ffi::{
    Py_TPFLAGS_DICT_SUBCLASS, Py_TPFLAGS_LIST_SUBCLASS, Py_TPFLAGS_LONG_SUBCLASS,
    Py_TPFLAGS_UNICODE_SUBCLASS, PyObject, PyType_GetFlags, PyTypeRef,
};

pub(crate) enum ObType {
    Str,
    Int,
    Bool,
    None,
    Float,
    List,
    Dict,
    Datetime,
    Date,
    Time,
    Tuple,
    Uuid,
    Dataclass,
    NumpyScalar,
    NumpyArray,
    Enum,
    StrSubclass,
    Fragment,
    Unknown,
}

#[inline(always)]
pub(crate) fn pyobject_to_obtype(ptr: *mut PyObject, opts: u32) -> ObType {
    pyobject_to_obtype_likely(PyTypeRef::from_pyobject(ptr))
        .unwrap_or_else(|| pyobject_to_obtype_unlikely(PyTypeRef::from_pyobject(ptr), opts))
}

#[inline(always)]
pub(crate) fn pyobject_to_obtype_likely(ob: PyTypeRef) -> Option<ObType> {
    unsafe {
        let ob_type = ob.as_ptr();
        if ob_type == STR_TYPE {
            Some(ObType::Str)
        } else if ob_type == INT_TYPE {
            Some(ObType::Int)
        } else if ob_type == BOOL_TYPE {
            Some(ObType::Bool)
        } else if ob_type == NONE_TYPE {
            Some(ObType::None)
        } else if ob_type == FLOAT_TYPE {
            Some(ObType::Float)
        } else if ob_type == LIST_TYPE {
            Some(ObType::List)
        } else if ob_type == DICT_TYPE {
            Some(ObType::Dict)
        } else if ob_type == DATETIME_TYPE {
            Some(ObType::Datetime)
        } else {
            None
        }
    }
}

const TPFLAGS_MASK: core::ffi::c_ulong = Py_TPFLAGS_UNICODE_SUBCLASS
    | Py_TPFLAGS_LONG_SUBCLASS
    | Py_TPFLAGS_LIST_SUBCLASS
    | Py_TPFLAGS_DICT_SUBCLASS;

#[inline(never)]
pub(crate) fn pyobject_to_obtype_unlikely(ob: PyTypeRef, opts: u32) -> ObType {
    unsafe {
        let ob_type = ob.as_ptr();
        if ob_type == UUID_TYPE {
            return ObType::Uuid;
        } else if ob_type == TUPLE_TYPE {
            return ObType::Tuple;
        } else if ob_type == DATE_TYPE {
            return ObType::Date;
        } else if ob_type == TIME_TYPE {
            return ObType::Time;
        } else if ob_type == FRAGMENT_TYPE {
            return ObType::Fragment;
        }

        if opt_disabled!(opts, PASSTHROUGH_SUBCLASS) {
            #[allow(nonstandard_style)]
            match PyType_GetFlags(ob_type) & TPFLAGS_MASK {
                Py_TPFLAGS_UNICODE_SUBCLASS => return ObType::StrSubclass,
                Py_TPFLAGS_LONG_SUBCLASS => return ObType::Int,
                Py_TPFLAGS_LIST_SUBCLASS => return ObType::List,
                Py_TPFLAGS_DICT_SUBCLASS => return ObType::Dict,
                _ => {}
            }
        }

        if core::ptr::eq(
            (*(ob_type.cast::<pyo3_ffi::PyTypeObject>()))
                .ob_base
                .ob_base
                .ob_type,
            ENUM_TYPE,
        ) {
            return ObType::Enum;
        }

        if opt_disabled!(opts, PASSTHROUGH_DATACLASS)
            && pydict_contains!(ob_type, DATACLASS_FIELDS_STR)
        {
            return ObType::Dataclass;
        }

        if opt_enabled!(opts, SERIALIZE_NUMPY) {
            cold_path!();
            if is_numpy_scalar(ob_type) {
                return ObType::NumpyScalar;
            } else if is_numpy_array(ob_type) {
                return ObType::NumpyArray;
            }
        }

        ObType::Unknown
    }
}
