use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("hello.rs");
    fs::write(
        &dest_path,
        "pub fn message() -> &'static str {
            \"Hello, World! This code is auto-generated via 'build.rs'\"
        }
        ",
    )
    .unwrap();
    println!("cargo:rerun-if-changed=build.rs");

    // Check if we're building on the Windows platform, and if so, it'll be "DLL" based rather than "SO"
    if env::var("CARGO_CFG_TARGET_FAMILY").unwrap() == "windows" {
        // Tell Rust where to find SDL2.dll
        let sdl2_path = ".\\lib";
        println!("cargo:rustc-link-search=native={}", sdl2_path);

        // Tell Rust to link to SDL2.dll
        println!("cargo:rustc-link-lib=dylib=SDL2");
    }
}
