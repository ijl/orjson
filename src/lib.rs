// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![cfg_attr(feature = "unstable-simd", feature(core_intrinsics))]
#![allow(unused_unsafe)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::zero_prefixed_literal)]

#[macro_use]
mod util;

mod deserialize;
mod exc;
mod ffi;
mod opt;
mod serialize;
mod typeref;
mod unicode;

use pyo3::ffi::*;
use std::borrow::Cow;
use std::os::raw::c_char;
use std::ptr::NonNull;

const DUMPS_DOC: &str =
    "dumps(obj, /, default=None, option=None)\n--\n\nSerialize Python objects to JSON.\0";
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
#[cold]
pub unsafe extern "C" fn PyInit_orjson() -> *mut PyObject {
    let init = PyModuleDef {
        m_base: PyModuleDef_HEAD_INIT,
        m_name: "orjson\0".as_ptr() as *const c_char,
        m_doc: std::ptr::null(),
        m_size: 0,
        m_methods: std::ptr::null_mut(),
        m_slots: std::ptr::null_mut(),
        m_traverse: None,
        m_clear: None,
        m_free: None,
    };
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

    #[cfg(Py_3_8)]
    {
        wrapped_dumps = PyMethodDef {
            ml_name: "dumps\0".as_ptr() as *const c_char,
            ml_meth: Some(unsafe {
                std::mem::transmute::<pyo3::ffi::_PyCFunctionFastWithKeywords, PyCFunction>(dumps)
            }),
            ml_flags: pyo3::ffi::METH_FASTCALL | METH_KEYWORDS,
            ml_doc: DUMPS_DOC.as_ptr() as *const c_char,
        };
    }
    #[cfg(not(Py_3_8))]
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
            PyCFunction_NewEx(
                Box::into_raw(Box::new(wrapped_dumps)),
                std::ptr::null_mut(),
                PyUnicode_InternFromString("orjson\0".as_ptr() as *const c_char),
            ),
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
            PyCFunction_NewEx(
                Box::into_raw(Box::new(wrapped_loads)),
                std::ptr::null_mut(),
                PyUnicode_InternFromString("orjson\0".as_ptr() as *const c_char),
            ),
        )
    };

    opt!(mptr, "OPT_APPEND_NEWLINE\0", opt::APPEND_NEWLINE);
    opt!(mptr, "OPT_INDENT_2\0", opt::INDENT_2);
    opt!(mptr, "OPT_NAIVE_UTC\0", opt::NAIVE_UTC);
    opt!(mptr, "OPT_NON_STR_KEYS\0", opt::NON_STR_KEYS);
    opt!(mptr, "OPT_OMIT_MICROSECONDS\0", opt::OMIT_MICROSECONDS);
    opt!(
        mptr,
        "OPT_PASSTHROUGH_DATACLASS\0",
        opt::PASSTHROUGH_DATACLASS
    );
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

    // maturin>=0.11.0 creates a python package that imports *, hiding dunder by default
    let all: [&str; 20] = [
        "__all__\0",
        "__version__\0",
        "dumps\0",
        "JSONDecodeError\0",
        "JSONEncodeError\0",
        "loads\0",
        "OPT_APPEND_NEWLINE\0",
        "OPT_INDENT_2\0",
        "OPT_NAIVE_UTC\0",
        "OPT_NON_STR_KEYS\0",
        "OPT_OMIT_MICROSECONDS\0",
        "OPT_PASSTHROUGH_DATACLASS\0",
        "OPT_PASSTHROUGH_DATETIME\0",
        "OPT_PASSTHROUGH_SUBCLASS\0",
        "OPT_SERIALIZE_DATACLASS\0",
        "OPT_SERIALIZE_NUMPY\0",
        "OPT_SERIALIZE_UUID\0",
        "OPT_SORT_KEYS\0",
        "OPT_STRICT_INTEGER\0",
        "OPT_UTC_Z\0",
    ];

    let pyall = PyTuple_New(all.len() as isize);
    for (i, obj) in all.iter().enumerate() {
        PyTuple_SET_ITEM(
            pyall,
            i as isize,
            PyUnicode_InternFromString(obj.as_ptr() as *const c_char),
        )
    }

    unsafe {
        PyModule_AddObject(mptr, "__all__\0".as_ptr() as *const c_char, pyall);
    };

    mptr
}

