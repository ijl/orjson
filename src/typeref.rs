// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::orjson_fragmenttype_new;
#[cfg(feature = "yyjson")]
use core::ffi::c_void;
use core::ffi::CStr;
#[cfg(feature = "yyjson")]
use core::mem::MaybeUninit;
use core::ptr::{null_mut, NonNull};
use once_cell::race::{OnceBool, OnceBox};

#[cfg(feature = "yyjson")]
use core::cell::UnsafeCell;
use pyo3_ffi::*;

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
    let layout = core::alloc::Layout::new::<YYJSONBuffer>();
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
        debug_assert!(crate::opt::MAX_OPT < i32::from(u16::MAX));

        assert!(crate::deserialize::KEY_MAP
            .set(crate::deserialize::KeyMap::default())
            .is_ok());

        crate::serialize::writer::set_str_formatter_fn();
        crate::str::set_str_create_fn();

        NONE = Py_None();
        TRUE = Py_True();
        FALSE = Py_False();
        EMPTY_UNICODE = PyUnicode_New(0, 255);

        STR_TYPE = &raw mut PyUnicode_Type;
        BYTES_TYPE = &raw mut PyBytes_Type;
        DICT_TYPE = &raw mut PyDict_Type;
        LIST_TYPE = &raw mut PyList_Type;
        TUPLE_TYPE = &raw mut PyTuple_Type;
        NONE_TYPE = (*NONE).ob_type;
        BOOL_TYPE = &raw mut PyBool_Type;
        INT_TYPE = &raw mut PyLong_Type;
        FLOAT_TYPE = &raw mut PyFloat_Type;
        BYTEARRAY_TYPE = &raw mut PyByteArray_Type;
        MEMORYVIEW_TYPE = &raw mut PyMemoryView_Type;

        PyDateTime_IMPORT();

        DATETIME_TYPE = look_up_datetime_type();
        DATE_TYPE = look_up_date_type();
        TIME_TYPE = look_up_time_type();
        UUID_TYPE = look_up_uuid_type();
        ENUM_TYPE = look_up_enum_type();

        FRAGMENT_TYPE = orjson_fragmenttype_new();

        FIELD_TYPE = look_up_field_type();
        ZONEINFO_TYPE = look_up_zoneinfo_type();

        INT_ATTR_STR = PyUnicode_InternFromString(c"int".as_ptr());
        UTCOFFSET_METHOD_STR = PyUnicode_InternFromString(c"utcoffset".as_ptr());
        NORMALIZE_METHOD_STR = PyUnicode_InternFromString(c"normalize".as_ptr());
        CONVERT_METHOD_STR = PyUnicode_InternFromString(c"convert".as_ptr());
        DST_STR = PyUnicode_InternFromString(c"dst".as_ptr());
        DICT_STR = PyUnicode_InternFromString(c"__dict__".as_ptr());
        DATACLASS_FIELDS_STR = PyUnicode_InternFromString(c"__dataclass_fields__".as_ptr());
        SLOTS_STR = PyUnicode_InternFromString(c"__slots__".as_ptr());
        FIELD_TYPE_STR = PyUnicode_InternFromString(c"_field_type".as_ptr());
        ARRAY_STRUCT_STR = PyUnicode_InternFromString(c"__array_struct__".as_ptr());
        DTYPE_STR = PyUnicode_InternFromString(c"dtype".as_ptr());
        DESCR_STR = PyUnicode_InternFromString(c"descr".as_ptr());
        VALUE_STR = PyUnicode_InternFromString(c"value".as_ptr());
        DEFAULT = PyUnicode_InternFromString(c"default".as_ptr());
        OPTION = PyUnicode_InternFromString(c"option".as_ptr());
        JsonEncodeError = pyo3_ffi::PyExc_TypeError;
        Py_INCREF(JsonEncodeError);
        JsonDecodeError = look_up_json_exc();
    };
    true
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_json_exc() -> *mut PyObject {
    unsafe {
        let module = PyImport_ImportModule(c"json".as_ptr());
        let module_dict = PyObject_GenericGetDict(module, null_mut());
        let ptr = PyMapping_GetItemString(module_dict, c"JSONDecodeError".as_ptr());
        let res = pyo3_ffi::PyErr_NewException(c"orjson.JSONDecodeError".as_ptr(), ptr, null_mut());
        Py_DECREF(ptr);
        Py_DECREF(module_dict);
        Py_DECREF(module);
        Py_INCREF(res);
        res
    }
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_numpy_type(
    numpy_module_dict: *mut PyObject,
    np_type: &CStr,
) -> *mut PyTypeObject {
    unsafe {
        let ptr = PyMapping_GetItemString(numpy_module_dict, np_type.as_ptr());
        Py_XDECREF(ptr);
        ptr.cast::<PyTypeObject>()
    }
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
pub fn load_numpy_types() -> Box<Option<NonNull<NumpyTypes>>> {
    unsafe {
        let numpy = PyImport_ImportModule(c"numpy".as_ptr());
        if numpy.is_null() {
            PyErr_Clear();
            return Box::new(None);
        }
        let numpy_module_dict = PyObject_GenericGetDict(numpy, null_mut());
        let types = Box::new(NumpyTypes {
            array: look_up_numpy_type(numpy_module_dict, c"ndarray"),
            float16: look_up_numpy_type(numpy_module_dict, c"half"),
            float32: look_up_numpy_type(numpy_module_dict, c"float32"),
            float64: look_up_numpy_type(numpy_module_dict, c"float64"),
            int8: look_up_numpy_type(numpy_module_dict, c"int8"),
            int16: look_up_numpy_type(numpy_module_dict, c"int16"),
            int32: look_up_numpy_type(numpy_module_dict, c"int32"),
            int64: look_up_numpy_type(numpy_module_dict, c"int64"),
            uint16: look_up_numpy_type(numpy_module_dict, c"uint16"),
            uint32: look_up_numpy_type(numpy_module_dict, c"uint32"),
            uint64: look_up_numpy_type(numpy_module_dict, c"uint64"),
            uint8: look_up_numpy_type(numpy_module_dict, c"uint8"),
            bool_: look_up_numpy_type(numpy_module_dict, c"bool_"),
            datetime64: look_up_numpy_type(numpy_module_dict, c"datetime64"),
        });
        Py_XDECREF(numpy_module_dict);
        Py_XDECREF(numpy);
        Box::new(Some(nonnull!(Box::<NumpyTypes>::into_raw(types))))
    }
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_field_type() -> *mut PyTypeObject {
    unsafe {
        let module = PyImport_ImportModule(c"dataclasses".as_ptr());
        let module_dict = PyObject_GenericGetDict(module, null_mut());
        let ptr = PyMapping_GetItemString(module_dict, c"_FIELD".as_ptr()).cast::<PyTypeObject>();
        Py_DECREF(module_dict);
        Py_DECREF(module);
        ptr
    }
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_enum_type() -> *mut PyTypeObject {
    unsafe {
        let module = PyImport_ImportModule(c"enum".as_ptr());
        let module_dict = PyObject_GenericGetDict(module, null_mut());
        let ptr = PyMapping_GetItemString(module_dict, c"EnumMeta".as_ptr()).cast::<PyTypeObject>();
        Py_DECREF(module_dict);
        Py_DECREF(module);
        ptr
    }
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_uuid_type() -> *mut PyTypeObject {
    unsafe {
        let uuid_mod = PyImport_ImportModule(c"uuid".as_ptr());
        let uuid_mod_dict = PyObject_GenericGetDict(uuid_mod, null_mut());
        let uuid = PyMapping_GetItemString(uuid_mod_dict, c"NAMESPACE_DNS".as_ptr());
        let ptr = (*uuid).ob_type;
        Py_DECREF(uuid);
        Py_DECREF(uuid_mod_dict);
        Py_DECREF(uuid_mod);
        ptr
    }
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_datetime_type() -> *mut PyTypeObject {
    unsafe {
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
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_date_type() -> *mut PyTypeObject {
    unsafe {
        let date = ((*PyDateTimeAPI()).Date_FromDate)(1, 1, 1, (*(PyDateTimeAPI())).DateType);
        let ptr = (*date).ob_type;
        Py_DECREF(date);
        ptr
    }
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_time_type() -> *mut PyTypeObject {
    unsafe {
        let time =
            ((*PyDateTimeAPI()).Time_FromTime)(0, 0, 0, 0, NONE, (*(PyDateTimeAPI())).TimeType);
        let ptr = (*time).ob_type;
        Py_DECREF(time);
        ptr
    }
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
unsafe fn look_up_zoneinfo_type() -> *mut PyTypeObject {
    unsafe {
        let module = PyImport_ImportModule(c"zoneinfo".as_ptr());
        let module_dict = PyObject_GenericGetDict(module, null_mut());
        let ptr = PyMapping_GetItemString(module_dict, c"ZoneInfo".as_ptr()).cast::<PyTypeObject>();
        Py_DECREF(module_dict);
        Py_DECREF(module);
        ptr
    }
}
