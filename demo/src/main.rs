use esbuild_rs::esbuild;

fn main() {
    println!("{}", std::str::from_utf8(esbuild(b"let ax = 1;")).unwrap());
}
