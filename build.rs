extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn build_static_espeak_ng() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let espeak_dir = format!("{}/espeak-ng", out_dir);
    let espeak_build_dir = format!("{}/build", &espeak_dir);
    let espeak_lib_dir = format!("{}/lib", &espeak_build_dir);

    Command::new("git")
        .args([
            "clone",
            "https://github.com/espeak-ng/espeak-ng",
            &espeak_dir,
        ])
        .status()
        .expect("Failed to clone espeak-ng");
    Command::new("./autogen.sh")
        .current_dir(&espeak_dir)
        .status()
        .expect("Failed to run autogen for espeak-ng");
    Command::new("./configure")
        .args([
            "--prefix",
            &espeak_build_dir,
            "--without-sonic",
            "--without-pcaudiolib",
            "--without-speechplayer",
            "--enable-static",
            "--disable-shared",
        ])
        .current_dir(&espeak_dir)
        .status()
        .expect("Failed to configure espeak-ng");
    Command::new("make")
        .current_dir(&espeak_dir)
        .status()
        .expect("Failed to build espeak-ng");
    Command::new("make")
        .args(&["install"])
        .current_dir(&espeak_dir)
        .status()
        .expect("Failed to install espeak-ng");

    println!("cargo:rustc-link-search=native={}", espeak_lib_dir);
    println!("cargo:rustc-link-lib=static=espeak-ng");
}

fn main() {
    if cfg!(feature = "static") {
        build_static_espeak_ng();
    } else {
        println!("cargo:rustc-link-lib=espeak-ng");
    }

    println!("cargo:rerun-if-changed=headers/wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("headers/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
