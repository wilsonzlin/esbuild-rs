use std::os::raw::{c_char, c_void};
use std::sync::Arc;

use libc::{ptrdiff_t, size_t};

use crate::bridge::{GoSlice, GoString, GoTransform};
use crate::wrapper::{Message, SliceContainer, StrContainer, TransformOptions, TransformResult};

struct TransformInvocationData {
    src_vec_arc_raw: *const Vec<u8>,
    opt_arc_raw: *const TransformOptions,
    cb_trait_ptr: *mut c_void,
}

extern "C" fn transform_callback(
    raw_cb_data: *mut c_void,
    js: StrContainer,
    js_source_map: StrContainer,
    raw_errors: *mut Message,
    errors_len: size_t,
    raw_warnings: *mut Message,
    warnings_len: size_t,
) -> () {
    unsafe {
        let cb_data: Box<TransformInvocationData> = Box::from_raw(raw_cb_data as *mut _);

        // Drop source code refcount.
        let _: Arc<Vec<u8>> = Arc::from_raw(cb_data.src_vec_arc_raw);

        // Drop options refcount.
        let _: Arc<TransformOptions> = Arc::from_raw(cb_data.opt_arc_raw);

        let rust_cb_trait_box: Box<Box<dyn FnOnce(TransformResult)>>
            = Box::from_raw(cb_data.cb_trait_ptr as *mut _);

        let errors = SliceContainer {
            ptr: raw_errors,
            len: errors_len,
        };
        let warnings = SliceContainer {
            ptr: raw_warnings,
            len: warnings_len,
        };

        rust_cb_trait_box(TransformResult {
            js,
            js_source_map,
            errors,
            warnings,
        });
    };
}

pub fn transform<F>(code: Arc<Vec<u8>>, options: Arc<TransformOptions>, cb: F) -> ()
    where F: FnOnce(TransformResult),
          F: Send + 'static,
{
    // Prepare code.
    let go_code = GoString {
        p: code.as_ptr() as *const c_char,
        n: code.len() as ptrdiff_t,
    };

    // Prepare callback.
    let cb_box = Box::new(cb) as Box<dyn FnOnce(TransformResult)>;
    let cb_trait_box = Box::new(cb_box);
    let cb_trait_ptr = Box::into_raw(cb_trait_box);

    let data = Box::into_raw(Box::new(TransformInvocationData {
        src_vec_arc_raw: Arc::into_raw(code.clone()),
        opt_arc_raw: Arc::into_raw(options.clone()),
        cb_trait_ptr: cb_trait_ptr as *mut c_void,
    }));

    unsafe {
        #[cfg(target_env = "msvc")]
        #[allow(non_snake_case)]
        let GoTransform = std::mem::transmute::<_, GoTransform>(crate::bridge::DLL.get_function("GoTransform"));

        // We can safely convert anything in TransformOptions into raw pointers, as the memory is managed the the Arc and we only used owned values.
        GoTransform(
            libc::malloc,
            transform_callback,
            data as *mut c_void,
            go_code,
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
            GoString::from_str_unmanaged(&options.source_file),
            options.loader as u8,
        );
    }
}
