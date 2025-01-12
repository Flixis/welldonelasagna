use rand;

fn main() {
    let build_id: u32 = rand::random();
    println!("cargo:rustc-env=BUILD_ID={:08x}", build_id);
    // Rerun if any source files change
    println!("cargo:rerun-if-changed=src");
    // Rerun if Cargo.toml changes
    println!("cargo:rerun-if-changed=Cargo.toml");
    // Rerun if build script changes
    println!("cargo:rerun-if-changed=build.rs");
}
