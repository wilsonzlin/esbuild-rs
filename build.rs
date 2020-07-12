use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn set_readonly(path: &PathBuf, readonly: bool) -> () {
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(readonly);
    fs::set_permissions(&path, permissions).expect(&format!("setting permissions on {:?}", path))
}

pub fn force_remove_file(path: &PathBuf) -> () {
    set_readonly(&path, false);
    fs::remove_file(&path).expect(&format!("removing file {:?}", path));
}

fn force_remove_dir(path: &PathBuf) -> () {
    set_readonly(&path, false);
    fs::remove_dir(&path).expect(&format!("removing dir {:?}", path));
}

fn force_remove_dir_all(path: &PathBuf) -> () {
    // We can't remove files in this directory if it is readonly.
    set_readonly(&path, false);
    for child in fs::read_dir(&path).unwrap() {
        let child = child.unwrap();
        let metadata = child.metadata().unwrap();
        let path = child.path();
        if metadata.is_dir() {
            force_remove_dir_all(&path);
        } else if metadata.is_file() {
            force_remove_file(&path);
        };
    };
    force_remove_dir(&path);
}

#[cfg(feature = "use-dylib")]
fn main() {}

#[cfg(not(feature = "use-dylib"))]
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let gopath = format!("{}/gopath", out_dir);
    let out_name = if cfg!(target_os = "windows") {
        "esbuild.lib"
    } else {
        "libesbuild.a"
    };

    Command::new("go")
        .env("GOPATH", gopath.clone())
        .current_dir("lib")
        .arg("get")
        .arg("./");

    Command::new("go")
        .env("GOPATH", gopath.clone())
        .current_dir("lib")
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(format!("{}/{}", out_dir, out_name))
        .arg("esbuild.go")
        .status()
        .expect("compile Go library");

    // Otherwise Cargo will complain that we've modified files outside OUT_DIR.
    fs::remove_file("lib/go.sum").expect("remove go.sum");
    // Go package manager makes dependency files read only, causing issues with rebuilding and
    // clearing.
    force_remove_dir_all(&PathBuf::from(gopath));

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static={}", "esbuild");
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework={}", "CoreFoundation");
        println!("cargo:rustc-link-lib=framework={}", "Security");
    };
}
