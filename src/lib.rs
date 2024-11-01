// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#![cfg_attr(feature = "avx512", feature(stdarch_x86_avx512, avx512_target_feature))]
#![cfg_attr(feature = "intrinsics", feature(core_intrinsics))]
#![cfg_attr(feature = "optimize", feature(optimize_attribute))]
#![cfg_attr(feature = "unstable-simd", feature(portable_simd))]
#![allow(internal_features)] // core_intrinsics
#![allow(non_camel_case_types)]
#![allow(static_mut_refs)]
#![allow(unknown_lints)] // internal_features
#![allow(unused_unsafe)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::uninlined_format_args)] // MSRV 1.66
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::zero_prefixed_literal)]

#[cfg(feature = "unwind")]
extern crate unwinding;

#[macro_use]
mod util;

mod deserialize;
mod ffi;
mod opt;
mod serialize;
mod str;
mod typeref;

use core::ffi::{c_char, c_int, c_void};
use pyo3_ffi::*;

#[allow(unused_imports)]
use core::ptr::{null, null_mut, NonNull};

#[cfg(Py_3_13)]
macro_rules! add {
    ($mptr:expr, $name:expr, $obj:expr) => {
        PyModule_Add($mptr, $name.as_ptr() as *const c_char, $obj);
    };
}

#[cfg(all(Py_3_10, not(Py_3_13)))]
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

        let wrapped_dumps = PyMethodDef {
            ml_name: "dumps\0".as_ptr() as *const c_char,
            ml_meth: PyMethodDefPointer {
                #[cfg(Py_3_10)]
                PyCFunctionFastWithKeywords: dumps,
                #[cfg(not(Py_3_10))]
                _PyCFunctionFastWithKeywords: dumps,
            },
            ml_flags: pyo3_ffi::METH_FASTCALL | METH_KEYWORDS,
            ml_doc: dumps_doc.as_ptr() as *const c_char,
        };

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

    add!(mptr, "Fragment\0", typeref::FRAGMENT_TYPE as *mut PyObject);

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

    0
}

#[cfg(Py_3_13)]
#[allow(non_upper_case_globals)]
const Py_mod_gil: c_int = 4;
#[cfg(Py_3_13)]
#[allow(non_upper_case_globals, dead_code, fuzzy_provenance_casts)]
const Py_MOD_GIL_USED: *mut c_void = 0 as *mut c_void;
#[cfg(Py_3_13)]
#[allow(non_upper_case_globals, dead_code, fuzzy_provenance_casts)]
const Py_MOD_GIL_NOT_USED: *mut c_void = 1 as *mut c_void;

#[cfg(not(Py_3_12))]
const PYMODULEDEF_LEN: usize = 2;
#[cfg(all(Py_3_12, not(Py_3_13)))]
const PYMODULEDEF_LEN: usize = 3;
#[cfg(Py_3_13)]
const PYMODULEDEF_LEN: usize = 4;

