use std::env;
use std::path::PathBuf;

fn main() {
    // Get the NUTTX_INCLUDE_DIR environment variable, error if not set
    let nuttx_dirs_str =
        env::var("NUTTX_INCLUDE_DIR").expect("NUTTX_INCLUDE_DIR environment variable not set (should be a single directory or a ':' separated list of directories)");

    // Split the paths by colon - this works whether there's one path or multiple paths
    // If there's no colon, this will yield a single-element vector
    let nuttx_dirs: Vec<&str> = nuttx_dirs_str.split(':').collect();

    // Get the current directory to locate wrapper.h
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let wrapper_path = current_dir.join("wrapper.h");
    let wrapper_path_str = wrapper_path.to_str().unwrap();

    // Check if wrapper.h exists
    if !wrapper_path.exists() {
        panic!(
            "wrapper.h not found at {}. Make sure the file exists in the crate root.",
            wrapper_path_str
        );
    }

    // Tell cargo to re-run this script if wrapper.h or env var changes
    println!("cargo:rerun-if-changed={}", wrapper_path_str);
    println!("cargo:rerun-if-env-changed=NUTTX_INCLUDE_DIR");

    // Also add current directory as include path for wrapper.h
    let current_include = format!("-I{}", current_dir.to_str().unwrap());

    // Create a bindgen builder
    let mut builder = bindgen::Builder::default()
        .use_core()
        // The input header we want to generate bindings for
        .header(wrapper_path_str)
        .clang_arg(&current_include)
        // Add flags to avoid standard includes and libraries
        .clang_arg("-nostdinc")
        .clang_arg("-nostdlib")
        // Tell cargo to invalidate the crate when any of these change
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    // Add all specified NuttX directories directly to the include paths
    for nuttx_dir in nuttx_dirs {
        let nuttx_dir = nuttx_dir.trim();
        if !nuttx_dir.is_empty() {
            // Use the provided directory path directly as an include path
            let include_path_str = format!("-I{}", nuttx_dir);

            // Add the include path to the builder
            builder = builder.clang_arg(&include_path_str);
        }
    }

    // Generate the bindings
    let bindings = builder.generate().expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_path = out_path.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write bindings!");
}
