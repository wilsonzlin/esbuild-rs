use std::os::raw::c_void;
use std::sync::Arc;

use libc::size_t;

use crate::bridge::{GoString, GoTransform};
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

/// This function transforms a string of source code into JavaScript. It can be used to minify
/// JavaScript, convert TypeScript/JSX to JavaScript, or convert newer JavaScript to older
/// JavaScript. The available options roughly correspond to esbuild's command-line flags.
///
/// The equivalent Go function will be called via Cgo, which will run the API from a Goroutine. This
/// means that this function will return immediately, and `cb` will be called sometime in the future
/// once the Goroutine completes. Additional concurrency management may be necessary to keep the
/// Rust program alive until all calls to this function actually complete.
///
/// # Arguments
///
/// * `code` - Source code to transform. Must be UTF-8. A reference will be held on the Arc until
///   the callback is asynchronously called from Go.
/// * `options` - Built TransformOptions created from a TransformOptionsBuilder. A reference will be
///   held on the Arc until the callback is asynchronously called from Go.
/// * `cb` - Closure to call once the Goroutine completes with the TransformResult.
///
/// # Examples
///
/// This example uses the [crossbeam](https://docs.rs/crossbeam/) crate to prevent Rust from exiting
/// until the transform completes.
///
/// ```
/// use std::sync::Arc;
/// use crossbeam::sync::WaitGroup;
/// use esbuild_rs::{TransformOptionsBuilder, transform, TransformResult};
///
/// fn main() {
///   let wg = WaitGroup::new();
///   let src = Arc::new(b"let x = NAME;".to_vec());
///   let mut options_builder = TransformOptionsBuilder::new();
///   options_builder.defines.insert("NAME".to_string(), "world".to_string());
///   let options = options_builder.build();
///   let task = wg.clone();
///   transform(src, options, |TransformResult { js, js_source_map, errors, warnings }| {
///     println!("Transform complete!");
///     drop(task);
///   });
///   wg.wait();
/// }
/// ```
pub fn transform<F>(code: Arc<Vec<u8>>, options: Arc<TransformOptions>, cb: F) -> ()
    where F: FnOnce(TransformResult),
          F: Send + 'static,
{
    // Prepare code.
    let go_code = unsafe { GoString::from_bytes_unmanaged(&code) };

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
            options.ffiapi_ptr,
        );
    }
}
