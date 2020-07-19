use std::os::raw::c_void;
use std::sync::Arc;
use libc::size_t;
use crate::bridge::{GoBuild, GoSlice, GoString};
use crate::wrapper::{BuildOptions, BuildResult, Message, OutputFile, SliceContainer};

struct BuildInvocationData {
    opt_arc_raw: *const BuildOptions,
    cb_trait_ptr: *mut c_void,
}

extern "C" fn build_callback(
    raw_cb_data: *mut c_void,
    raw_output_files: *mut OutputFile,
    output_files_len: size_t,
    raw_errors: *mut Message,
    errors_len: size_t,
    raw_warnings: *mut Message,
    warnings_len: size_t,
) -> () {
    unsafe {
        let cb_data: Box<BuildInvocationData> = Box::from_raw(raw_cb_data as *mut _);

        // Drop refcount.
        let _: Arc<BuildOptions> = Arc::from_raw(cb_data.opt_arc_raw);

        let rust_cb_trait_box: Box<Box<dyn FnOnce(BuildResult)>>
            = Box::from_raw(cb_data.cb_trait_ptr as *mut _);

        let output_files = SliceContainer {
            ptr: raw_output_files,
            len: output_files_len,
        };
        let errors = SliceContainer {
            ptr: raw_errors,
            len: errors_len,
        };
        let warnings = SliceContainer {
            ptr: raw_warnings,
            len: warnings_len,
        };

        rust_cb_trait_box(BuildResult {
            output_files,
            errors,
            warnings,
        });
    };
}

pub fn build<F>(options: Arc<BuildOptions>, cb: F) -> ()
    where F: FnOnce(BuildResult),
          F: Send + 'static,
{
    // Prepare callback.
    let cb_box = Box::new(cb) as Box<dyn FnOnce(BuildResult)>;
    let cb_trait_box = Box::new(cb_box);
    let cb_trait_ptr = Box::into_raw(cb_trait_box);

    let data = Box::into_raw(Box::new(BuildInvocationData {
        opt_arc_raw: Arc::into_raw(options.clone()),
        cb_trait_ptr: cb_trait_ptr as *mut c_void,
    }));

    unsafe {
        #[cfg(target_env = "msvc")]
        #[allow(non_snake_case)]
        let GoBuild = std::mem::transmute::<_, GoBuild>(crate::bridge::DLL.get_function("GoBuild"));

        // We can safely convert anything in BuildOptions into raw pointers, as the memory is managed the the Arc and we only used owned values.
        GoBuild(
            libc::malloc,
            build_callback,
            data as *mut c_void,

            options.source_map as u8,
            options.target as u8,
            options.engines.vec.as_ptr(),
            options.engines.vec.len(),
            options.strict.nullish_coalescing,
            options.strict.class_fields,

            options.minify_whitespace,
            options.minify_identifiers,
            options.minify_syntax,

            GoString::from_str_unmanaged(&options.jsx_factory),
            GoString::from_str_unmanaged(&options.jsx_fragment),

            options.defines.vec.as_ptr(),
            options.defines.vec.len(),
            GoSlice::from_vec_unamanged(&options.pure_functions.vec),

            GoString::from_str_unmanaged(&options.global_name),
            options.bundle,
            options.splitting,
            GoString::from_str_unmanaged(&options.outfile),
            GoString::from_str_unmanaged(&options.metafile),
            GoString::from_str_unmanaged(&options.outdir),
            options.platform as u8,
            options.format as u8,
            GoSlice::from_vec_unamanged(&options.externals.vec),
            options.loaders.vec.as_ptr(),
            options.loaders.vec.len(),
            GoSlice::from_vec_unamanged(&options.resolve_extensions.vec),
            GoString::from_str_unmanaged(&options.tsconfig),

            GoSlice::from_vec_unamanged(&options.entry_points.vec),
        );
    }
}
