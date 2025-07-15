// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::ffi::orjson_fragmenttype_new;
use core::ffi::CStr;
use core::ptr::{null_mut, NonNull};
use once_cell::race::{OnceBool, OnceBox};

use pyo3_ffi::*;

pub(crate) static mut DEFAULT: *mut PyObject = null_mut();
pub(crate) static mut OPTION: *mut PyObject = null_mut();

pub(crate) static mut NONE: *mut PyObject = null_mut();
pub(crate) static mut TRUE: *mut PyObject = null_mut();
pub(crate) static mut FALSE: *mut PyObject = null_mut();
pub(crate) static mut EMPTY_UNICODE: *mut PyObject = null_mut();

pub(crate) static mut BYTES_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut BYTEARRAY_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut MEMORYVIEW_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut STR_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut INT_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut BOOL_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut NONE_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut FLOAT_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut LIST_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut DICT_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut DATETIME_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut DATE_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut TIME_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut TUPLE_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut UUID_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut ENUM_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut FIELD_TYPE: *mut PyTypeObject = null_mut();
pub(crate) static mut FRAGMENT_TYPE: *mut PyTypeObject = null_mut();

pub(crate) static mut ZONEINFO_TYPE: *mut PyTypeObject = null_mut();

pub(crate) static mut UTCOFFSET_METHOD_STR: *mut PyObject = null_mut();
pub(crate) static mut NORMALIZE_METHOD_STR: *mut PyObject = null_mut();
pub(crate) static mut CONVERT_METHOD_STR: *mut PyObject = null_mut();
pub(crate) static mut DST_STR: *mut PyObject = null_mut();

pub(crate) static mut DICT_STR: *mut PyObject = null_mut();
pub(crate) static mut DATACLASS_FIELDS_STR: *mut PyObject = null_mut();
pub(crate) static mut SLOTS_STR: *mut PyObject = null_mut();
pub(crate) static mut FIELD_TYPE_STR: *mut PyObject = null_mut();
pub(crate) static mut ARRAY_STRUCT_STR: *mut PyObject = null_mut();
pub(crate) static mut DTYPE_STR: *mut PyObject = null_mut();
pub(crate) static mut DESCR_STR: *mut PyObject = null_mut();
pub(crate) static mut VALUE_STR: *mut PyObject = null_mut();
pub(crate) static mut INT_ATTR_STR: *mut PyObject = null_mut();

#[allow(non_upper_case_globals)]
pub(crate) static mut JsonEncodeError: *mut PyObject = null_mut();
#[allow(non_upper_case_globals)]
pub(crate) static mut JsonDecodeError: *mut PyObject = null_mut();

unsafe fn look_up_type_object(module_name: &CStr, member_name: &CStr) -> *mut PyTypeObject {
    unsafe {
        let module = PyImport_ImportModule(module_name.as_ptr());
        let module_dict = PyObject_GenericGetDict(module, null_mut());
        let ptr = PyMapping_GetItemString(module_dict, member_name.as_ptr()).cast::<PyTypeObject>();
        Py_DECREF(module_dict);
        Py_DECREF(module);
        ptr
    }
}

static INIT: OnceBool = OnceBool::new();

pub(crate) fn init_typerefs() {
    INIT.get_or_init(_init_typerefs_impl);
}

#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
fn _init_typerefs_impl() -> bool {
    unsafe {
        debug_assert!(crate::opt::MAX_OPT < i32::from(u16::MAX));

        #[cfg(not(Py_GIL_DISABLED))]
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
        let datetime_capsule = pyo3_ffi::PyCapsule_Import(c"datetime.datetime_CAPI".as_ptr(), 1)
            .cast::<pyo3_ffi::PyDateTime_CAPI>();

        DATETIME_TYPE = (*datetime_capsule).DateTimeType;
        DATE_TYPE = (*datetime_capsule).DateType;
        TIME_TYPE = (*datetime_capsule).TimeType;
        ZONEINFO_TYPE = (*datetime_capsule).TZInfoType;

        UUID_TYPE = look_up_type_object(c"uuid", c"UUID");
        ENUM_TYPE = look_up_type_object(c"enum", c"EnumMeta");
        FIELD_TYPE = look_up_type_object(c"dataclasses", c"_FIELD");

        FRAGMENT_TYPE = orjson_fragmenttype_new();

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
        let json_jsondecodeerror =
            look_up_type_object(c"json", c"JSONDecodeError").cast::<PyObject>();
        JsonDecodeError = pyo3_ffi::PyErr_NewException(
            c"orjson.JSONDecodeError".as_ptr(),
            json_jsondecodeerror,
            null_mut(),
        );
        Py_XDECREF(json_jsondecodeerror);
    };
    true
}

pub(crate) struct NumpyTypes {
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

pub(crate) static mut NUMPY_TYPES: OnceBox<Option<NonNull<NumpyTypes>>> = OnceBox::new();

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
pub(crate) fn load_numpy_types() -> Box<Option<NonNull<NumpyTypes>>> {
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
