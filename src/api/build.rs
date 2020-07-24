use std::future::Future;
use std::os::raw::c_void;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use libc::size_t;

use crate::bridge::GoBuild;
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

/// This function runs an end-to-end build operation. It takes an array of file paths as entry
/// points, parses them and all of their dependencies, and returns the output files to write to the
/// file system. The available options roughly correspond to esbuild's command-line flags.
///
/// The equivalent Go function will be called via Cgo, which will run the API from a Goroutine. This
/// means that this function will return immediately, and `cb` will be called sometime in the future
/// once the Goroutine completes. Additional concurrency management may be necessary to keep the
/// Rust program alive until all calls to this function actually complete.
///
/// # Arguments
///
/// * `options` - Built BuildOptions created from a BuildOptionsBuilder. A reference will be held on
///   the Arc until the callback is asynchronously called from Go.
/// * `cb` - Closure to call once the Goroutine completes with the BuildResult.
///
/// # Examples
///
/// This example uses the [crossbeam](https://docs.rs/crossbeam/) crate to prevent Rust from exiting
/// until the build completes.
///
/// ```
/// use crossbeam::sync::WaitGroup;
/// use esbuild_rs::{BuildOptionsBuilder, build_direct, BuildResult};
///
/// fn main() {
///   let mut options_builder = BuildOptionsBuilder::new();
///   options_builder.entry_points.push("index.js".to_string());
///   let options = options_builder.build();
///
///   let wg = WaitGroup::new();
///   let task = wg.clone();
///   build_direct(options, |BuildResult { output_files, errors, warnings }| {
///     println!("Build complete!");
///     drop(task);
///   });
///   wg.wait();
/// }
/// ```
pub fn build_direct<F>(options: Arc<BuildOptions>, cb: F) -> ()
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
            options.ffiapi_ptr,
        );
    }
}

struct BuildFutureState {
    result: Option<BuildResult>,
    waker: Option<Waker>,
}

pub struct BuildFuture {
    state: Arc<Mutex<BuildFutureState>>,
}

/// Future wrapper for `build_direct`.
///
/// # Arguments
///
/// * `options` - Built BuildOptions created from a BuildOptionsBuilder. A reference will be
///   held on the Arc until the Future completes.
///
/// # Examples
///
/// This example uses the [async-std](https://crates.io/crates/async-std) async runtime.
///
/// ```
/// use std::sync::Arc;
/// use async_std::task;
/// use esbuild_rs::{BuildOptionsBuilder, build, BuildResult};
///
/// fn main() {
///   let mut options_builder = BuildOptionsBuilder::new();
///   options_builder.entry_points.push("index.js".to_string());
///   let options = options_builder.build();
///
///   let res = task::block_on(build(options));
/// }
/// ```
pub fn build(options: Arc<BuildOptions>) -> BuildFuture {
    let state = Arc::new(Mutex::new(BuildFutureState {
        result: None,
        waker: None,
    }));
    let state_cb_copy = state.clone();
    build_direct(options, move |result| {
        let mut state = state_cb_copy.lock().unwrap();
        state.result = Some(result);
        if let Some(waker) = state.waker.take() {
            waker.wake();
        };
    });
    BuildFuture {
        state,
    }
}

impl Future for BuildFuture {
    type Output = BuildResult;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        match state.result.take() {
            Some(result) => Poll::Ready(result),
            None => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
