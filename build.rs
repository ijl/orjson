// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2021-2026)

fn main() {
    let python_config = pyo3_build_config::get();

    if python_config.is_free_threaded() && std::env::var("ORJSON_BUILD_FREETHREADED").is_err() {
        not_supported("free-threaded Python")
    }

    #[allow(unused_variables)]
    let is_64_bit_python = matches!(python_config.pointer_width(), Some(64));

    match python_config.implementation() {
        pyo3_build_config::PythonImplementation::CPython => {
            println!("cargo:rustc-cfg=CPython");
            let is_abi3 = matches!(
                python_config.target_abi().kind(),
                pyo3_build_config::PythonAbiKind::Stable(_)
            );
            if is_abi3 {
                println!("cargo:rustc-cfg=Py_LIMITED_ABI");
            }
            #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
            if is_64_bit_python && !is_abi3 {
                println!("cargo:rustc-cfg=feature=\"inline_int\"");
                #[cfg(target_endian = "little")]
                println!("cargo:rustc-cfg=feature=\"inline_str\"");
            }
        }
        _ => not_supported(&python_config.implementation().to_string()),
    }

    for cfg in python_config.build_script_outputs() {
        println!("{cfg}");
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=include/yyjson/*");
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");
    println!("cargo:rerun-if-env-changed=LDFLAGS");
    println!("cargo:rerun-if-env-changed=ORJSON_BUILD_FREETHREADED");
    println!("cargo:rerun-if-env-changed=RUSTFLAGS");
    println!("cargo:rustc-check-cfg=cfg(CPython)");
    println!("cargo:rustc-check-cfg=cfg(GraalPy)");
    println!("cargo:rustc-check-cfg=cfg(nightly)");
    println!("cargo:rustc-check-cfg=cfg(optimize)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_10)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_11)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_12)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_13)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_14)");
    println!("cargo:rustc-check-cfg=cfg(Py_3_15)");
    println!("cargo:rustc-check-cfg=cfg(Py_GIL_DISABLED)");
    println!("cargo:rustc-check-cfg=cfg(Py_LIMITED_ABI)");
    println!("cargo:rustc-check-cfg=cfg(PyPy)");
    println!("cargo:rustc-check-cfg=cfg(trusted_len)");

    #[cfg(all(target_arch = "x86_64", not(target_os = "macos")))]
    if is_64_bit_python {
        println!("cargo:rustc-cfg=feature=\"avx512\"");
    }

    cc::Build::new()
        .file("include/yyjson/yyjson.c")
        .include("include/yyjson")
        .define("YYJSON_DISABLE_INCR_READER", "1")
        .define("YYJSON_DISABLE_NON_STANDARD", "1")
        .define("YYJSON_DISABLE_UTF8_VALIDATION", "1")
        .define("YYJSON_DISABLE_UTILS", "1")
        .define("YYJSON_DISABLE_WRITER", "1")
        .define("YYJSON_READER_DEPTH_LIMIT", "1024")
        .compile("yyjson")
}

fn not_supported(flavor: &str) {
    let version = env!("CARGO_PKG_VERSION");
    eprintln!("\n\n\norjson v{version} does not support {flavor}\n\n\n");
    std::process::exit(1);
}
