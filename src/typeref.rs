// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::orjson_fragmenttype_new;
use core::ffi::c_char;
#[cfg(feature = "yyjson")]
use core::ffi::c_void;
#[cfg(feature = "yyjson")]
use core::mem::MaybeUninit;
use core::ptr::{null_mut, NonNull};
use once_cell::race::{OnceBool, OnceBox};
use pyo3_ffi::*;
#[cfg(feature = "yyjson")]
use std::cell::UnsafeCell;

pub struct NumpyTypes {
    pub array: *mut PyTypeObject,
    pub float64: *mut PyTypeObject,
    pub float32: *mut PyTypeObject,
    pub float16: *mut PyTypeObject,
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
pub static mut FRAGMENT_TYPE: *mut PyTypeObject = null_mut();

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

#[cfg(feature = "yyjson")]
pub const YYJSON_BUFFER_SIZE: usize = 1024 * 1024 * 8;

#[cfg(feature = "yyjson")]
#[repr(align(64))]
struct YYJSONBuffer(UnsafeCell<MaybeUninit<[u8; YYJSON_BUFFER_SIZE]>>);

#[cfg(feature = "yyjson")]
pub struct YYJSONAlloc {
    pub alloc: crate::ffi::yyjson::yyjson_alc,
    _buffer: Box<YYJSONBuffer>,
}

#[cfg(feature = "yyjson")]
pub static mut YYJSON_ALLOC: OnceBox<YYJSONAlloc> = OnceBox::new();

#[cfg(feature = "yyjson")]
pub fn yyjson_init() -> Box<YYJSONAlloc> {
    // Using unsafe to ensure allocation happens on the heap without going through the stack
    // so we don't stack overflow in debug mode. Once rust-lang/rust#63291 is stable (Box::new_uninit)
    // we can use that instead.
    let layout = std::alloc::Layout::new::<YYJSONBuffer>();
    let buffer = unsafe { Box::from_raw(std::alloc::alloc(layout).cast::<YYJSONBuffer>()) };
    let mut alloc = crate::ffi::yyjson::yyjson_alc {
        malloc: None,
        realloc: None,
        free: None,
        ctx: null_mut(),
    };
    unsafe {
        crate::ffi::yyjson::yyjson_alc_pool_init(
            &mut alloc,
            buffer.0.get().cast::<c_void>(),
            YYJSON_BUFFER_SIZE,
        );
    }
    Box::new(YYJSONAlloc {
        alloc,
        _buffer: buffer,
    })
}

#[allow(non_upper_case_globals)]
pub static mut JsonEncodeError: *mut PyObject = null_mut();
#[allow(non_upper_case_globals)]
pub static mut JsonDecodeError: *mut PyObject = null_mut();

static INIT: OnceBool = OnceBool::new();

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
pub fn init_typerefs() {
    INIT.get_or_init(_init_typerefs_impl);
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
fn _init_typerefs_impl() -> bool {
    unsafe {
        debug_assert!(crate::opt::MAX_OPT < u16::MAX as i32);

        assert!(crate::deserialize::KEY_MAP
            .set(crate::deserialize::KeyMap::default())
            .is_ok());
        FRAGMENT_TYPE = orjson_fragmenttype_new();
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
    };
    true
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_json_exc() -> *mut PyObject {
    let module = PyImport_ImportModule("json\0".as_ptr() as *const c_char);
    let module_dict = PyObject_GenericGetDict(module, null_mut());
    let ptr = PyMapping_GetItemString(module_dict, "JSONDecodeError\0".as_ptr() as *const c_char);
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
unsafe fn look_up_numpy_type(numpy_module_dict: *mut PyObject, np_type: &str) -> *mut PyTypeObject {
    let ptr = PyMapping_GetItemString(numpy_module_dict, np_type.as_ptr() as *const c_char);
    Py_XDECREF(ptr);
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
        let numpy_module_dict = PyObject_GenericGetDict(numpy, null_mut());
        let types = Box::new(NumpyTypes {
            array: look_up_numpy_type(numpy_module_dict, "ndarray\0"),
            float16: look_up_numpy_type(numpy_module_dict, "half\0"),
            float32: look_up_numpy_type(numpy_module_dict, "float32\0"),
            float64: look_up_numpy_type(numpy_module_dict, "float64\0"),
            int8: look_up_numpy_type(numpy_module_dict, "int8\0"),
            int16: look_up_numpy_type(numpy_module_dict, "int16\0"),
            int32: look_up_numpy_type(numpy_module_dict, "int32\0"),
            int64: look_up_numpy_type(numpy_module_dict, "int64\0"),
            uint16: look_up_numpy_type(numpy_module_dict, "uint16\0"),
            uint32: look_up_numpy_type(numpy_module_dict, "uint32\0"),
            uint64: look_up_numpy_type(numpy_module_dict, "uint64\0"),
            uint8: look_up_numpy_type(numpy_module_dict, "uint8\0"),
            bool_: look_up_numpy_type(numpy_module_dict, "bool_\0"),
            datetime64: look_up_numpy_type(numpy_module_dict, "datetime64\0"),
        });
        Py_XDECREF(numpy_module_dict);
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
