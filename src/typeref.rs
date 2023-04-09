// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use ahash::RandomState;
use once_cell::race::OnceBox;
use pyo3_ffi::*;
use std::os::raw::c_char;
use std::ptr::{null_mut, NonNull};
use std::sync::Once;

pub struct NumpyTypes {
    pub array: *mut PyTypeObject,
    pub float64: *mut PyTypeObject,
    pub float32: *mut PyTypeObject,
    pub int64: *mut PyTypeObject,
    pub int32: *mut PyTypeObject,
    pub int16: *mut PyTypeObject,
    pub int8: *mut PyTypeObject,
    pub uint64: *mut PyTypeObject,
    pub uint32: *mut PyTypeObject,
    pub uint16: *mut PyTypeObject,
    pub uint8: *mut PyTypeObject,
    pub bool_: *mut PyTypeObject,
    pub datetime64: *mut PyTypeObject,
}

pub static mut DEFAULT: *mut PyObject = null_mut();
pub static mut OPTION: *mut PyObject = null_mut();

pub static mut NONE: *mut PyObject = null_mut();
pub static mut TRUE: *mut PyObject = null_mut();
pub static mut FALSE: *mut PyObject = null_mut();
pub static mut EMPTY_UNICODE: *mut PyObject = null_mut();

pub static mut BYTES_TYPE: *mut PyTypeObject = null_mut();
pub static mut BYTEARRAY_TYPE: *mut PyTypeObject = null_mut();
pub static mut MEMORYVIEW_TYPE: *mut PyTypeObject = null_mut();
pub static mut STR_TYPE: *mut PyTypeObject = null_mut();
pub static mut INT_TYPE: *mut PyTypeObject = null_mut();
pub static mut BOOL_TYPE: *mut PyTypeObject = null_mut();
pub static mut NONE_TYPE: *mut PyTypeObject = null_mut();
pub static mut FLOAT_TYPE: *mut PyTypeObject = null_mut();
pub static mut LIST_TYPE: *mut PyTypeObject = null_mut();
pub static mut DICT_TYPE: *mut PyTypeObject = null_mut();
pub static mut DATETIME_TYPE: *mut PyTypeObject = null_mut();
pub static mut DATE_TYPE: *mut PyTypeObject = null_mut();
pub static mut TIME_TYPE: *mut PyTypeObject = null_mut();
pub static mut TUPLE_TYPE: *mut PyTypeObject = null_mut();
pub static mut UUID_TYPE: *mut PyTypeObject = null_mut();
pub static mut ENUM_TYPE: *mut PyTypeObject = null_mut();
pub static mut FIELD_TYPE: *mut PyTypeObject = null_mut();

pub static mut NUMPY_TYPES: OnceBox<Option<NonNull<NumpyTypes>>> = OnceBox::new();

#[cfg(Py_3_9)]
pub static mut ZONEINFO_TYPE: *mut PyTypeObject = null_mut();

pub static mut UTCOFFSET_METHOD_STR: *mut PyObject = null_mut();
pub static mut NORMALIZE_METHOD_STR: *mut PyObject = null_mut();
pub static mut CONVERT_METHOD_STR: *mut PyObject = null_mut();
pub static mut DST_STR: *mut PyObject = null_mut();

pub static mut DICT_STR: *mut PyObject = null_mut();
pub static mut DATACLASS_FIELDS_STR: *mut PyObject = null_mut();
pub static mut SLOTS_STR: *mut PyObject = null_mut();
pub static mut FIELD_TYPE_STR: *mut PyObject = null_mut();
pub static mut ARRAY_STRUCT_STR: *mut PyObject = null_mut();
pub static mut DTYPE_STR: *mut PyObject = null_mut();
pub static mut DESCR_STR: *mut PyObject = null_mut();
pub static mut VALUE_STR: *mut PyObject = null_mut();
pub static mut INT_ATTR_STR: *mut PyObject = null_mut();

pub static mut HASH_BUILDER: OnceBox<ahash::RandomState> = OnceBox::new();

pub fn ahash_init() -> Box<ahash::RandomState> {
    unsafe {
        Box::new(RandomState::with_seeds(
            VALUE_STR as u64,
            DICT_TYPE as u64,
            STR_TYPE as u64,
            BYTES_TYPE as u64,
        ))
    }
}

#[cfg(feature = "yyjson")]
pub const YYJSON_BUFFER_SIZE: usize = 1024 * 1024 * 8;

#[cfg(feature = "yyjson")]
pub static mut YYJSON_ALLOC: OnceBox<crate::yyjson::yyjson_alc> = OnceBox::new();

#[cfg(feature = "yyjson")]
pub fn yyjson_init() -> Box<crate::yyjson::yyjson_alc> {
    unsafe {
        let buffer = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(
            YYJSON_BUFFER_SIZE,
            64,
        ));
        let mut alloc = crate::yyjson::yyjson_alc {
            malloc: None,
            realloc: None,
            free: None,
            ctx: null_mut(),
        };
        crate::yyjson::yyjson_alc_pool_init(
            &mut alloc,
            buffer as *mut std::os::raw::c_void,
            YYJSON_BUFFER_SIZE,
        );
        Box::new(alloc)
    }
}

