// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![cfg_attr(feature = "intrinsics", feature(core_intrinsics))]
#![cfg_attr(feature = "optimize", feature(optimize_attribute))]
#![allow(unused_unsafe)]
#![allow(clippy::explicit_auto_deref)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::unnecessary_unwrap)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::zero_prefixed_literal)]
#![allow(non_camel_case_types)]

#[macro_use]
mod util;

mod deserialize;
mod error;
mod ffi;
mod opt;
mod serialize;
mod typeref;
mod unicode;

#[cfg(feature = "yyjson")]
mod yyjson;

use pyo3_ffi::*;
use std::borrow::Cow;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;

#[allow(unused_imports)]
use std::ptr::{null, null_mut, NonNull};

#[cfg(Py_3_10)]
macro_rules! add {
    ($mptr:expr, $name:expr, $obj:expr) => {
        PyModule_AddObjectRef($mptr, $name.as_ptr() as *const c_char, $obj);
    };
}

#[cfg(not(Py_3_10))]
macro_rules! add {
    ($mptr:expr, $name:expr, $obj:expr) => {
        PyModule_AddObject($mptr, $name.as_ptr() as *const c_char, $obj);
    };
}

macro_rules! opt {
    ($mptr:expr, $name:expr, $opt:expr) => {
        #[cfg(all(not(target_os = "windows"), target_pointer_width = "64"))]
        PyModule_AddIntConstant($mptr, $name.as_ptr() as *const c_char, $opt as i64);
        #[cfg(all(not(target_os = "windows"), target_pointer_width = "32"))]
        PyModule_AddIntConstant($mptr, $name.as_ptr() as *const c_char, $opt as i32);
        #[cfg(target_os = "windows")]
        PyModule_AddIntConstant($mptr, $name.as_ptr() as *const c_char, $opt as i32);
    };
}

#[allow(non_snake_case)]
#[no_mangle]
#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
pub unsafe extern "C" fn orjson_init_exec(mptr: *mut PyObject) -> c_int {
    typeref::init_typerefs();
    {
        let version = env!("CARGO_PKG_VERSION");
        let pyversion =
            PyUnicode_FromStringAndSize(version.as_ptr() as *const c_char, version.len() as isize);
        add!(mptr, "__version__\0", pyversion);
    }
    {
        let dumps_doc =
            "dumps(obj, /, default=None, option=None)\n--\n\nSerialize Python objects to JSON.\0";

        let wrapped_dumps: PyMethodDef;

        #[cfg(Py_3_8)]
        {
            wrapped_dumps = PyMethodDef {
                ml_name: "dumps\0".as_ptr() as *const c_char,
                ml_meth: PyMethodDefPointer {
                    _PyCFunctionFastWithKeywords: dumps,
                },
                ml_flags: pyo3_ffi::METH_FASTCALL | METH_KEYWORDS,
                ml_doc: dumps_doc.as_ptr() as *const c_char,
            };
        }
        #[cfg(not(Py_3_8))]
        {
            wrapped_dumps = PyMethodDef {
                ml_name: "dumps\0".as_ptr() as *const c_char,
                ml_meth: PyMethodDefPointer {
                    PyCFunctionWithKeywords: dumps,
                },
                ml_flags: METH_VARARGS | METH_KEYWORDS,
                ml_doc: dumps_doc.as_ptr() as *const c_char,
            };
        }

        let func = PyCFunction_NewEx(
            Box::into_raw(Box::new(wrapped_dumps)),
            null_mut(),
            PyUnicode_InternFromString("orjson\0".as_ptr() as *const c_char),
        );
        add!(mptr, "dumps\0", func);
    }

    {
        let loads_doc = "loads(obj, /)\n--\n\nDeserialize JSON to Python objects.\0";

        let wrapped_loads = PyMethodDef {
            ml_name: "loads\0".as_ptr() as *const c_char,
            ml_meth: PyMethodDefPointer { PyCFunction: loads },
            ml_flags: METH_O,
            ml_doc: loads_doc.as_ptr() as *const c_char,
        };
        let func = PyCFunction_NewEx(
            Box::into_raw(Box::new(wrapped_loads)),
            null_mut(),
            PyUnicode_InternFromString("orjson\0".as_ptr() as *const c_char),
        );
        add!(mptr, "loads\0", func);
    }

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

    add!(mptr, "JSONDecodeError\0", typeref::JsonDecodeError);
    add!(mptr, "JSONEncodeError\0", typeref::JsonEncodeError);

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

    add!(mptr, "__all__\0", pyall);
    0
}

#[allow(non_snake_case)]
#[no_mangle]
#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
pub unsafe extern "C" fn PyInit_orjson() -> *mut PyModuleDef {
    let mod_slots: Box<[PyModuleDef_Slot; 2]> = Box::new([
        PyModuleDef_Slot {
            slot: Py_mod_exec,
            value: orjson_init_exec as *mut c_void,
        },
        PyModuleDef_Slot {
            slot: 0,
            value: null_mut(),
        },
    ]);

    let init = Box::new(PyModuleDef {
        m_base: PyModuleDef_HEAD_INIT,
        m_name: "orjson\0".as_ptr() as *const c_char,
        m_doc: null(),
        m_size: 0,
        m_methods: null_mut(),
        m_slots: Box::into_raw(mod_slots) as *mut PyModuleDef_Slot,
        m_traverse: None,
        m_clear: None,
        m_free: None,
    });
    let init_ptr = Box::into_raw(init);
    PyModuleDef_Init(init_ptr);
    init_ptr
}

#[cold]
#[inline(never)]
#[cfg_attr(feature = "optimize", optimize(size))]
fn raise_loads_exception(err: deserialize::DeserializeError) -> *mut PyObject {
    let pos = err.pos();
    let msg = err.message;
    let doc = match err.data {
        Some(as_str) => unsafe {
            PyUnicode_FromStringAndSize(as_str.as_ptr() as *const c_char, as_str.len() as isize)
        },
        None => {
            ffi!(Py_INCREF(crate::typeref::EMPTY_UNICODE));
            unsafe { crate::typeref::EMPTY_UNICODE }
        }
    };
    unsafe {
        let err_msg =
            PyUnicode_FromStringAndSize(msg.as_ptr() as *const c_char, msg.len() as isize);
        let args = PyTuple_New(3);
        let pos = PyLong_FromLongLong(pos);
        PyTuple_SET_ITEM(args, 0, err_msg);
        PyTuple_SET_ITEM(args, 1, doc);
        PyTuple_SET_ITEM(args, 2, pos);
        PyErr_SetObject(typeref::JsonDecodeError, args);
        Py_DECREF(args);
    };
    null_mut()
}

#[cold]
#[inline(never)]
#[cfg_attr(feature = "optimize", optimize(size))]
fn raise_dumps_exception(msg: Cow<str>) -> *mut PyObject {
    unsafe {
        let err_msg =
            PyUnicode_FromStringAndSize(msg.as_ptr() as *const c_char, msg.len() as isize);
        PyErr_SetObject(typeref::JsonEncodeError, err_msg);
        Py_DECREF(err_msg);
    };
    null_mut()
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

    let num_args = PyVectorcall_NARGS(nargs as usize);
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
        for i in 0..=Py_SIZE(kwnames).saturating_sub(1) {
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

    let num_args = Py_SIZE(args);
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
        for (arg, val) in crate::ffi::PyDictIter::from_pyobject(kwds) {
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
