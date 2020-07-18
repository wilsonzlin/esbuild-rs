use std::mem;
use std::os::raw::{c_char, c_void};
use std::sync::Arc;

use libc::{ptrdiff_t, size_t};

use crate::bridge::{FfiapiMessage, GoSlice, GoString, GoTransform};
pub use crate::prelude::*;

mod bridge;
mod prelude;

struct TransformInvocationData {
    src_code_ptr: *mut u8,
    src_code_len: usize,
    src_code_cap: usize,

    opt_ptr: *const TransformOptions,

    cb_trait_ptr: *mut c_void,
}

extern "C" fn transform_callback(
    raw_cb_data: *mut c_void,
    out_len: size_t,
    raw_errors: *mut FfiapiMessage,
    errors_len: size_t,
    raw_warnings: *mut FfiapiMessage,
    warnings_len: size_t,
) -> () {
    unsafe {
        let cb_data: Box<TransformInvocationData> = Box::from_raw(raw_cb_data as *mut _);

        // Drop refcount.
        let _: Arc<TransformOptions> = Arc::from_raw(cb_data.opt_ptr);

        let mut code = Vec::from_raw_parts(cb_data.src_code_ptr, cb_data.src_code_len, cb_data.src_code_cap);
        code.truncate(out_len);

        let rust_cb_trait_box: Box<Box<dyn FnOnce(Vec<u8>, CVec<FfiapiMessage>, CVec<FfiapiMessage>)>>
            = Box::from_raw(cb_data.cb_trait_ptr as *mut _);

        let errors = CVec {
            ptr: raw_errors,
            len: errors_len,
        };
        let warnings = CVec {
            ptr: raw_warnings,
            len: warnings_len,
        };

        rust_cb_trait_box(code, errors, warnings);
    };
}

pub fn transform<F>(mut code: Vec<u8>, options: Arc<TransformOptions>, cb: F) -> ()
    where F: FnOnce(Vec<u8>, CVec<FfiapiMessage>, CVec<FfiapiMessage>),
          F: Send + 'static,
{
    // Prepare code.
    let src_code_ptr = code.as_mut_ptr();
    let src_code_len = code.len();
    let src_code_cap = code.capacity();
    let go_code = GoString {
        p: src_code_ptr as *const c_char,
        n: src_code_len as ptrdiff_t,
    };
    mem::forget(code);

    // Prepare options.
    // By converting to pointer, we consume it and therefore increase the refcount.
    let opt = options.clone();
    // We can safely convert anything in TransformOptions into raw pointers, as the memory is managed the the Arc and we only used owned values.
    let opt_engines_ptr = opt.engines.vec.as_ptr();
    let opt_engines_len = opt.engines.vec.len();
    let opt_defines_ptr = opt.defines.vec.as_ptr();
    let opt_defines_len = opt.defines.vec.len();

    // Prepare callback.
    let cb_box = Box::new(cb) as Box<dyn FnOnce(Vec<u8>, CVec<FfiapiMessage>, CVec<FfiapiMessage>)>;
    let cb_trait_box = Box::new(cb_box);
    let cb_trait_ptr = Box::into_raw(cb_trait_box);

    let data = Box::into_raw(Box::new(TransformInvocationData {
        src_code_ptr,
        src_code_len,
        src_code_cap,

        opt_ptr: Arc::into_raw(options.clone()),

        cb_trait_ptr: cb_trait_ptr as *mut c_void,
    }));

    unsafe {
        #[cfg(target_env="msvc")]
        #[allow(non_snake_case)]
        let GoTransform = mem::transmute::<_, GoTransform>(crate::bridge::DLL.get_function("GoTransform"));

        GoTransform(
            libc::malloc,
            transform_callback,
            data as *mut c_void,
            // To save memory allocations and avoid having to copy again to Vec after slice::from_raw_parts,
            // we direct Go to write the output directly into the source Vec.
            src_code_ptr as *mut c_void,
            go_code,
            opt.source_map as u8,
            opt.target as u8,
            opt_engines_ptr,
            opt_engines_len,
            opt.strict.nullish_coalescing,
            opt.strict.class_fields,
            opt.minify_whitespace,
            opt.minify_identifiers,
            opt.minify_syntax,
            GoString::from_str_unmanaged(&opt.jsx_factory),
            GoString::from_str_unmanaged(&opt.jsx_fragment),
            opt_defines_ptr,
            opt_defines_len,
            GoSlice::from_vec_unamanged(&opt.pure_functions),
            GoString::from_str_unmanaged(&opt.source_file),
            opt.loader as u8,
        );
    }
}
