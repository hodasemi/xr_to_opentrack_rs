// use std::{env, path::PathBuf};

fn static_library(library_name: &str) {
    println!("cargo:rustc-link-lib=static={}", library_name);
}

fn dynamic_library(library_name: &str) {
    println!("cargo:rustc-link-lib=dylib={}", library_name);
}

fn native_path(path: &str) {
    println!("cargo:rustc-link-search=native={}", path);
}

// fn create_viture_bindings(header: &str) {
//     // The bindgen::Builder is the main entry point
//     // to bindgen, and lets you build up options for
//     // the resulting bindings.
//     let bindings = bindgen::Builder::default()
//         // The input header we would like to generate
//         // bindings for.
//         .header(header)
//         .clang_arg("-std=c23")
//         // Tell cargo to invalidate the built crate whenever any of the
//         // included header files changed.
//         .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
//         // Finish the builder and generate the bindings.
//         .generate()
//         // Unwrap the Result and panic on failure.
//         .expect("Unable to generate bindings");

//     // Write the bindings to the $OUT_DIR/bindings.rs file.
//     let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
//     bindings
//         .write_to_file(out_path.join("bindings.rs"))
//         .expect("Couldn't write bindings!");
// }

fn main() {
    native_path("viture_one_linux_sdk_1.0.7/libs");
    dynamic_library("viture_one_sdk");

    // create_viture_bindings("viture_one_linux_sdk_1.0.7/include/viture.h");
}
