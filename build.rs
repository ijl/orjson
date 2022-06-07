// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[cfg(feature = "yyjson")]
extern crate cc;

#[allow(dead_code)]
#[cfg(feature = "yyjson")]
fn build_yyjson() {
    // note -march is not automatically propagated by -C target-cpu
    cc::Build::new()
        .file("include/yyjson.c")
        .include("include")
        .define("YYJSON_DISABLE_WRITER", "1")
        .define("YYJSON_DISABLE_NON_STANDARD", "1")
        .compiler("clang")
        .static_flag(true)
        .use_plt(false)
        .opt_level_str("2")
        .flag("-flto=thin")
        .compile("yyjson");
}

fn main() {
    pyo3_build_config::use_pyo3_cfgs();

    #[cfg(feature = "yyjson")]
    build_yyjson();
}
