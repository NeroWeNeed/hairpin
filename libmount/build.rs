use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=mount");
    let bindings = bindgen::Builder::default()
        .header("libmount_source.h")
        .blocklist_type("_bindgen_ty_1")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_default(true)
        .disable_name_namespacing()
        .enable_cxx_namespaces()
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