#[allow(non_upper_case_globals)]
pub static mut JsonEncodeError: *mut PyObject = null_mut();
#[allow(non_upper_case_globals)]
pub static mut JsonDecodeError: *mut PyObject = null_mut();

static INIT: Once = Once::new();

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
pub fn init_typerefs() {
    INIT.call_once(|| unsafe {
        assert!(crate::deserialize::KEY_MAP
            .set(crate::deserialize::KeyMap::default())
            .is_ok());
        HASH_BUILDER.get_or_init(ahash_init);
        PyDateTime_IMPORT();
        NONE = Py_None();
        TRUE = Py_True();
        FALSE = Py_False();
        EMPTY_UNICODE = PyUnicode_New(0, 255);
        STR_TYPE = (*EMPTY_UNICODE).ob_type;
        BYTES_TYPE = (*PyBytes_FromStringAndSize("".as_ptr() as *const c_char, 0)).ob_type;

        {
            let bytearray = PyByteArray_FromStringAndSize("".as_ptr() as *const c_char, 0);
            BYTEARRAY_TYPE = (*bytearray).ob_type;

            let memoryview = PyMemoryView_FromObject(bytearray);
            MEMORYVIEW_TYPE = (*memoryview).ob_type;
            Py_DECREF(memoryview);
            Py_DECREF(bytearray);
        }

        DICT_TYPE = (*PyDict_New()).ob_type;
        LIST_TYPE = (*PyList_New(0)).ob_type;
        TUPLE_TYPE = (*PyTuple_New(0)).ob_type;
        NONE_TYPE = (*NONE).ob_type;
        BOOL_TYPE = (*TRUE).ob_type;
        INT_TYPE = (*PyLong_FromLongLong(0)).ob_type;
        FLOAT_TYPE = (*PyFloat_FromDouble(0.0)).ob_type;
        DATETIME_TYPE = look_up_datetime_type();
        DATE_TYPE = look_up_date_type();
        TIME_TYPE = look_up_time_type();
        UUID_TYPE = look_up_uuid_type();
        ENUM_TYPE = look_up_enum_type();
        FIELD_TYPE = look_up_field_type();

        #[cfg(Py_3_9)]
        {
            ZONEINFO_TYPE = look_up_zoneinfo_type();
        }

        INT_ATTR_STR = PyUnicode_InternFromString("int\0".as_ptr() as *const c_char);
        UTCOFFSET_METHOD_STR = PyUnicode_InternFromString("utcoffset\0".as_ptr() as *const c_char);
        NORMALIZE_METHOD_STR = PyUnicode_InternFromString("normalize\0".as_ptr() as *const c_char);
        CONVERT_METHOD_STR = PyUnicode_InternFromString("convert\0".as_ptr() as *const c_char);
        DST_STR = PyUnicode_InternFromString("dst\0".as_ptr() as *const c_char);
        DICT_STR = PyUnicode_InternFromString("__dict__\0".as_ptr() as *const c_char);
        DATACLASS_FIELDS_STR =
            PyUnicode_InternFromString("__dataclass_fields__\0".as_ptr() as *const c_char);
        SLOTS_STR = PyUnicode_InternFromString("__slots__\0".as_ptr() as *const c_char);
        FIELD_TYPE_STR = PyUnicode_InternFromString("_field_type\0".as_ptr() as *const c_char);
        ARRAY_STRUCT_STR =
            PyUnicode_InternFromString("__array_struct__\0".as_ptr() as *const c_char);
        DTYPE_STR = PyUnicode_InternFromString("dtype\0".as_ptr() as *const c_char);
        DESCR_STR = PyUnicode_InternFromString("descr\0".as_ptr() as *const c_char);
        VALUE_STR = PyUnicode_InternFromString("value\0".as_ptr() as *const c_char);
        DEFAULT = PyUnicode_InternFromString("default\0".as_ptr() as *const c_char);
        OPTION = PyUnicode_InternFromString("option\0".as_ptr() as *const c_char);
        JsonEncodeError = pyo3_ffi::PyExc_TypeError;
        Py_INCREF(JsonEncodeError);
        JsonDecodeError = look_up_json_exc();
    });
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_json_exc() -> *mut PyObject {
    let module = PyImport_ImportModule("json\0".as_ptr() as *const c_char);
    let module_dict = PyObject_GenericGetDict(module, null_mut());
    let ptr = PyMapping_GetItemString(module_dict, "JSONDecodeError\0".as_ptr() as *const c_char)
        as *mut PyObject;
    let res = pyo3_ffi::PyErr_NewException(
        "orjson.JSONDecodeError\0".as_ptr() as *const c_char,
        ptr,
        null_mut(),
    );
    Py_DECREF(ptr);
    Py_DECREF(module_dict);
    Py_DECREF(module);
    Py_INCREF(res);
    res
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_numpy_type(numpy_module: *mut PyObject, np_type: &str) -> *mut PyTypeObject {
    let mod_dict = PyObject_GenericGetDict(numpy_module, null_mut());
    let ptr = PyMapping_GetItemString(mod_dict, np_type.as_ptr() as *const c_char);
    Py_XDECREF(ptr);
    Py_XDECREF(mod_dict);
    ptr as *mut PyTypeObject
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
pub fn load_numpy_types() -> Box<Option<NonNull<NumpyTypes>>> {
    unsafe {
        let numpy = PyImport_ImportModule("numpy\0".as_ptr() as *const c_char);
        if numpy.is_null() {
            PyErr_Clear();
            return Box::new(None);
        }
        let types = Box::new(NumpyTypes {
            array: look_up_numpy_type(numpy, "ndarray\0"),
            float32: look_up_numpy_type(numpy, "float32\0"),
            float64: look_up_numpy_type(numpy, "float64\0"),
            int8: look_up_numpy_type(numpy, "int8\0"),
            int16: look_up_numpy_type(numpy, "int16\0"),
            int32: look_up_numpy_type(numpy, "int32\0"),
            int64: look_up_numpy_type(numpy, "int64\0"),
            uint16: look_up_numpy_type(numpy, "uint16\0"),
            uint32: look_up_numpy_type(numpy, "uint32\0"),
            uint64: look_up_numpy_type(numpy, "uint64\0"),
            uint8: look_up_numpy_type(numpy, "uint8\0"),
            bool_: look_up_numpy_type(numpy, "bool_\0"),
            datetime64: look_up_numpy_type(numpy, "datetime64\0"),
        });
        Py_XDECREF(numpy);
        Box::new(Some(nonnull!(Box::<NumpyTypes>::into_raw(types))))
    }
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_field_type() -> *mut PyTypeObject {
    let module = PyImport_ImportModule("dataclasses\0".as_ptr() as *const c_char);
    let module_dict = PyObject_GenericGetDict(module, null_mut());
    let ptr = PyMapping_GetItemString(module_dict, "_FIELD\0".as_ptr() as *const c_char)
        as *mut PyTypeObject;
    Py_DECREF(module_dict);
    Py_DECREF(module);
    ptr
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_enum_type() -> *mut PyTypeObject {
    let module = PyImport_ImportModule("enum\0".as_ptr() as *const c_char);
    let module_dict = PyObject_GenericGetDict(module, null_mut());
    let ptr = PyMapping_GetItemString(module_dict, "EnumMeta\0".as_ptr() as *const c_char)
        as *mut PyTypeObject;
    Py_DECREF(module_dict);
    Py_DECREF(module);
    ptr
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_uuid_type() -> *mut PyTypeObject {
    let uuid_mod = PyImport_ImportModule("uuid\0".as_ptr() as *const c_char);
    let uuid_mod_dict = PyObject_GenericGetDict(uuid_mod, null_mut());
    let uuid = PyMapping_GetItemString(uuid_mod_dict, "NAMESPACE_DNS\0".as_ptr() as *const c_char);
    let ptr = (*uuid).ob_type;
    Py_DECREF(uuid);
    Py_DECREF(uuid_mod_dict);
    Py_DECREF(uuid_mod);
    ptr
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_datetime_type() -> *mut PyTypeObject {
    let datetime = ((*PyDateTimeAPI()).DateTime_FromDateAndTime)(
        1970,
        1,
        1,
        0,
        0,
        0,
        0,
        NONE,
        (*(PyDateTimeAPI())).DateTimeType,
    );
    let ptr = (*datetime).ob_type;
    Py_DECREF(datetime);
    ptr
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_date_type() -> *mut PyTypeObject {
    let date = ((*PyDateTimeAPI()).Date_FromDate)(1, 1, 1, (*(PyDateTimeAPI())).DateType);
    let ptr = (*date).ob_type;
    Py_DECREF(date);
    ptr
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_time_type() -> *mut PyTypeObject {
    let time = ((*PyDateTimeAPI()).Time_FromTime)(0, 0, 0, 0, NONE, (*(PyDateTimeAPI())).TimeType);
    let ptr = (*time).ob_type;
    Py_DECREF(time);
    ptr
}

#[cfg(Py_3_9)]
#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_zoneinfo_type() -> *mut PyTypeObject {
    let module = PyImport_ImportModule("zoneinfo\0".as_ptr() as *const c_char);
    let module_dict = PyObject_GenericGetDict(module, null_mut());
    let ptr = PyMapping_GetItemString(module_dict, "ZoneInfo\0".as_ptr() as *const c_char)
        as *mut PyTypeObject;
    Py_DECREF(module_dict);
    Py_DECREF(module);
    ptr
}
