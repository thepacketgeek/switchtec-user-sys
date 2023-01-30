use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(&out_dir);
    let orig_dir = env::current_dir().unwrap();

    // Check for clang dependency
    if Command::new("clang").arg("-v").output().is_err() {
        eprintln!("Clang is required for bindgen, please check installation instructions: https://rust-lang.github.io/rust-bindgen/requirements.html");
        std::process::exit(1);
    }

    // Make sure that switchtec-user submodule is available locally
    Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .output()
        .expect("couldn't download switchtec-user submodule");

    // Generate Rust Bindings for C Library
    let bindings = bindgen::Builder::default()
        .header("switchtec-user/inc/switchtec/switchtec.h")
        .clang_arg("-Iswitchtec-user/inc")
        .rustfmt_bindings(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Unable to save bindings");

    // Compile switchtec-user library
    env::set_current_dir(&out_path).unwrap();

    let root_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root_path: PathBuf = [&root_dir, "switchtec-user", "configure"].iter().collect();
    Command::new(root_path)
        .output()
        .expect("couldn't run ./configure");

    env::set_current_dir(orig_dir).unwrap();

    cc::Build::new()
        .include("switchtec-user/inc")
        .include(&out_dir)
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
