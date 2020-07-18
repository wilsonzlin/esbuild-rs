use std::sync::Arc;
use crossbeam::sync::WaitGroup;
use esbuild_rs::*;

fn main() {
    let code = b"let ax = 1".to_vec();
    let wg = WaitGroup::new();
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

    let transform_wg = wg.clone();
    transform(code, options, |TransformResult { js, errors, warnings }| {
        println!("Transform complete with {} errors and {} warnings", errors.len(), warnings.len());
        println!("{}", String::from_utf8(js).unwrap());
        drop(transform_wg);
    });

    wg.wait();
}
