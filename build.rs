fn main() {
    let path = "./";
    let lib = "esbuild";

    println!("cargo:rustc-link-search=native={}", path);
    println!("cargo:rustc-link-lib=static={}", lib);
}
