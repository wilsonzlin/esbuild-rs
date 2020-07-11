use esbuild_rs::esbuild;

fn main() {
    println!("{}", esbuild("let ax = 1;"));
}
