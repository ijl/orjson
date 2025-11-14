// SPDX-License-Identifier: (Apache-2.0 OR MIT)

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=include/yyjson/*");
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");
    println!("cargo:rerun-if-env-changed=LDFLAGS");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    println!("cargo:rustc-check-cfg=cfg(cold_path)");
    println!("cargo:rustc-check-cfg=cfg(CPython)");
    println!("cargo:rustc-check-cfg=cfg(GraalPy)");
    println!("cargo:rustc-check-cfg=cfg(optimize)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_10)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_11)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_12)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_13)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_14)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_15)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_9)");
    println!("cargo:rustc-check-cfg=cfg(Py_GIL_DISABLED)");
    println!("cargo:rustc-check-cfg=cfg(PyPy)");

    let python_config = pyo3_build_config::get();
    for cfg in python_config.build_script_outputs() {
        println!("{cfg}");
    }

    #[allow(unused_variables)]
    let is_64_bit_python = matches!(python_config.pointer_width, Some(64));

    match python_config.implementation {
        pyo3_build_config::PythonImplementation::CPython => {
            println!("cargo:rustc-cfg=CPython");
            #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
            if is_64_bit_python {
                println!("cargo:rustc-cfg=feature=\"inline_int\"");
            }
        }
        pyo3_build_config::PythonImplementation::GraalPy => {
            panic!("orjson does not support GraalPy")
        }
        pyo3_build_config::PythonImplementation::PyPy => {
            panic!("orjson does not support PyPy")
        }
    }

    #[cfg(all(target_arch = "x86_64", not(target_os = "macos")))]
    if version_check::is_min_version("1.89.0").unwrap_or(false) && is_64_bit_python {
        println!("cargo:rustc-cfg=feature=\"avx512\"");
    }

    if version_check::supports_feature("cold_path").unwrap_or(false) {
        println!("cargo:rustc-cfg=feature=\"cold_path\"");
    }

    if version_check::supports_feature("optimize_attribute").unwrap_or(false) {
        println!("cargo:rustc-cfg=feature=\"optimize\"");
    }

    cc::Build::new()
        .file("include/yyjson/yyjson.c")
        .include("include/yyjson")
        .define("YYJSON_DISABLE_NON_STANDARD", "1")
        .define("YYJSON_DISABLE_UTF8_VALIDATION", "1")
        .define("YYJSON_DISABLE_UTILS", "1")
        .define("YYJSON_DISABLE_WRITER", "1")
        .compile("yyjson")
}
