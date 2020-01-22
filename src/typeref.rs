// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::os::raw::c_char;
use std::sync::Once;

pub static mut HASH_SEED: u64 = 0;
pub static mut NONE: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut TRUE: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut FALSE: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut STR_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut BYTES_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut BYTEARRAY_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut DICT_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut LIST_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut TUPLE_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut NONE_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut BOOL_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut INT_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut FLOAT_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut DATETIME_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut DATE_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut TIME_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut UUID_PTR: *mut pyo3::ffi::PyTypeObject = 0 as *mut pyo3::ffi::PyTypeObject;
pub static mut INT_ATTR_STR: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut UTCOFFSET_METHOD_STR: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut NORMALIZE_METHOD_STR: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut CONVERT_METHOD_STR: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut DST_STR: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut DATACLASS_FIELDS_STR: *mut pyo3::ffi::PyObject = 0 as *mut pyo3::ffi::PyObject;
pub static mut STR_HASH_FUNCTION: Option<pyo3::ffi::hashfunc> = None;

static EMTPY_STR: &str = "";

static INIT: Once = Once::new();

pub unsafe fn look_up_uuid_type() -> *mut pyo3::ffi::PyTypeObject {
    let uuid_mod = pyo3::ffi::PyImport_ImportModule("uuid\0".as_ptr() as *const c_char);
    let uuid_mod_dict = pyo3::ffi::PyModule_GetDict(uuid_mod);
    let uuid = pyo3::ffi::PyMapping_GetItemString(
        uuid_mod_dict,
        "NAMESPACE_DNS\0".as_ptr() as *const c_char,
    );
    let ptr = (*uuid).ob_type;
    pyo3::ffi::Py_DECREF(uuid);
    pyo3::ffi::Py_DECREF(uuid_mod_dict);
    pyo3::ffi::Py_DECREF(uuid_mod);
    ptr
}

pub fn init_typerefs() {
    INIT.call_once(|| unsafe {
        pyo3::ffi::PyDateTime_IMPORT();
        HASH_SEED = rand::random::<u64>();
        NONE = pyo3::ffi::Py_None();
        TRUE = pyo3::ffi::Py_True();
        FALSE = pyo3::ffi::Py_False();
        let unicode = pyo3::ffi::PyUnicode_New(0, 255);
        STR_PTR = (*unicode).ob_type;
        STR_HASH_FUNCTION = (*((*unicode).ob_type)).tp_hash;
        BYTES_PTR = (*pyo3::ffi::PyBytes_FromStringAndSize(
            EMTPY_STR.as_ptr() as *const c_char,
            EMTPY_STR.len() as pyo3::ffi::Py_ssize_t,
        ))
        .ob_type;
        BYTEARRAY_PTR = (*pyo3::ffi::PyByteArray_FromStringAndSize(
            EMTPY_STR.as_ptr() as *const c_char,
            EMTPY_STR.len() as pyo3::ffi::Py_ssize_t,
        ))
        .ob_type;
        DICT_PTR = (*pyo3::ffi::PyDict_New()).ob_type;
        LIST_PTR = (*pyo3::ffi::PyList_New(0 as pyo3::ffi::Py_ssize_t)).ob_type;
        TUPLE_PTR = (*pyo3::ffi::PyTuple_New(0 as pyo3::ffi::Py_ssize_t)).ob_type;
        NONE_PTR = (*NONE).ob_type;
        BOOL_PTR = (*TRUE).ob_type;
        INT_PTR = (*pyo3::ffi::PyLong_FromLongLong(0)).ob_type;
        FLOAT_PTR = (*pyo3::ffi::PyFloat_FromDouble(0.0)).ob_type;
        let datetime = (pyo3::ffi::PyDateTimeAPI.DateTime_FromDateAndTime)(
            1970,
            1,
            1,
            0,
            0,
            0,
            0,
            NONE,
            pyo3::ffi::PyDateTimeAPI.DateTimeType,
        );
        DATETIME_PTR = (*datetime).ob_type;
        let date =
            (pyo3::ffi::PyDateTimeAPI.Date_FromDate)(1970, 1, 1, pyo3::ffi::PyDateTimeAPI.DateType);
        DATE_PTR = (*date).ob_type;
        let time = (pyo3::ffi::PyDateTimeAPI.Time_FromTime)(
            0,
            0,
            0,
            0,
            NONE,
            pyo3::ffi::PyDateTimeAPI.TimeType,
        );
        TIME_PTR = (*time).ob_type;
        UUID_PTR = look_up_uuid_type();
        // We'll be looking up the "int" attribute on UUIDs, so it's convenient
        // to define this ahead of time as a constant.
        INT_ATTR_STR = pyo3::ffi::PyUnicode_FromStringAndSize("int".as_ptr() as *const c_char, 3);
        UTCOFFSET_METHOD_STR =
            pyo3::ffi::PyUnicode_FromStringAndSize("utcoffset".as_ptr() as *const c_char, 9);
        NORMALIZE_METHOD_STR =
            pyo3::ffi::PyUnicode_FromStringAndSize("normalize".as_ptr() as *const c_char, 9);
        CONVERT_METHOD_STR =
            pyo3::ffi::PyUnicode_FromStringAndSize("convert".as_ptr() as *const c_char, 7);
        DST_STR = pyo3::ffi::PyUnicode_FromStringAndSize("dst".as_ptr() as *const c_char, 3);
        DATACLASS_FIELDS_STR = pyo3::ffi::PyUnicode_FromStringAndSize(
            "__dataclass_fields__".as_ptr() as *const c_char,
            20,
        );
        pyo3::ffi::Py_DECREF(datetime);
        pyo3::ffi::Py_DECREF(date);
        pyo3::ffi::Py_DECREF(time);
    });
}
