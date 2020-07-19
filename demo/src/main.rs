use crossbeam::sync::WaitGroup;
use esbuild_rs::*;
use std::sync::Arc;

fn run_build() {
    let wg = WaitGroup::new();
    let mut entry_points = EntryPoints::new();
    entry_points.add("samplejs/input.js".to_string());
    let options = Arc::new(BuildOptions {
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
        pure_functions: PureFunctions::new(),
        global_name: "".to_string(),
        bundle: true,
        splitting: false,
        outfile: "samplejs/output.js".to_string(),
        metafile: "".to_string(),
        outdir: "".to_string(),
        platform: Platform::Browser,
        format: Format::IIFE,
        externals: Externals::new(),
        loaders: Loaders::new(),
        resolve_extensions: ResolveExtensions::new(),
        tsconfig: "".to_string(),
        entry_points,
    });

    let build_wg = wg.clone();
    build(options, |BuildResult { output_files, errors, warnings }| {
        println!("Build complete");
        println!("Errors:");
        for msg in &*errors {
            println!("{}", msg);
        };
        println!("Warnings:");
        for msg in &*warnings {
            println!("{}", msg);
        };
        println!("{} output files", output_files.len());
        drop(build_wg);
    });

    wg.wait();
}

fn run_transform() {
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
        pure_functions: PureFunctions::new(),
        source_file: "".to_string(),
        loader: Loader::JS,
    });

    let transform_wg = wg.clone();
    transform(code, options, |TransformResult { js, errors, warnings }| {
        println!("Transform complete");
        println!("Errors:");
        for msg in &*errors {
            println!("{}", msg);
        };
        println!("Warnings:");
        for msg in &*warnings {
            println!("{}", msg);
        };
        println!("{}", String::from_utf8(js).unwrap());
        drop(transform_wg);
    });

    wg.wait();
}

fn main() {
    run_build();
    println!();
    run_transform();
}