#[allow(non_snake_case)]
#[no_mangle]
#[cold]
#[cfg_attr(feature = "optimize", optimize(size))]
pub unsafe extern "C" fn PyInit_orjson() -> *mut PyModuleDef {
    let mod_slots: Box<[PyModuleDef_Slot; PYMODULEDEF_LEN]> = Box::new([
        PyModuleDef_Slot {
            slot: Py_mod_exec,
            value: orjson_init_exec as *mut c_void,
        },
        #[cfg(Py_3_12)]
        PyModuleDef_Slot {
            slot: Py_mod_multiple_interpreters,
            value: Py_MOD_MULTIPLE_INTERPRETERS_NOT_SUPPORTED,
        },
        #[cfg(Py_3_13)]
        PyModuleDef_Slot {
            slot: Py_mod_gil,
            value: Py_MOD_GIL_USED,
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
            use_immortal!(crate::typeref::EMPTY_UNICODE)
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
        debug_assert!(ffi!(Py_REFCNT(args)) <= 2);
        Py_DECREF(args);
    };
    null_mut()
}

#[cold]
#[inline(never)]
#[cfg_attr(feature = "optimize", optimize(size))]
fn raise_dumps_exception_fixed(msg: &str) -> *mut PyObject {
    unsafe {
        let err_msg =
            PyUnicode_FromStringAndSize(msg.as_ptr() as *const c_char, msg.len() as isize);
        PyErr_SetObject(typeref::JsonEncodeError, err_msg);
        debug_assert!(ffi!(Py_REFCNT(err_msg)) <= 2);
        Py_DECREF(err_msg);
    };
    null_mut()
}

#[cold]
#[inline(never)]
#[cfg_attr(feature = "optimize", optimize(size))]
#[cfg(Py_3_12)]
fn raise_dumps_exception_dynamic(err: &str) -> *mut PyObject {
    unsafe {
        let cause_exc: *mut PyObject = PyErr_GetRaisedException();

        let err_msg =
            PyUnicode_FromStringAndSize(err.as_ptr() as *const c_char, err.len() as isize);
        PyErr_SetObject(typeref::JsonEncodeError, err_msg);
        debug_assert!(ffi!(Py_REFCNT(err_msg)) <= 2);
        Py_DECREF(err_msg);

        if !cause_exc.is_null() {
            let exc: *mut PyObject = PyErr_GetRaisedException();
            PyException_SetCause(exc, cause_exc);
            PyErr_SetRaisedException(exc);
        }
    };
    null_mut()
}

#[cold]
#[inline(never)]
#[cfg_attr(feature = "optimize", optimize(size))]
#[cfg(not(Py_3_12))]
fn raise_dumps_exception_dynamic(err: &str) -> *mut PyObject {
    unsafe {
        let mut cause_tp: *mut PyObject = null_mut();
        let mut cause_val: *mut PyObject = null_mut();
        let mut cause_traceback: *mut PyObject = null_mut();
        PyErr_Fetch(&mut cause_tp, &mut cause_val, &mut cause_traceback);

        let err_msg =
            PyUnicode_FromStringAndSize(err.as_ptr() as *const c_char, err.len() as isize);
        PyErr_SetObject(typeref::JsonEncodeError, err_msg);
        debug_assert!(ffi!(Py_REFCNT(err_msg)) == 2);
        Py_DECREF(err_msg);
        let mut tp: *mut PyObject = null_mut();
        let mut val: *mut PyObject = null_mut();
        let mut traceback: *mut PyObject = null_mut();
        PyErr_Fetch(&mut tp, &mut val, &mut traceback);
        PyErr_NormalizeException(&mut tp, &mut val, &mut traceback);

        if !cause_tp.is_null() {
            PyErr_NormalizeException(&mut cause_tp, &mut cause_val, &mut cause_traceback);
            PyException_SetCause(val, cause_val);
            Py_DECREF(cause_tp);
        }
        if !cause_traceback.is_null() {
            Py_DECREF(cause_traceback);
        }

        PyErr_Restore(tp, val, traceback);
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
        return raise_dumps_exception_fixed(
            "dumps() missing 1 required positional argument: 'obj'",
        );
    }
    if num_args & 2 == 2 {
        default = Some(NonNull::new_unchecked(*args.offset(1)));
    }
    if num_args & 3 == 3 {
        optsptr = Some(NonNull::new_unchecked(*args.offset(2)));
    }
    if unlikely!(!kwnames.is_null()) {
        for i in 0..=Py_SIZE(kwnames).saturating_sub(1) {
            let arg = PyTuple_GET_ITEM(kwnames, i as Py_ssize_t);
            if arg == typeref::DEFAULT {
                if unlikely!(num_args & 2 == 2) {
                    return raise_dumps_exception_fixed(
                        "dumps() got multiple values for argument: 'default'",
                    );
                }
                default = Some(NonNull::new_unchecked(*args.offset(num_args + i)));
            } else if arg == typeref::OPTION {
                if unlikely!(num_args & 3 == 3) {
                    return raise_dumps_exception_fixed(
                        "dumps() got multiple values for argument: 'option'",
                    );
                }
                optsptr = Some(NonNull::new_unchecked(*args.offset(num_args + i)));
            } else {
                return raise_dumps_exception_fixed("dumps() got an unexpected keyword argument");
            }
        }
    }

    let mut optsbits: i32 = 0;
    if unlikely!(optsptr.is_some()) {
        let opts = optsptr.unwrap();
        if (*opts.as_ptr()).ob_type == typeref::INT_TYPE {
            optsbits = PyLong_AsLong(optsptr.unwrap().as_ptr()) as i32;
            if unlikely!(!(0..=opt::MAX_OPT).contains(&optsbits)) {
                return raise_dumps_exception_fixed("Invalid opts");
            }
        } else if unlikely!(opts.as_ptr() != typeref::NONE) {
            return raise_dumps_exception_fixed("Invalid opts");
        }
    }

    match crate::serialize::serialize(*args, default, optsbits as opt::Opt) {
        Ok(val) => val.as_ptr(),
        Err(err) => raise_dumps_exception_dynamic(err.as_str()),
    }
}
