use rand;
use std::env;

fn main() {
    let build_id: u32 = rand::random();
    println!("cargo:rustc-env=BUILD_ID={:08x}", build_id);
    // Rerun if any source files change
    println!("cargo:rerun-if-changed=src");
    // Rerun if Cargo.toml changes
    println!("cargo:rerun-if-changed=Cargo.toml");
    // Rerun if build script changes
    println!("cargo:rerun-if-changed=build.rs");

    // Get target info
    let target = env::var("TARGET").unwrap_or_else(|_| String::from("unknown"));
    
    // Only add Windows/Cygwin paths when targeting Windows
    if !target.contains("linux") && !target.contains("wsl") {
        // Tell cargo to look for libraries in the Cygwin paths
        println!("cargo:rustc-link-search=C:/cygwin64/lib");
        println!("cargo:rustc-link-search=C:/cygwin64/usr/lib");
        
        // For the ring crate specifically
        println!("cargo:rustc-env=OPENSSL_DIR=C:/cygwin64/usr");
        println!("cargo:rustc-env=OPENSSL_LIB_DIR=C:/cygwin64/usr/lib");
        println!("cargo:rustc-env=OPENSSL_INCLUDE_DIR=C:/cygwin64/usr/include");
    } else {
        // Linux-specific configurations (if any needed)
        println!("cargo:rustc-link-arg=-Wl,--no-as-needed");
    }
}
