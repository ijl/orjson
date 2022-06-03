// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use ahash::RandomState;
use once_cell::unsync::Lazy;
use pyo3_ffi::*;
use std::os::raw::c_char;
use std::ptr::NonNull;
use std::sync::Once;

pub struct NumpyTypes {
    pub array: *mut PyTypeObject,
    pub float64: *mut PyTypeObject,
    pub float32: *mut PyTypeObject,
    pub int64: *mut PyTypeObject,
    pub int32: *mut PyTypeObject,
    pub int8: *mut PyTypeObject,
    pub uint64: *mut PyTypeObject,
    pub uint32: *mut PyTypeObject,
    pub uint8: *mut PyTypeObject,
    pub bool_: *mut PyTypeObject,
    pub datetime64: *mut PyTypeObject,
}

pub static mut DEFAULT: *mut PyObject = 0 as *mut PyObject;
pub static mut OPTION: *mut PyObject = 0 as *mut PyObject;

pub static mut NONE: *mut PyObject = 0 as *mut PyObject;
pub static mut TRUE: *mut PyObject = 0 as *mut PyObject;
pub static mut FALSE: *mut PyObject = 0 as *mut PyObject;

pub static mut BYTES_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut BYTEARRAY_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut MEMORYVIEW_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut STR_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut INT_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut BOOL_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut NONE_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut FLOAT_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut LIST_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut DICT_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut DATETIME_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut DATE_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut TIME_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut TUPLE_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut UUID_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut ENUM_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;

#[cfg(Py_3_9)]
pub static mut ZONEINFO_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;

pub static mut NUMPY_TYPES: Lazy<Option<NumpyTypes>> = Lazy::new(|| unsafe { load_numpy_types() });
pub static mut FIELD_TYPE: Lazy<NonNull<PyObject>> = Lazy::new(|| unsafe { look_up_field_type() });

pub static mut INT_ATTR_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut UTCOFFSET_METHOD_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut NORMALIZE_METHOD_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut CONVERT_METHOD_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut EMPTY_UNICODE: *mut PyObject = 0 as *mut PyObject;
pub static mut DST_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut DICT_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut DATACLASS_FIELDS_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut SLOTS_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut FIELD_TYPE_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut ARRAY_STRUCT_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut DTYPE_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut DESCR_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut VALUE_STR: *mut PyObject = 0 as *mut PyObject;

pub static mut STR_HASH_FUNCTION: Option<hashfunc> = None;

pub static mut HASH_BUILDER: Lazy<ahash::RandomState> = Lazy::new(|| unsafe {
    RandomState::with_seeds(
        VALUE_STR as u64,
        DICT_TYPE as u64,
        STR_TYPE as u64,
        BYTES_TYPE as u64,
    )
});
#[cfg(feature = "yyjson")]
pub const YYJSON_BUFFER_SIZE: usize = 1024 * 1024 * 8;

#[cfg(feature = "yyjson")]
pub static mut YYJSON_ALLOC: Lazy<crate::yyjson::yyjson_alc> = Lazy::new(|| unsafe {
    let buffer = Box::<[u8; YYJSON_BUFFER_SIZE]>::new([0; YYJSON_BUFFER_SIZE]);
    let mut alloc = crate::yyjson::yyjson_alc {
        malloc: None,
        realloc: None,
        free: None,
        ctx: std::ptr::null_mut(),
    };
    unsafe {
        crate::yyjson::yyjson_alc_pool_init(
            &mut alloc,
            Box::into_raw(buffer) as *mut std::os::raw::c_void,
            YYJSON_BUFFER_SIZE as u64,
        )
    };
    alloc
});

#[allow(non_upper_case_globals)]
pub static mut JsonEncodeError: *mut PyObject = 0 as *mut PyObject;
#[allow(non_upper_case_globals)]
pub static mut JsonDecodeError: *mut PyObject = 0 as *mut PyObject;

static INIT: Once = Once::new();

#[cold]
#[cfg_attr(feature = "unstable-simd", optimize(size))]
pub fn init_typerefs() {
    INIT.call_once(|| unsafe {
        assert!(crate::deserialize::KEY_MAP
            .set(crate::deserialize::KeyMap::default())
            .is_ok());
        PyDateTime_IMPORT();
        NONE = Py_None();
        TRUE = Py_True();
        FALSE = Py_False();
        EMPTY_UNICODE = PyUnicode_New(0, 255);
        STR_TYPE = (*EMPTY_UNICODE).ob_type;
        STR_HASH_FUNCTION = (*((*EMPTY_UNICODE).ob_type)).tp_hash;
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
        JsonDecodeError = look_up_json_exc();
    });
}

#[cold]
#[cfg_attr(feature = "unstable-simd", optimize(size))]
unsafe fn look_up_json_exc() -> *mut PyObject {
    let module = PyImport_ImportModule("json\0".as_ptr() as *const c_char);
    let module_dict = PyObject_GenericGetDict(module, std::ptr::null_mut());
    let ptr = PyMapping_GetItemString(module_dict, "JSONDecodeError\0".as_ptr() as *const c_char)
        as *mut PyObject;
    let res = pyo3_ffi::PyErr_NewException(
        "orjson.JSONDecodeError\0".as_ptr() as *const c_char,
        ptr,
        std::ptr::null_mut(),
    );
    Py_DECREF(ptr);
    Py_DECREF(module_dict);
    Py_DECREF(module);
    res
}

