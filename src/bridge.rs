use std::mem;
use std::os::raw::{c_char, c_void};

use libc::{ptrdiff_t, size_t};

use crate::wrapper::{Engine, Loader, Message, OutputFile, StrContainer};

const DUMMY_SAFE_PTR: &[u8] = &[0u8; 1024];

// Rust provides Unique::empty() as the pointer for empty allocations, which equals 0x1.
// This causes bad pointer panics in Go and even the occasional BSOD on Windows.
// See more at https://github.com/rust-lang/rust/issues/39625.
// This applies to GoString and FfiapiGoStringGoSlice.
pub fn get_allocation_pointer<T>(data: &[T]) -> *const T {
    if data.is_empty() {
        // We can't provide NULL as sometimes Go will panic on finding one, even with length 0.
        // DUMMY_SAFE_PTR points to a static array of 1024 bytes, zeroed out and safe to read.
        DUMMY_SAFE_PTR.as_ptr() as *const T
    } else {
        data.as_ptr()
    }
}

#[repr(C)]
pub struct GoString {
    pub p: *const c_char,
    pub n: ptrdiff_t,
}

impl GoString {
    pub fn from_string(mut str: String) -> GoString {
        str.shrink_to_fit();
        let ptr = get_allocation_pointer(str.as_bytes());
        let len = str.len();
        mem::forget(str);
        GoString {
            p: ptr as *const c_char,
            n: len as ptrdiff_t,
        }
    }

    // WARNING: The string must live for the lifetime of GoString.
    pub unsafe fn from_bytes_unmanaged(str: &[u8]) -> GoString {
        let ptr = get_allocation_pointer(str);
        let len = str.len();
        GoString {
            p: ptr as *const c_char,
            n: len as ptrdiff_t,
        }
    }
}

#[repr(C)]
pub struct FfiapiGoStringGoSlice {
    data: *mut c_void,
    len: ptrdiff_t,
    cap: ptrdiff_t,
}

impl FfiapiGoStringGoSlice {
    // WARNING: The string must live for the lifetime of GoSlice.
    pub unsafe fn from_vec_unamanged<T>(vec: &Vec<T>) -> FfiapiGoStringGoSlice {
        let ptr = get_allocation_pointer(vec);
        let len = vec.len();
        let cap = vec.capacity();
        FfiapiGoStringGoSlice {
            data: ptr as *mut c_void,
            len: len as ptrdiff_t,
            cap: cap as ptrdiff_t,
        }
    }
}

#[repr(C)]
pub struct FfiapiMapStringStringEntry {
    pub name: GoString,
    pub value: GoString,
}

impl FfiapiMapStringStringEntry {
    pub fn from_map_entry((name, value): (String, String)) -> FfiapiMapStringStringEntry {
        FfiapiMapStringStringEntry {
            name: GoString::from_string(name),
            value: GoString::from_string(value),
        }
    }
}

#[repr(C)]
pub struct FfiapiEngine {
    pub name: u8,
    pub version: GoString,
}

impl FfiapiEngine {
    pub fn from_engine(engine: Engine) -> FfiapiEngine {
        FfiapiEngine {
            name: engine.name as u8,
            version: GoString::from_string(engine.version),
        }
    }
}

#[repr(C)]
pub struct FfiapiLoader {
    pub name: GoString,
    pub loader: u8,
}

impl FfiapiLoader {
    pub fn from_map_entry((name, loader): (String, Loader)) -> FfiapiLoader {
        FfiapiLoader {
            name: GoString::from_string(name),
            loader: loader as u8,
        }
    }
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
    code: StrContainer,
    map: StrContainer,
    errors: *mut Message,
    errors_len: size_t,
    warnings: *mut Message,
    warnings_len: size_t,
) -> ();

#[repr(C)]
pub struct FfiapiBuildOptions {
    pub source_map: u8,
    pub sources_content: u8,

    pub target: u8,
    pub engines: *const FfiapiEngine,
    pub engines_len: size_t,

    pub minify_whitespace: bool,
    pub minify_identifiers: bool,
    pub minify_syntax: bool,
    pub charset: u8,
    pub tree_shaking: u8,

    pub jsx_factory: GoString,
    pub jsx_fragment: GoString,

    pub define: *const FfiapiMapStringStringEntry,
    pub define_len: size_t,
    // Slice of GoStrings.
    pub pure: FfiapiGoStringGoSlice,
    pub avoid_tdz: bool,
    pub keep_names: bool,

    pub global_name: GoString,
    pub bundle: bool,
    pub splitting: bool,
    pub outfile: GoString,
    pub metafile: GoString,
    pub outdir: GoString,
    pub outbase: GoString,
    pub platform: u8,
    pub format: u8,
    // Slice of GoStrings.
    pub external: FfiapiGoStringGoSlice,
    // Slice of GoStrings.
    pub main_fields: FfiapiGoStringGoSlice,
    pub loader: *const FfiapiLoader,
    pub loader_len: size_t,
    // Slice of GoStrings.
    pub resolve_extensions: FfiapiGoStringGoSlice,
    pub tsconfig: GoString,
    pub out_extensions: *const FfiapiMapStringStringEntry,
    pub out_extensions_len: size_t,
    pub public_path: GoString,
    // Slice of GoStrings.
    pub inject: FfiapiGoStringGoSlice,
    pub banner: GoString,
    pub footer: GoString,

    // Slice of GoStrings.
    pub entry_points: FfiapiGoStringGoSlice,
    pub write: bool,
    pub incremental: bool,
}

#[repr(C)]
pub struct FfiapiTransformOptions {
    pub source_map: u8,
    pub sources_content: u8,

    pub target: u8,
    pub format: u8,
    pub global_name: GoString,
    pub engines: *const FfiapiEngine,
    pub engines_len: size_t,

    pub minify_whitespace: bool,
    pub minify_identifiers: bool,
    pub minify_syntax: bool,
    pub charset: u8,
    pub tree_shaking: u8,

    pub jsx_factory: GoString,
    pub jsx_fragment: GoString,
    pub tsconfig_raw: GoString,
    pub footer: GoString,
    pub banner: GoString,

    pub define: *const FfiapiMapStringStringEntry,
    pub define_len: size_t,
    // Slice of GoStrings.
    pub pure: FfiapiGoStringGoSlice,
    pub avoid_tdz: bool,
    pub keep_names: bool,

    pub source_file: GoString,
    pub loader: u8,
}

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
    opt: *const FfiapiBuildOptions,
));

declare_ffi_fn!(GoTransform (
    alloc: Allocator,
    cb: TransformApiCallback,
    cb_data: *mut c_void,
    code: GoString,
    opt: *const FfiapiTransformOptions,
));
