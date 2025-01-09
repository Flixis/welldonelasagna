use rand;

fn main() {
    let build_id: u32 = rand::random();
    println!("cargo:rustc-env=BUILD_ID={:08x}", build_id);
    println!("cargo:rerun-if-changed=build.rs");
}
