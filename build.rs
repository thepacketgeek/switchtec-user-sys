use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // Generate Rust Bindings for C Library
    let bindings = bindgen::Builder::default()
        .header("switchtec-user/inc/switchtec/switchtec.h")
        .clang_arg("-Iswitchtec-user/inc")
        .clang_arg("-Iswitchtec-user/lib")
        .clang_arg("-Iswitchtec-user/lib/platform")
        .clang_arg("-Iswitchtec-user/src")
        .rustfmt_bindings(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to save bindings");

    // Compile switchtec-user library
    let previous_dir = env::current_dir().unwrap();
    env::set_current_dir(&Path::new("switchtec-user")).unwrap();
    Command::new("./configure")
        .output()
        .expect("couldn't run ./configure");
    env::set_current_dir(&previous_dir).unwrap();
    cc::Build::new()
        .include("switchtec-user/inc")
        .include("switchtec-user")
        .include("switchtec-user/lib")
        .include("switchtec-user/lib/platform")
        .file("switchtec-user/lib/crc.c")
        .file("switchtec-user/lib/diag.c")
        .file("switchtec-user/lib/events.c")
        .file("switchtec-user/lib/fabric.c")
        .file("switchtec-user/lib/fw.c")
        .file("switchtec-user/lib/gas_mrpc.c")
        .file("switchtec-user/lib/mfg.c")
        .file("switchtec-user/lib/mrpc.c")
        .file("switchtec-user/lib/pmon.c")
        .file("switchtec-user/lib/switchtec.c")
        .file("switchtec-user/lib/platform/platform.c")
        .file("switchtec-user/lib/platform/linux.c")
        .file("switchtec-user/lib/platform/linux-eth.c")
        .file("switchtec-user/lib/platform/linux-i2c.c")
        .file("switchtec-user/lib/platform/linux-uart.c")
        .file("switchtec-user/lib/platform/gasops.c")
        .warnings(false)
        .extra_warnings(false)
        .compile("libswitchtec.a");
}
