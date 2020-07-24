use crossbeam::sync::WaitGroup;
use esbuild_rs::*;
use std::sync::Arc;
use std::collections::HashMap;

fn run_build() {
    let wg = WaitGroup::new();
    let mut entry_points = Vec::<String>::new();
    entry_points.push("samplejs/input.js".to_string());
    let options_builder = BuildOptionsBuilder {
        source_map: SourceMap::None,
        target: Target::ESNext,
        engines: Vec::new(),
        strict: StrictOptions {
            nullish_coalescing: true,
            class_fields: true,
        },
        minify_whitespace: true,
        minify_identifiers: true,
        minify_syntax: true,
        jsx_factory: "".to_string(),
        jsx_fragment: "".to_string(),
        defines: HashMap::new(),
        pure_functions: Vec::new(),
        global_name: "".to_string(),
        bundle: true,
        splitting: false,
        outfile: "samplejs/output.js".to_string(),
        metafile: "".to_string(),
        outdir: "".to_string(),
        platform: Platform::Browser,
        format: Format::IIFE,
        externals: Vec::new(),
        loaders: HashMap::new(),
        resolve_extensions: Vec::new(),
        tsconfig: "".to_string(),
        entry_points,
    };
    let options = options_builder.build();

    let build_wg = wg.clone();
    build_direct(options, |BuildResult { output_files, errors, warnings }| {
        println!("Build complete");
        println!("Errors:");
        for msg in errors.as_slice() {
            println!("{}", msg);
        };
        println!("Warnings:");
        for msg in warnings.as_slice() {
            println!("{}", msg);
        };
        for file in output_files.as_slice() {
            println!("Output file {}: {}", file.path.as_str(), file.data.as_str());
        };
        drop(build_wg);
    });

    wg.wait();
}

fn run_transform() {
    let code = Arc::new(br#"
        let ax = 1;

        const x = NAME;

        alert();
        // Intentionally without semicolon.
        important()
        console.log();

        React.render(
            <div></div>
        );
    "#.to_vec());
    let wg = WaitGroup::new();

    let mut defines = HashMap::<String, String>::new();
    defines.insert("NAME".to_string(), "'myname'".to_string());

    let mut pure_functions = Vec::<String>::new();
    pure_functions.push("alert".to_string());
    pure_functions.push("console.log".to_string());

    let options_builder = TransformOptionsBuilder {
        source_map: SourceMap::None,
        target: Target::ESNext,
        engines: Vec::new(),
        strict: StrictOptions {
            nullish_coalescing: true,
            class_fields: true,
        },
        minify_whitespace: true,
        minify_identifiers: true,
        minify_syntax: true,
        jsx_factory: "React.createComponent".to_string(),
        jsx_fragment: "".to_string(),
        defines,
        pure_functions,
        source_file: "Rust literal raw string".to_string(),
        loader: Loader::JSX,
    };
    let options = options_builder.build();

    let transform_wg = wg.clone();
    transform_direct(code, options, |TransformResult { js, js_source_map, errors, warnings }| {
        println!("Transform complete");
        println!("Errors:");
        for msg in errors.as_slice() {
            println!("{}", msg);
        };
        println!("Warnings:");
        for msg in warnings.as_slice() {
            println!("{}", msg);
        };
        println!("Result:");
        println!("{}", js.as_str());
        println!("Source map:");
        println!("{}", js_source_map.as_str());
        drop(transform_wg);
    });

    wg.wait();
}

fn main() {
    run_build();
    println!();
    run_transform();
}
