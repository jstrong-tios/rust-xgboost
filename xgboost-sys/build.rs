extern crate bindgen;
extern crate cmake;

use std::process::Command;
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let xgb_root = Path::new(&out_dir).join("xgboost");
    eprintln!("xgb_root = {}", xgb_root.display());

    // copy source code into OUT_DIR for compilation if it doesn't exist
    if !xgb_root.exists() {
        Command::new("cp")
            .args(&["-r", "xgboost", xgb_root.to_str().unwrap()])
            .status()
            .unwrap_or_else(|e| {
                panic!("Failed to copy ./xgboost to {}: {}", xgb_root.display(), e);
            });
    }

    let cmake_dst = cmake::Config::new(&xgb_root)
        .define("BUILD_STATIC_LIB", "ON")
        .build();

    println!("cargo:rustc-link-search=native={}", cmake_dst.display());
    println!("cargo:rustc-link-search=native={}", cmake_dst.join("lib").display());

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", cmake_dst.join("include").display()))
        .clang_arg(format!("-I{}", cmake_dst.join("rabit/include").display()))
        .generate()
        .expect("Unable to generate bindings.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings.");

    // check if built with multithreading support, otherwise link to dummy lib
    if cmake_dst.join("lib/librabit.a").exists() {
        println!("cargo:rustc-link-lib=static=rabit");
        println!("cargo:rustc-link-lib=dylib=gomp");
    } else {
        println!("cargo:rustc-link-lib=static=rabit_empty");
    }

    // link to appropriate C++ lib
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }

    println!("cargo:rustc-link-lib=static=dmlc");
    println!("cargo:rustc-link-lib=static=xgboost");
}
