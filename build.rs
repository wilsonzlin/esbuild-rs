use std::process::Command;

fn main() {
    Command::new("go")
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg("libesbuild.a")
        .arg("esbuild.go")
        .status()
        .expect("compile Go library");

    let path = "./";
    let lib = "esbuild";

    println!("cargo:rustc-link-search=native={}", path);
    println!("cargo:rustc-link-lib=static={}", lib);
}
