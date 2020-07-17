use std::sync::Arc;
use esbuild_rs::*;

fn main() {
    let code = b"let ax = 1".to_vec();
    let options = Arc::new(TransformOptions {
        source_map: SourceMap::None,
        target: Target::ESNext,
        engines: Engines::new(),
        strict: StrictOptions {
            nullish_coalescing: true,
            class_fields: true,
        },
        minify_whitespace: true,
        minify_identifiers: true,
        minify_syntax: true,
        jsx_factory: "".to_string(),
        jsx_fragment: "".to_string(),
        defines: Defines::new(),
        pure_functions: Vec::new(),
        source_file: "".to_string(),
        loader: Loader::JS,
    });
    transform(code, options, |min, errors, warnings| {
        println!("{}", String::from_utf8(min).unwrap());
    });
}
