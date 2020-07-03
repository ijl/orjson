// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::decode;
use crate::encode;
use crate::opt;
use crate::typeref;

use pyo3::ffi::*;
use std::borrow::Cow;
use std::os::raw::c_char;
use std::ptr::NonNull;

const DUMPS_DOC: &str = "dumps(obj, /, default, option)\n--\n\nSerialize Python objects to JSON.\0";
const LOADS_DOC: &str = "loads(obj, /)\n--\n\nDeserialize JSON to Python objects.\0";

macro_rules! opt {
    ($mptr:expr, $name:expr, $opt:expr) => {
        unsafe {
            #[cfg(not(target_os = "windows"))]
            PyModule_AddIntConstant($mptr, $name.as_ptr() as *const c_char, $opt as i64);
            #[cfg(target_os = "windows")]
            PyModule_AddIntConstant($mptr, $name.as_ptr() as *const c_char, $opt as i32);
        }
    };
}

#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn PyInit_orjson() -> *mut PyObject {
    let mut init = PyModuleDef_INIT;
    init.m_name = "orjson\0".as_ptr() as *const c_char;
    let mptr = PyModule_Create(Box::into_raw(Box::new(init)));

    let version = env!("CARGO_PKG_VERSION");
    unsafe {
        PyModule_AddObject(
            mptr,
            "__version__\0".as_ptr() as *const c_char,
            PyUnicode_FromStringAndSize(version.as_ptr() as *const c_char, version.len() as isize),
        )
    };

    let wrapped_dumps: PyMethodDef;

    #[cfg(python37)]
    {
        wrapped_dumps = PyMethodDef {
            ml_name: "dumps\0".as_ptr() as *const c_char,
            ml_meth: Some(unsafe {
                std::mem::transmute::<crate::ffi::_PyCFunctionFastWithKeywords, PyCFunction>(dumps)
            }),
            ml_flags: pyo3::ffi::METH_FASTCALL | METH_KEYWORDS,
            ml_doc: DUMPS_DOC.as_ptr() as *const c_char,
        };
    }

    #[cfg(not(python37))]
    {
        wrapped_dumps = PyMethodDef {
            ml_name: "dumps\0".as_ptr() as *const c_char,
            ml_meth: Some(unsafe {
                std::mem::transmute::<PyCFunctionWithKeywords, PyCFunction>(dumps)
            }),
            ml_flags: METH_VARARGS | METH_KEYWORDS,
            ml_doc: DUMPS_DOC.as_ptr() as *const c_char,
        };
    }

    unsafe {
        PyModule_AddObject(
            mptr,
            "dumps\0".as_ptr() as *const c_char,
            PyCFunction_New(Box::into_raw(Box::new(wrapped_dumps)), std::ptr::null_mut()),
        )
    };

    let wrapped_loads = PyMethodDef {
        ml_name: "loads\0".as_ptr() as *const c_char,
        ml_meth: Some(loads),
        ml_flags: METH_O,
        ml_doc: LOADS_DOC.as_ptr() as *const c_char,
    };

    unsafe {
        PyModule_AddObject(
            mptr,
            "loads\0".as_ptr() as *const c_char,
            PyCFunction_New(Box::into_raw(Box::new(wrapped_loads)), std::ptr::null_mut()),
        )
    };

    opt!(mptr, "OPT_APPEND_NEWLINE\0", opt::APPEND_NEWLINE);
    opt!(mptr, "OPT_INDENT_2\0", opt::INDENT_2);
    opt!(mptr, "OPT_NAIVE_UTC\0", opt::NAIVE_UTC);
    opt!(mptr, "OPT_NON_STR_KEYS\0", opt::NON_STR_KEYS);
    opt!(mptr, "OPT_OMIT_MICROSECONDS\0", opt::OMIT_MICROSECONDS);
    opt!(
        mptr,
        "OPT_PASSTHROUGH_DATETIME\0",
        opt::PASSTHROUGH_DATETIME
    );
    opt!(
        mptr,
        "OPT_PASSTHROUGH_SUBCLASS\0",
        opt::PASSTHROUGH_SUBCLASS
    );
    opt!(mptr, "OPT_SERIALIZE_DATACLASS\0", opt::SERIALIZE_DATACLASS);
    opt!(mptr, "OPT_SERIALIZE_NUMPY\0", opt::SERIALIZE_NUMPY);
    opt!(mptr, "OPT_SERIALIZE_UUID\0", opt::SERIALIZE_UUID);
    opt!(mptr, "OPT_SORT_KEYS\0", opt::SORT_KEYS);
    opt!(mptr, "OPT_STRICT_INTEGER\0", opt::STRICT_INTEGER);
    opt!(mptr, "OPT_UTC_Z\0", opt::UTC_Z);

    typeref::init_typerefs();

    unsafe {
        PyModule_AddObject(
            mptr,
            "JSONDecodeError\0".as_ptr() as *const c_char,
            typeref::JsonDecodeError,
        );
        PyModule_AddObject(
            mptr,
            "JSONEncodeError\0".as_ptr() as *const c_char,
            typeref::JsonEncodeError,
        )
    };

    mptr
}

