use std::process::Command;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("go")
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(format!("{}/{}", out_dir, "libesbuild.a"))
        .arg("esbuild.go")
        .status()
        .expect("compile Go library");

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", "esbuild");
}