#[cold]
#[cfg_attr(feature = "unstable-simd", optimize(size))]
unsafe fn look_up_numpy_type(numpy_module: *mut PyObject, np_type: &str) -> *mut PyTypeObject {
    let mod_dict = PyObject_GenericGetDict(numpy_module, std::ptr::null_mut());
    let ptr = PyMapping_GetItemString(mod_dict, np_type.as_ptr() as *const c_char);
    Py_XDECREF(ptr);
    Py_XDECREF(mod_dict);
    ptr as *mut PyTypeObject
}

#[cold]
#[cfg_attr(feature = "unstable-simd", optimize(size))]
unsafe fn load_numpy_types() -> Option<NumpyTypes> {
    let numpy = PyImport_ImportModule("numpy\0".as_ptr() as *const c_char);
    if numpy.is_null() {
        PyErr_Clear();
        return None;
    }

    let types = Some(NumpyTypes {
        array: look_up_numpy_type(numpy, "ndarray\0"),
        float32: look_up_numpy_type(numpy, "float32\0"),
        float64: look_up_numpy_type(numpy, "float64\0"),
        int8: look_up_numpy_type(numpy, "int8\0"),
        int32: look_up_numpy_type(numpy, "int32\0"),
        int64: look_up_numpy_type(numpy, "int64\0"),
        uint32: look_up_numpy_type(numpy, "uint32\0"),
        uint64: look_up_numpy_type(numpy, "uint64\0"),
        uint8: look_up_numpy_type(numpy, "uint8\0"),
        bool_: look_up_numpy_type(numpy, "bool_\0"),
        datetime64: look_up_numpy_type(numpy, "datetime64\0"),
    });
    Py_XDECREF(numpy);
    types
}

#[cold]
#[cfg_attr(feature = "unstable-simd", optimize(size))]
unsafe fn look_up_field_type() -> NonNull<PyObject> {
    let module = PyImport_ImportModule("dataclasses\0".as_ptr() as *const c_char);
    let module_dict = PyObject_GenericGetDict(module, std::ptr::null_mut());
    let ptr = PyMapping_GetItemString(module_dict, "_FIELD\0".as_ptr() as *const c_char)
        as *mut PyTypeObject;
    Py_DECREF(module_dict);
    Py_DECREF(module);
    NonNull::new_unchecked(ptr as *mut PyObject)
}

#[cold]
#[cfg_attr(feature = "unstable-simd", optimize(size))]
unsafe fn look_up_enum_type() -> *mut PyTypeObject {
    let module = PyImport_ImportModule("enum\0".as_ptr() as *const c_char);
    let module_dict = PyObject_GenericGetDict(module, std::ptr::null_mut());
    let ptr = PyMapping_GetItemString(module_dict, "EnumMeta\0".as_ptr() as *const c_char)
        as *mut PyTypeObject;
    Py_DECREF(module_dict);
    Py_DECREF(module);
    ptr
}

#[cold]
#[cfg_attr(feature = "unstable-simd", optimize(size))]
unsafe fn look_up_uuid_type() -> *mut PyTypeObject {
    let uuid_mod = PyImport_ImportModule("uuid\0".as_ptr() as *const c_char);
    let uuid_mod_dict = PyObject_GenericGetDict(uuid_mod, std::ptr::null_mut());
    let uuid = PyMapping_GetItemString(uuid_mod_dict, "NAMESPACE_DNS\0".as_ptr() as *const c_char);
    let ptr = (*uuid).ob_type;
    Py_DECREF(uuid);
    Py_DECREF(uuid_mod_dict);
    Py_DECREF(uuid_mod);
    ptr
}

#[cold]
#[cfg_attr(feature = "unstable-simd", optimize(size))]
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
#[cfg_attr(feature = "unstable-simd", optimize(size))]
unsafe fn look_up_date_type() -> *mut PyTypeObject {
    let date = ((*PyDateTimeAPI()).Date_FromDate)(1, 1, 1, (*(PyDateTimeAPI())).DateType);
    let ptr = (*date).ob_type;
    Py_DECREF(date);
    ptr
}

#[cold]
#[cfg_attr(feature = "unstable-simd", optimize(size))]
unsafe fn look_up_time_type() -> *mut PyTypeObject {
    let time = ((*PyDateTimeAPI()).Time_FromTime)(0, 0, 0, 0, NONE, (*(PyDateTimeAPI())).TimeType);
    let ptr = (*time).ob_type;
    Py_DECREF(time);
    ptr
}

#[cfg(Py_3_9)]
#[cold]
#[cfg_attr(feature = "unstable-simd", optimize(size))]
unsafe fn look_up_zoneinfo_type() -> *mut PyTypeObject {
    let module = PyImport_ImportModule("zoneinfo\0".as_ptr() as *const c_char);
    let module_dict = PyObject_GenericGetDict(module, std::ptr::null_mut());
    let ptr = PyMapping_GetItemString(module_dict, "ZoneInfo\0".as_ptr() as *const c_char)
        as *mut PyTypeObject;
    Py_DECREF(module_dict);
    Py_DECREF(module);
    ptr
}
