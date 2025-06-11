use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=usb-1.0");
    let bindings = bindgen::Builder::default()
        .header("libusb_source.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
