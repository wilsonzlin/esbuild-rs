use std::env;
use std::process::Command;

fn main() {
    let use_dll = cfg!(feature = "use-dll");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_name = if use_dll {
        "esbuild.dll"
    } else {
        // This is the name on Windows as well because we use the gnu toolchain.
        "libesbuild.a"
    };

    Command::new("go")
        .current_dir("lib")
        .arg("build")
        .arg("-mod=vendor")
        .arg(if use_dll {
            "-buildmode=c-shared"
        } else {
            "-buildmode=c-archive"
        })
        .arg("-o")
        .arg(format!("{}/{}", out_dir, out_name))
        .arg("vendor/github.com/evanw/esbuild/pkg/ffiapi/ffiapi.go")
        .status()
        .expect("compile Go library");

    if !use_dll {
        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=static=esbuild");
        if cfg!(target_os = "macos") {
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=Security");
        };
    };
}
