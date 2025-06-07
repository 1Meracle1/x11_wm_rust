use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=x11_headers.h");

    for lib in [
        "X11",
        "X11-xcb",
        "Xcursor",
        "xcb",
        "xcb-cursor",
        "xcb-icccm",
        "xcb-ewmh",
        "xcb-randr",
        "xcb-image",
        "xcb-shape",
        "X11-xcb",
        "xkbcommon",
        "xkbcommon-x11",
        "xcb-xkb",
    ] {
        println!("cargo:rustc-link-lib={}", lib);
    }

    // Generate bindings
    let bindings = bindgen::Builder::default()
        // The input header we want to generate bindings for
        .header("x11_headers.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Generate bindings
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    // let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(PathBuf::from("bindings.rs"))
        .expect("Couldn't write bindings!");
}
