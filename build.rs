use std::process::Command;

fn main() {
    Command::new("go")
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(format!("{}/{}", env!("OUT_DIR"), "libesbuild.a"))
        .arg("esbuild.go")
        .status()
        .expect("compile Go library");

    println!("cargo:rustc-link-search=native={}", env!("OUT_DIR"));
    println!("cargo:rustc-link-lib=static={}", "esbuild");
}
