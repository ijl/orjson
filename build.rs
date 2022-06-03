// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[cfg(feature = "yyjson")]
extern crate bindgen;

#[cfg(feature = "yyjson")]
extern crate cc;

#[cfg(feature = "yyjson")]
use std::path::PathBuf;

#[allow(dead_code)]
#[cfg(feature = "yyjson")]
fn build_yyjson() {
    println!("cargo:rerun-if-changed=include/yyjson.h");
    println!("cargo:rerun-if-changed=include/yyjson.c");

    let bindings = bindgen::Builder::default()
        .header("include/yyjson.h")
        .generate_comments(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_function("yyjson_alc_pool_init")
        .allowlist_function("yyjson_arr_iter_next")
        .allowlist_function("yyjson_doc_free")
        .allowlist_function("yyjson_obj_iter_next")
        .allowlist_function("yyjson_read_opts")
        .allowlist_type("yyjson_alc")
        .allowlist_type("yyjson_doc")
        .allowlist_type("yyjson_read_code")
        .allowlist_type("yyjson_read_err")
        .allowlist_type("yyjson_val")
        .allowlist_var("YYJSON_READ_NOFLAG")
        .allowlist_var("YYJSON_READ_SUCCESS")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src");
    bindings
        .write_to_file(out_path.join("yyjson.rs"))
        .expect("Couldn't write bindings!");

    // note -march is not automatically propagated by -C target-cpu
    cc::Build::new()
        .file("include/yyjson.c")
        .include("include")
        .define("YYJSON_DISABLE_WRITER", "1")
        .define("YYJSON_DISABLE_NON_STANDARD", "1")
        .compiler("clang")
        .compile("yyjson");
}

fn main() {
    pyo3_build_config::use_pyo3_cfgs();

    #[cfg(feature = "yyjson")]
    build_yyjson();
}