#[cold]
fn raise_loads_exception(msg: Cow<str>) -> *mut PyObject {
    unsafe {
        let err_msg =
            PyUnicode_FromStringAndSize(msg.as_ptr() as *const c_char, msg.len() as isize);
        let args = PyTuple_New(3);
        let doc = PyUnicode_New(0, 255);
        let pos = PyLong_FromLongLong(0);
        PyTuple_SET_ITEM(args, 0, err_msg);
        PyTuple_SET_ITEM(args, 1, doc);
        PyTuple_SET_ITEM(args, 2, pos);
        PyErr_SetObject(typeref::JsonDecodeError, args);
        Py_DECREF(args);
    };
    std::ptr::null_mut()
}

#[cold]
fn raise_dumps_exception(msg: Cow<str>) -> *mut PyObject {
    unsafe {
        let err_msg =
            PyUnicode_FromStringAndSize(msg.as_ptr() as *const c_char, msg.len() as isize);
        PyErr_SetObject(typeref::JsonEncodeError, err_msg);
        Py_DECREF(err_msg);
    };
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn loads(_self: *mut PyObject, obj: *mut PyObject) -> *mut PyObject {
    match decode::deserialize(obj) {
        Ok(val) => val.as_ptr(),
        Err(err) => raise_loads_exception(Cow::Owned(err)),
    }
}

#[cfg(python37)]
#[no_mangle]
pub unsafe extern "C" fn dumps(
    _self: *mut PyObject,
    args: *const *mut PyObject,
    nargs: Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject {
    let mut default: Option<NonNull<PyObject>> = None;
    let mut optsptr: Option<NonNull<PyObject>> = None;

    let num_args = pyo3::ffi::PyVectorcall_NARGS(nargs as isize);
    if unlikely!(num_args == 0) {
        return raise_dumps_exception(Cow::Borrowed(
            "dumps() missing 1 required positional argument: 'obj'",
        ));
    }
    if num_args & 2 == 2 {
        default = Some(NonNull::new_unchecked(*args.offset(1)));
    }
    if num_args & 3 == 3 {
        optsptr = Some(NonNull::new_unchecked(*args.offset(2)));
    }
    if !kwnames.is_null() {
        for i in 0..=PyTuple_GET_SIZE(kwnames) - 1 {
            let arg = PyTuple_GET_ITEM(kwnames, i as Py_ssize_t);
            if arg == typeref::DEFAULT {
                if unlikely!(num_args & 2 == 2) {
                    return raise_dumps_exception(Cow::Borrowed(
                        "dumps() got multiple values for argument: 'default'",
                    ));
                }
                default = Some(NonNull::new_unchecked(*args.offset(num_args + i)));
            } else if arg == typeref::OPTION {
                if unlikely!(num_args & 3 == 3) {
                    return raise_dumps_exception(Cow::Borrowed(
                        "dumps() got multiple values for argument: 'option'",
                    ));
                }
                optsptr = Some(NonNull::new_unchecked(*args.offset(num_args + i)));
            } else {
                return raise_dumps_exception(Cow::Borrowed(
                    "dumps() got an unexpected keyword argument",
                ));
            }
        }
    }

    let mut optsbits: i32 = 0;
    if optsptr.is_some() {
        if (*optsptr.unwrap().as_ptr()).ob_type != typeref::INT_TYPE {
            return raise_dumps_exception(Cow::Borrowed("Invalid opts"));
        }
        optsbits = PyLong_AsLong(optsptr.unwrap().as_ptr()) as i32;
        if optsbits < 0 || optsbits > opt::MAX_OPT {
            return raise_dumps_exception(Cow::Borrowed("Invalid opts"));
        }
    }

    match encode::serialize(*args, default, optsbits as opt::Opt) {
        Ok(val) => val.as_ptr(),
        Err(err) => raise_dumps_exception(Cow::Borrowed(&err)),
    }
}

#[cfg(not(python37))]
#[no_mangle]
pub unsafe extern "C" fn dumps(
    _self: *mut PyObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject {
    let mut default: Option<NonNull<PyObject>> = None;
    let mut optsptr: Option<NonNull<PyObject>> = None;

    let num_args = PyTuple_GET_SIZE(args);
    if unlikely!(num_args == 0) {
        return raise_dumps_exception(Cow::Borrowed(
            "dumps() missing 1 required positional argument: 'obj'",
        ));
    }
    if num_args & 2 == 2 {
        default = Some(NonNull::new_unchecked(PyTuple_GET_ITEM(args, 1)));
    }
    if num_args & 3 == 3 {
        optsptr = Some(NonNull::new_unchecked(PyTuple_GET_ITEM(args, 2)));
    }

    if !kwds.is_null() {
        let len = unsafe { crate::dict::PyDict_GET_SIZE(kwds) as usize };
        let mut pos = 0isize;
        let mut arg: *mut PyObject = std::ptr::null_mut();
        let mut val: *mut PyObject = std::ptr::null_mut();
        for _ in 0..=len - 1 {
            unsafe { _PyDict_Next(kwds, &mut pos, &mut arg, &mut val, std::ptr::null_mut()) };
            if arg == typeref::DEFAULT {
                if unlikely!(num_args & 2 == 2) {
                    return raise_dumps_exception(Cow::Borrowed(
                        "dumps() got multiple values for argument: 'default'",
                    ));
                }
                default = Some(NonNull::new_unchecked(val));
            } else if arg == typeref::OPTION {
                if unlikely!(num_args & 3 == 3) {
                    return raise_dumps_exception(Cow::Borrowed(
                        "dumps() got multiple values for argument: 'option'",
                    ));
                }
                optsptr = Some(NonNull::new_unchecked(val));
            } else if arg.is_null() {
                break;
            } else {
                return raise_dumps_exception(Cow::Borrowed(
                    "dumps() got an unexpected keyword argument",
                ));
            }
        }
    }

    let mut optsbits: i32 = 0;
    if optsptr.is_some() {
        if (*optsptr.unwrap().as_ptr()).ob_type != typeref::INT_TYPE {
            return raise_dumps_exception(Cow::Borrowed("Invalid opts"));
        }
        optsbits = PyLong_AsLong(optsptr.unwrap().as_ptr()) as i32;
        if optsbits < 0 || optsbits > opt::MAX_OPT {
            return raise_dumps_exception(Cow::Borrowed("Invalid opts"));
        }
    }

    match encode::serialize(PyTuple_GET_ITEM(args, 0), default, optsbits as opt::Opt) {
        Ok(val) => val.as_ptr(),
        Err(err) => raise_dumps_exception(Cow::Owned(err)),
    }
}
