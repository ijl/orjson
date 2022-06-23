// SPDX-License-Identifier: (Apache-2.0 OR MIT)

#[allow(dead_code)]
#[cfg(feature = "yyjson")]
fn build_yyjson() {
    cc::Build::new()
        .file("include/yyjson/yyjson.c")
        .include("include/yyjson")
        .define("YYJSON_DISABLE_WRITER", "1")
        .define("YYJSON_DISABLE_NON_STANDARD", "1")
        .compile("yyjson");
}

fn main() {
    pyo3_build_config::use_pyo3_cfgs();

    #[cfg(feature = "yyjson")]
    build_yyjson();
}
