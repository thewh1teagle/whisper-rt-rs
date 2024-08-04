use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=CLBlast_DIR");
    let clblast_dir = env::var("CLBlast_DIR").unwrap();
    let clblast_dir = PathBuf::from(clblast_dir);

    println!("cargo:rustc-link-search={}", clblast_dir.join("..\\..").canonicalize().unwrap().display());
    println!("cargo:warning=rustc-link-search={}", clblast_dir.join("..\\..").canonicalize().unwrap().display());
    println!("cargo:rustc-link-search=C:\\vcpkg\\packages\\opencl_x64-windows\\lib");
}