#[cold]
#[inline(never)]
fn raise_loads_exception(err: deserialize::DeserializeError) -> *mut PyObject {
    let pos = err.pos() as i64;
    let msg = err.message;
    let doc = err.data;
    unsafe {
        let err_msg =
            PyUnicode_FromStringAndSize(msg.as_ptr() as *const c_char, msg.len() as isize);
        let args = PyTuple_New(3);
        let doc = PyUnicode_FromStringAndSize(doc.as_ptr() as *const c_char, doc.len() as isize);
        let pos = PyLong_FromLongLong(pos);
        PyTuple_SET_ITEM(args, 0, err_msg);
        PyTuple_SET_ITEM(args, 1, doc);
        PyTuple_SET_ITEM(args, 2, pos);
        PyErr_SetObject(typeref::JsonDecodeError, args);
        Py_DECREF(args);
    };
    std::ptr::null_mut()
}

#[cold]
#[inline(never)]
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
    match crate::deserialize::deserialize(obj) {
        Ok(val) => val.as_ptr(),
        Err(err) => raise_loads_exception(err),
    }
}

#[cfg(Py_3_8)]
#[no_mangle]
pub unsafe extern "C" fn dumps(
    _self: *mut PyObject,
    args: *const *mut PyObject,
    nargs: Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject {
    let mut default: Option<NonNull<PyObject>> = None;
    let mut optsptr: Option<NonNull<PyObject>> = None;

    let num_args = pyo3::ffi::PyVectorcall_NARGS(nargs as usize);
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
    if let Some(opts) = optsptr {
        if (*opts.as_ptr()).ob_type != typeref::INT_TYPE {
            return raise_dumps_exception(Cow::Borrowed("Invalid opts"));
        }
        optsbits = PyLong_AsLong(optsptr.unwrap().as_ptr()) as i32;
        if !(0..=opt::MAX_OPT).contains(&optsbits) {
            return raise_dumps_exception(Cow::Borrowed("Invalid opts"));
        }
    }

    match crate::serialize::serialize(*args, default, optsbits as opt::Opt) {
        Ok(val) => val.as_ptr(),
        Err(err) => raise_dumps_exception(Cow::Borrowed(&err)),
    }
}

#[cfg(not(Py_3_8))]
#[no_mangle]
pub unsafe extern "C" fn dumps(
    _self: *mut PyObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject {
    let mut default: Option<NonNull<PyObject>> = None;
    let mut optsptr: Option<NonNull<PyObject>> = None;

    let obj = PyTuple_GET_ITEM(args, 0);

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
        let len = unsafe { crate::ffi::PyDict_GET_SIZE(kwds) };
        let mut pos = 0isize;
        let mut arg: *mut PyObject = std::ptr::null_mut();
        let mut val: *mut PyObject = std::ptr::null_mut();
        for _ in 0..=len.saturating_sub(1) {
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
    if let Some(opts) = optsptr {
        if (*opts.as_ptr()).ob_type != typeref::INT_TYPE {
            return raise_dumps_exception(Cow::Borrowed("Invalid opts"));
        }
        optsbits = PyLong_AsLong(optsptr.unwrap().as_ptr()) as i32;
        if optsbits < 0 || optsbits > opt::MAX_OPT {
            return raise_dumps_exception(Cow::Borrowed("Invalid opts"));
        }
    }

    match crate::serialize::serialize(obj, default, optsbits as opt::Opt) {
        Ok(val) => val.as_ptr(),
        Err(err) => raise_dumps_exception(Cow::Owned(err)),
    }
}
