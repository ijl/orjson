// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::{
    Opt, PASSTHROUGH_DATACLASS, PASSTHROUGH_DATETIME, PASSTHROUGH_SUBCLASS, SERIALIZE_NUMPY,
};
use crate::serialize::per_type::{is_numpy_array, is_numpy_scalar};
use crate::typeref::{
    BOOL_TYPE, DATACLASS_FIELDS_STR, DATETIME_TYPE, DATE_TYPE, DICT_TYPE, ENUM_TYPE, FLOAT_TYPE,
    FRAGMENT_TYPE, INT_TYPE, LIST_TYPE, NONE_TYPE, STR_TYPE, TIME_TYPE, TUPLE_TYPE, UUID_TYPE,
};

#[repr(u32)]
pub enum ObType {
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

pub fn pyobject_to_obtype(obj: *mut pyo3_ffi::PyObject, opts: Opt) -> ObType {
    let ob_type = ob_type!(obj);
    if is_class_by_type!(ob_type, STR_TYPE) {
        ObType::Str
    } else if is_class_by_type!(ob_type, INT_TYPE) {
        ObType::Int
    } else if is_class_by_type!(ob_type, BOOL_TYPE) {
        ObType::Bool
    } else if is_class_by_type!(ob_type, NONE_TYPE) {
        ObType::None
    } else if is_class_by_type!(ob_type, FLOAT_TYPE) {
        ObType::Float
    } else if is_class_by_type!(ob_type, LIST_TYPE) {
        ObType::List
    } else if is_class_by_type!(ob_type, DICT_TYPE) {
        ObType::Dict
    } else if is_class_by_type!(ob_type, DATETIME_TYPE) && opt_disabled!(opts, PASSTHROUGH_DATETIME)
    {
        ObType::Datetime
    } else {
        pyobject_to_obtype_unlikely(ob_type, opts)
    }
}

#[cfg_attr(feature = "optimize", optimize(size))]
#[inline(never)]
pub fn pyobject_to_obtype_unlikely(ob_type: *mut pyo3_ffi::PyTypeObject, opts: Opt) -> ObType {
    if is_class_by_type!(ob_type, UUID_TYPE) {
        return ObType::Uuid;
    } else if is_class_by_type!(ob_type, TUPLE_TYPE) {
        return ObType::Tuple;
    } else if is_class_by_type!(ob_type, FRAGMENT_TYPE) {
        return ObType::Fragment;
    }

    if opt_disabled!(opts, PASSTHROUGH_DATETIME) {
        if is_class_by_type!(ob_type, DATE_TYPE) {
            return ObType::Date;
        } else if is_class_by_type!(ob_type, TIME_TYPE) {
            return ObType::Time;
        }
    }

    if opt_disabled!(opts, PASSTHROUGH_SUBCLASS) {
        if is_subclass_by_flag!(ob_type, Py_TPFLAGS_UNICODE_SUBCLASS) {
            return ObType::StrSubclass;
        } else if is_subclass_by_flag!(ob_type, Py_TPFLAGS_LONG_SUBCLASS) {
            return ObType::Int;
        } else if is_subclass_by_flag!(ob_type, Py_TPFLAGS_LIST_SUBCLASS) {
            return ObType::List;
        } else if is_subclass_by_flag!(ob_type, Py_TPFLAGS_DICT_SUBCLASS) {
            return ObType::Dict;
        }
    }

    if is_subclass_by_type!(ob_type, ENUM_TYPE) {
        return ObType::Enum;
    }

    if opt_disabled!(opts, PASSTHROUGH_DATACLASS) && pydict_contains!(ob_type, DATACLASS_FIELDS_STR)
    {
        return ObType::Dataclass;
    }

    if unlikely!(opt_enabled!(opts, SERIALIZE_NUMPY)) {
        if is_numpy_scalar(ob_type) {
            return ObType::NumpyScalar;
        } else if is_numpy_array(ob_type) {
            return ObType::NumpyArray;
        }
    }

    ObType::Unknown
}
