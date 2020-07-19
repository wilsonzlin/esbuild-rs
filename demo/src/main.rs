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

    let mut defines = Defines::new();
    defines.add("NAME".to_string(), "'myname'".to_string());

    let mut pure_functions = PureFunctions::new();
    pure_functions.add("alert".to_string());
    pure_functions.add("console.log".to_string());

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
        jsx_factory: "React.createComponent".to_string(),
        jsx_fragment: "".to_string(),
        defines,
        pure_functions,
        source_file: "Rust literal raw string".to_string(),
        loader: Loader::JSX,
    });

    let transform_wg = wg.clone();
    transform(code, options, |TransformResult { js, js_source_map, errors, warnings }| {
        println!("Transform complete");
        println!("Errors:");
        for msg in &*errors {
            println!("{}", msg);
        };
        println!("Warnings:");
        for msg in &*warnings {
            println!("{}", msg);
        };
        println!("Result:");
        println!("{}", &*js);
        println!("Source map:");
        println!("{}", &*js_source_map);
        drop(transform_wg);
    });

    wg.wait();
}

fn main() {
    run_build();
    println!();
    run_transform();
}
