use std::mem;
use std::os::raw::{c_char, c_void};

use libc::{ptrdiff_t, size_t};

use crate::wrapper::{Message, OutputFile, StrContainer};

type GoInt = isize;

#[repr(C)]
pub struct GoString {
    pub p: *const c_char,
    pub n: ptrdiff_t,
}

impl GoString {
    pub fn from_string(mut str: String) -> GoString {
        str.shrink_to_fit();
        let ptr = str.as_ptr();
        let len = str.len();
        mem::forget(str);
        GoString {
            p: ptr as *const c_char,
            n: len as ptrdiff_t,
        }
    }

    // WARNING: The string must live for the lifetime of GoString.
    pub unsafe fn from_str_unmanaged(str: &str) -> GoString {
        // Rust provides Unique::empty() as the pointer for empty allocations, which equals 0x1.
        // This causes bad pointer panics in Go and even the occasional BSOD on Windows.
        // See more at https://github.com/rust-lang/rust/issues/39625.
        let ptr = if str.is_empty() {
            std::ptr::null()
        } else {
            str.as_ptr()
        };
        let len = str.len();
        GoString {
            p: ptr as *const c_char,
            n: len as ptrdiff_t,
        }
    }
}

#[repr(C)]
pub struct GoSlice {
    data: *mut c_void,
    len: GoInt,
    cap: GoInt,
}

impl GoSlice {
    // WARNING: The string must live for the lifetime of GoSlice.
    pub unsafe fn from_vec_unamanged<T>(vec: &Vec<T>) -> GoSlice {
        // Rust provides Unique::empty() as the pointer for empty allocations, which equals 0x1.
        // This causes bad pointer panics in Go and even the occasional BSOD on Windows.
        // See more at https://github.com/rust-lang/rust/issues/39625.
        let ptr = if vec.is_empty() {
            std::ptr::null()
        } else {
            vec.as_ptr()
        };
        let len = vec.len();
        let cap = vec.capacity();
        GoSlice {
            data: ptr as *mut c_void,
            len: len as GoInt,
            cap: cap as GoInt,
        }
    }
}

#[repr(C)]
pub struct FfiapiDefine {
    pub from: GoString,
    pub to: GoString,
}

#[repr(C)]
pub struct FfiapiEngine {
    pub name: u8,
    pub version: GoString,
}

#[repr(C)]
pub struct FfiapiLoader {
    pub name: GoString,
    pub loader: u8,
}

pub type Allocator = unsafe extern "C" fn(n: size_t) -> *mut c_void;

pub type BuildApiCallback = extern "C" fn(
    cb_data: *mut c_void,
    output_files: *mut OutputFile,
    output_files_len: size_t,
    errors: *mut Message,
    errors_len: size_t,
    warnings: *mut Message,
    warnings_len: size_t,
) -> ();

pub type TransformApiCallback = extern "C" fn(
    cb_data: *mut c_void,
    js: StrContainer,
    js_source_map: StrContainer,
    errors: *mut Message,
    errors_len: size_t,
    warnings: *mut Message,
    warnings_len: size_t,
) -> ();

#[cfg(target_env = "msvc")]
const DLL_BIN: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/esbuild.dll"));

#[cfg(target_env = "msvc")]
lazy_static::lazy_static! {
    pub static ref DLL: memorymodule_rs::MemoryModule<'static> = memorymodule_rs::MemoryModule::new(DLL_BIN);
}

#[cfg(not(target_env = "msvc"))]
macro_rules! declare_ffi_fn {
    ($name:ident (
        $(
            $argn:ident: $argt:ty,
        )*
    )) => (
        extern "C" {
            pub fn $name (
                $($argn: $argt,)*
            );
        }
    )
}

#[cfg(target_env = "msvc")]
macro_rules! declare_ffi_fn {
    ($name:ident (
        $(
            $argn:ident: $argt:ty,
        )*
    )) => (
        pub type $name = extern "C" fn (
            $($argn: $argt,)*
        );
    )
}

declare_ffi_fn!(GoBuild (
    alloc: Allocator,
    cb: BuildApiCallback,
    cb_data: *mut c_void,

    source_map: u8,
    target: u8,
    engines: *const FfiapiEngine,
    engines_len: size_t,
    strict_nullish_coalescing: bool,
    strict_class_fields: bool,

    minify_whitespace: bool,
    minify_identifiers: bool,
    minify_syntax: bool,

    jsx_factory: GoString,
    jsx_fragment: GoString,

    defines: *const FfiapiDefine,
    defines_len: size_t,
    // Slice of GoStrings.
    pure_functions: GoSlice,

    global_name: GoString,
    bundle: bool,
    splitting: bool,
    outfile: GoString,
    metafile: GoString,
    outdir: GoString,
    platform: u8,
    format: u8,
    // Slice of GoStrings.
    externals: GoSlice,
    loaders: *const FfiapiLoader,
    loaders_len: size_t,
    // Slice of GoStrings.
    resolve_extensions: GoSlice,
    tsconfig: GoString,

    // Slice of GoStrings.
    entry_points: GoSlice,
));

declare_ffi_fn!(GoTransform (
    alloc: Allocator,
    cb: TransformApiCallback,
    cb_data: *mut c_void,
    code: GoString,

    source_map: u8,
    target: u8,
    engines: *const FfiapiEngine,
    engines_len: size_t,
    strict_nullish_coalescing: bool,
    strict_class_fields: bool,

    minify_whitespace: bool,
    minify_identifiers: bool,
    minify_syntax: bool,

    jsx_factory: GoString,
    jsx_fragment: GoString,

    defines: *const FfiapiDefine,
    defines_len: size_t,
    // Slice of GoStrings.
    pure_functions: GoSlice,

    source_file: GoString,
    loader: u8,
));
