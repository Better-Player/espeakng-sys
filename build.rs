extern crate bindgen;

use std::env;
use std::path::PathBuf;

static STATIC_LIBRARIES: &[&str] = &[
    "stdc++",
    "sonic",
    "pthread",
    "pcaudio",
    "asound",
    "libc",
    "espeak-ng",
];

fn add_arch(arch: &str) {
    println!("cargo:rustc-link-search=native=/usr/lib/{arch}");
    println!("cargo:rustc-link-search=native=/usr/lib/gcc/{arch}/10");
    println!("cargo:rustc-link-search=native=/usr/lib/gcc/{arch}/11");
}

fn add_search_paths() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match target_arch.as_str() {
        "aarch64" => {
            add_arch("aarch64-linux-gnu");
        }
        "x86" | "x86_64" => {
            add_arch("x86_64-linux-gnu");
        }
        _ => panic!("Unsupported architecture: {}", target_arch),
    }
    println!("cargo:rustc-link-search=native=/usr/lib");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
}

fn main() {
    if cfg!(feature = "static") {
        add_search_paths();
        for lib in STATIC_LIBRARIES {
            println!("cargo:rustc-link-lib=static={lib}");
        }
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
