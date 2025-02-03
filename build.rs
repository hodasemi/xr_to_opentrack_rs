fn static_library(library_name: &str) {
    println!("cargo:rustc-link-lib=static={}", library_name);
}

fn dynamic_library(library_name: &str) {
    println!("cargo:rustc-link-lib=dylib={}", library_name);
}

fn native_path(path: &str) {
    println!("cargo:rustc-link-search=native={}", path);
}

fn main() {
    native_path("viture_one_linux_sdk_1.0.7/libs");
    static_library("viture_one_sdk_static");
    dynamic_library("usb-1.0");
}
