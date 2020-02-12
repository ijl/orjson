// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::ffi::*;
use std::os::raw::c_char;
use std::sync::Once;

pub static mut HASH_SEED: u64 = 0;
pub static mut NONE: *mut PyObject = 0 as *mut PyObject;
pub static mut TRUE: *mut PyObject = 0 as *mut PyObject;
pub static mut FALSE: *mut PyObject = 0 as *mut PyObject;
pub static mut STR_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut BYTES_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut BYTEARRAY_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut DICT_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut LIST_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut TUPLE_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut NONE_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut BOOL_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut INT_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut FLOAT_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut DATETIME_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut DATE_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut TIME_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut UUID_TYPE: *mut PyTypeObject = 0 as *mut PyTypeObject;
pub static mut INT_ATTR_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut UTCOFFSET_METHOD_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut NORMALIZE_METHOD_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut CONVERT_METHOD_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut DST_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut DICT_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut DATACLASS_FIELDS_STR: *mut PyObject = 0 as *mut PyObject;
pub static mut STR_HASH_FUNCTION: Option<hashfunc> = None;

static INIT: Once = Once::new();

pub fn init_typerefs() {
    INIT.call_once(|| unsafe {
        PyDateTime_IMPORT();
        HASH_SEED = rand::random::<u64>();
        NONE = Py_None();
        TRUE = Py_True();
        FALSE = Py_False();
        let unicode = PyUnicode_New(0, 255);
        STR_TYPE = (*unicode).ob_type;
        STR_HASH_FUNCTION = (*((*unicode).ob_type)).tp_hash;
        BYTES_TYPE = (*PyBytes_FromStringAndSize("".as_ptr() as *const c_char, 0)).ob_type;
        BYTEARRAY_TYPE = (*PyByteArray_FromStringAndSize("".as_ptr() as *const c_char, 0)).ob_type;
        DICT_TYPE = (*PyDict_New()).ob_type;
        LIST_TYPE = (*PyList_New(0 as Py_ssize_t)).ob_type;
        TUPLE_TYPE = (*PyTuple_New(0 as Py_ssize_t)).ob_type;
        NONE_TYPE = (*NONE).ob_type;
        BOOL_TYPE = (*TRUE).ob_type;
        INT_TYPE = (*PyLong_FromLongLong(0)).ob_type;
        FLOAT_TYPE = (*PyFloat_FromDouble(0.0)).ob_type;
        DATETIME_TYPE = look_up_datetime_type();
        DATE_TYPE = look_up_date_type();
        TIME_TYPE = look_up_time_type();
        UUID_TYPE = look_up_uuid_type();
        INT_ATTR_STR = PyUnicode_FromStringAndSize("int".as_ptr() as *const c_char, 3);
        UTCOFFSET_METHOD_STR =
            PyUnicode_FromStringAndSize("utcoffset".as_ptr() as *const c_char, 9);
        NORMALIZE_METHOD_STR =
            PyUnicode_FromStringAndSize("normalize".as_ptr() as *const c_char, 9);
        CONVERT_METHOD_STR = PyUnicode_FromStringAndSize("convert".as_ptr() as *const c_char, 7);
        DST_STR = PyUnicode_FromStringAndSize("dst".as_ptr() as *const c_char, 3);
        DICT_STR = PyUnicode_FromStringAndSize("__dict__".as_ptr() as *const c_char, 8);
        DATACLASS_FIELDS_STR =
            PyUnicode_FromStringAndSize("__dataclass_fields__".as_ptr() as *const c_char, 20);
    });
}

unsafe fn look_up_uuid_type() -> *mut PyTypeObject {
    let uuid_mod = PyImport_ImportModule("uuid\0".as_ptr() as *const c_char);
    let uuid_mod_dict = PyModule_GetDict(uuid_mod);
    let uuid = PyMapping_GetItemString(uuid_mod_dict, "NAMESPACE_DNS\0".as_ptr() as *const c_char);
    let ptr = (*uuid).ob_type;
    Py_DECREF(uuid);
    Py_DECREF(uuid_mod_dict);
    Py_DECREF(uuid_mod);
    ptr
}

unsafe fn look_up_datetime_type() -> *mut PyTypeObject {
    let datetime = (PyDateTimeAPI.DateTime_FromDateAndTime)(
        1970,
        1,
        1,
        0,
        0,
        0,
        0,
        NONE,
        PyDateTimeAPI.DateTimeType,
    );
    let ptr = (*datetime).ob_type;
    Py_DECREF(datetime);
    ptr
}

unsafe fn look_up_date_type() -> *mut PyTypeObject {
    let date = (PyDateTimeAPI.Date_FromDate)(1970, 1, 1, PyDateTimeAPI.DateType);
    let ptr = (*date).ob_type;
    Py_DECREF(date);
    ptr
}

unsafe fn look_up_time_type() -> *mut PyTypeObject {
    let time = (PyDateTimeAPI.Time_FromTime)(0, 0, 0, 0, NONE, PyDateTimeAPI.TimeType);
    let ptr = (*time).ob_type;
    Py_DECREF(time);
    ptr
}
