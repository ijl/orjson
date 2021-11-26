use std::process::Command;

fn main() {
    let out = Command::new("python")
        .args(&["-c", "import sys; print(sys.version_info[1])"])
        .output()
        .expect("python version did not print");
    let version = u8::from_str_radix(String::from_utf8_lossy(&out.stdout).trim(), 10)
        .expect("python version was not parsed");
    for each in 8..(version + 1) {
        println!("cargo:rustc-cfg=Py_3_{}", each);
    }
}
