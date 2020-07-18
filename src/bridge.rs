use std::os::raw::{c_char, c_void, c_int};
use std::mem;
use lazy_static::lazy_static;
use libc::{ptrdiff_t, size_t};

type GoInt = i64;

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
        let ptr = str.as_ptr();
        let len = str.len();
        GoString {
            p: ptr as *const c_char,
            n: len as ptrdiff_t,
        }
    }
}

#[repr(C)]
pub struct GoSlice {
    data: *const c_void,
    len: GoInt,
    cap: GoInt,
}

impl GoSlice {
    // WARNING The string must live for the lifetime of GoSlice.
    pub unsafe fn from_vec_unamanged<T>(vec: &Vec<T>) -> GoSlice {
        let ptr = vec.as_ptr();
        let len = vec.len();
        let cap = vec.capacity();
        GoSlice {
            data: ptr as *const c_void,
            len: len as GoInt,
            cap: cap as GoInt,
        }
    }
}

#[repr(C)]
pub struct FfiapiString {
    len: size_t,
    data: *mut c_char,
}

#[repr(C)]
pub struct FfiapiMessage {
    file: FfiapiString,
    line: c_int,
    column: c_int,
    length: c_int,
    text: FfiapiString,
}

#[repr(C)]
pub struct FfiapiEngine {
    pub name: u8,
    pub version: GoString,
}

#[repr(C)]
pub struct FfiapiDefine {
    pub from: GoString,
    pub to: GoString,
}

pub type Allocator = unsafe extern "C" fn(n: size_t) -> *mut c_void;

pub type TransformApiCallback = extern "C" fn(
    cb_data: *mut c_void,
    out_len: size_t,
    errors: *mut FfiapiMessage,
    errors_len: size_t,
    warnings: *mut FfiapiMessage,
    warnings_len: size_t,
) -> ();

#[cfg(not(feature = "use-dll"))]
extern "C" {
    pub fn GoTransform(
        alloc: Allocator,
        cb: TransformApiCallback,
        cb_data: *mut c_void,
        out: *mut c_void,
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
    ) -> ();
}

#[cfg(feature = "use-dll")]
const DLL_BIN: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/esbuild.dll"));

#[cfg(feature = "use-dll")]
lazy_static! {
    pub static ref DLL: memorymodule_rs::MemoryModule<'static> = memorymodule_rs::MemoryModule::new(DLL_BIN);
}

#[cfg(feature = "use-dll")]
// TODO Combine with extern "C" declaration for not(use-dll).
pub type GoTransform = extern "C" fn(
    alloc: Allocator,
    cb: TransformApiCallback,
    cb_data: *mut c_void,
    out: *mut c_void,
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
);
