use std::env;
use std::fs;
use std::process::Command;

use force_remove::force_remove_dir_all;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let gopath = format!("{}/gopath", out_dir);

    Command::new("go")
        .env("GOPATH", gopath.clone())
        .arg("get")
        .arg("./");

    Command::new("go")
        .env("GOPATH", gopath.clone())
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(format!("{}/{}", out_dir, "libesbuild.a"))
        .arg("esbuild.go")
        .status()
        .expect("compile Go library");

    // Otherwise Cargo will complain that we've modified files outside OUT_DIR.
    fs::remove_file("go.sum").expect("remove go.sum");
    // Go package manager makes dependency files read only, causing issues with rebuilding and
    // clearing.
    force_remove_dir_all(gopath.clone()).expect("remove temporary GOPATH dir");

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", "esbuild");
}
