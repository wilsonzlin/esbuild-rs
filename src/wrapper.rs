use std::{convert, fmt, slice, str};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::os::raw::{c_char, c_void};
use std::sync::Arc;

use libc::{ptrdiff_t, size_t};

use crate::bridge::{FfiapiBuildOptions, FfiapiDefine, FfiapiEngine, FfiapiGoStringGoSlice, FfiapiLoader, FfiapiTransformOptions, get_allocation_pointer, GoString};

#[inline(always)]
fn transform<I, S: IntoIterator<Item=I>, O, T: Fn(I) -> O>(src: S, mapper: T) -> Vec<O> {
    src.into_iter().map(mapper).collect::<Vec<O>>()
}

// We wrap C arrays allocated from Go and sent to us in SliceContainer, such as `*ffiapi_message`.
// This will own the memory, make it usable as a slice, and drop using the matching deallocator.
pub struct SliceContainer<T> {
    pub(crate) ptr: *mut T,
    pub(crate) len: usize,
}

impl <T> SliceContainer<T> {
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.ptr, self.len)
        }
    }
}

unsafe impl<T> Send for SliceContainer<T> {}

unsafe impl<T> Sync for SliceContainer<T> {}

impl<T> convert::AsRef<[T]> for SliceContainer<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> Drop for SliceContainer<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.len {
                drop(self.ptr.offset(i as isize));
            };
            // We pass `malloc` to Go as the allocator.
            libc::free(self.ptr as *mut c_void);
        };
    }
}

// This is the ffiapi_string struct in C; we declare it here to avoid having to needlessly rewrap in StrContainer.
// This will own the memory, make it usable as a str, and drop using the matching deallocator.
#[repr(C)]
pub struct StrContainer {
    len: size_t,
    data: *mut c_char,
}

impl StrContainer {
    pub fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(self.data as *mut u8, self.len))
        }
    }
}

unsafe impl Send for StrContainer {}

unsafe impl Sync for StrContainer {}

impl convert::AsRef<str> for StrContainer {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Drop for StrContainer {
    fn drop(&mut self) {
        unsafe {
            // We pass `malloc` to Go as the allocator.
            libc::free(self.data as *mut c_void);
        };
    }
}

// This is the ffiapi_message struct in C; we declare it here to avoid having to needlessly rewrap in Message.
#[repr(C)]
pub struct Message {
    pub file: StrContainer,
    pub line: ptrdiff_t,
    pub column: ptrdiff_t,
    pub length: ptrdiff_t,
    pub text: StrContainer,
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}:{}:{}]", self.text.as_str(), self.file.as_str(), self.line, self.column)
    }
}

// This is the ffiapi_output_file struct in C; we declare it here to avoid having to needlessly rewrap in OutputFile.
#[repr(C)]
pub struct OutputFile {
    pub path: StrContainer,
    pub data: StrContainer,
}

#[derive(Copy, Clone)]
pub enum EngineName {
    Chrome,
    Edge,
    Firefox,
    IOS,
    Node,
    Safari,
}

#[derive(Copy, Clone)]
pub enum Format {
    Default,
    IIFE,
    CommonJS,
    ESModule,
}

#[derive(Copy, Clone)]
pub enum Loader {
    JS,
    JSX,
    TS,
    TSX,
    JSON,
    Text,
    Base64,
    DataURL,
    File,
    Binary,
}

#[derive(Copy, Clone)]
pub enum Platform {
    Browser,
    Node,
}

#[derive(Copy, Clone)]
pub enum SourceMap {
    None,
    Inline,
    Linked,
    External,
}

#[derive(Copy, Clone)]
pub enum Target {
    ESNext,
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
}

#[derive(Clone)]
pub struct Engine {
    pub name: EngineName,
    pub version: String,
}

#[derive(Clone)]
pub struct StrictOptions {
    pub nullish_coalescing: bool,
    pub class_fields: bool,
}

// BuildOptions and TransformOptions are nice APIs that mimics official Go API and use standard Rust
// types. They're similar to Ffiapi*Options, but we create a separate struct for ease of use, as
// Ffiapi*Options uses raw pointers which are difficult to mutate, either directly or in
// abstracted methods/helper functions.

#[derive(Clone)]
pub struct BuildOptionsBuilder {
    pub source_map: SourceMap,
    pub target: Target,
    pub engines: Vec<Engine>,
    pub strict: StrictOptions,

    pub minify_whitespace: bool,
    pub minify_identifiers: bool,
    pub minify_syntax: bool,

    pub jsx_factory: String,
    pub jsx_fragment: String,

    pub defines: HashMap<String, String>,
    pub pure_functions: Vec<String>,

    pub global_name: String,
    pub bundle: bool,
    pub splitting: bool,
    pub outfile: String,
    pub metafile: String,
    pub outdir: String,
    pub platform: Platform,
    pub format: Format,
    pub externals: Vec<String>,
    pub loaders: HashMap<String, Loader>,
    pub resolve_extensions: Vec<String>,
    pub tsconfig: String,

    pub entry_points: Vec<String>,
}

pub struct BuildOptions {
    // We keep data that fields of ffiapi_ptr point to.
    engines: Vec<FfiapiEngine>,
    jsx_factory: String,
    jsx_fragment: String,
    defines: Vec<FfiapiDefine>,
    pure_functions: Vec<GoString>,
    global_name: String,
    outfile: String,
    metafile: String,
    outdir: String,
    externals: Vec<GoString>,
    loaders: Vec<FfiapiLoader>,
    resolve_extensions: Vec<GoString>,
    tsconfig: String,
    entry_points: Vec<GoString>,
    pub(crate) ffiapi_ptr: *const FfiapiBuildOptions,
}

unsafe impl Send for BuildOptions {}

unsafe impl Sync for BuildOptions {}

impl Drop for BuildOptions {
    fn drop(&mut self) {
        unsafe {
            let _: Box<FfiapiBuildOptions> = Box::from_raw(self.ffiapi_ptr as _);
        };
    }
}

impl BuildOptionsBuilder {
    pub fn new() -> BuildOptionsBuilder {
        BuildOptionsBuilder {
            source_map: SourceMap::None,
            target: Target::ESNext,
            engines: Vec::new(),
            strict: StrictOptions {
                class_fields: false,
                nullish_coalescing: false,
            },
            minify_whitespace: false,
            minify_identifiers: false,
            minify_syntax: false,
            jsx_factory: String::new(),
            jsx_fragment: String::new(),
            defines: HashMap::new(),
            pure_functions: Vec::new(),
            global_name: String::new(),
            bundle: false,
            splitting: false,
            outfile: String::new(),
            metafile: String::new(),
            outdir: String::new(),
            platform: Platform::Browser,
            format: Format::Default,
            externals: Vec::new(),
            loaders: HashMap::new(),
            resolve_extensions: Vec::new(),
            tsconfig: String::new(),
            entry_points: Vec::new(),
        }
    }

    pub fn build(self) -> Arc<BuildOptions> {
        let mut res = Arc::new(BuildOptions {
            // We move into Arc first before creating pointers to data in it, as the move to the
            // heap by Arc should change the data's location.
            engines: transform(self.engines, FfiapiEngine::from_engine),
            jsx_factory: self.jsx_factory,
            jsx_fragment: self.jsx_fragment,
            defines: transform(self.defines, FfiapiDefine::from_map_entry),
            pure_functions: transform(self.pure_functions, GoString::from_string),
            global_name: self.global_name,
            outfile: self.outfile,
            metafile: self.metafile,
            outdir: self.outdir,
            externals: transform(self.externals, GoString::from_string),
            loaders: transform(self.loaders, FfiapiLoader::from_map_entry),
            resolve_extensions: transform(self.resolve_extensions, GoString::from_string),
            tsconfig: self.tsconfig,
            entry_points: transform(self.entry_points, GoString::from_string),
            ffiapi_ptr: std::ptr::null(),
        });

        unsafe {
            let ffiapi_ptr = Box::into_raw(Box::new(FfiapiBuildOptions {
                source_map: self.source_map as u8,
                target: self.target as u8,
                engines: get_allocation_pointer(&res.engines),
                engines_len: res.engines.len(),
                strict_nullish_coalescing: self.strict.nullish_coalescing,
                strict_class_fields: self.strict.class_fields,

                minify_whitespace: self.minify_whitespace,
                minify_identifiers: self.minify_identifiers,
                minify_syntax: self.minify_syntax,

                jsx_factory: GoString::from_bytes_unmanaged(res.jsx_factory.as_bytes()),
                jsx_fragment: GoString::from_bytes_unmanaged(res.jsx_fragment.as_bytes()),

                defines: get_allocation_pointer(&res.defines),
                defines_len: res.defines.len(),
                pure_functions: FfiapiGoStringGoSlice::from_vec_unamanged(&res.pure_functions),

                global_name: GoString::from_bytes_unmanaged(res.global_name.as_bytes()),
                bundle: self.bundle,
                splitting: self.splitting,
                outfile: GoString::from_bytes_unmanaged(res.outfile.as_bytes()),
                metafile: GoString::from_bytes_unmanaged(res.metafile.as_bytes()),
                outdir: GoString::from_bytes_unmanaged(res.outdir.as_bytes()),
                platform: self.platform as u8,
                format: self.format as u8,
                externals: FfiapiGoStringGoSlice::from_vec_unamanged(&res.externals),
                loaders: get_allocation_pointer(&res.loaders),
                loaders_len: res.loaders.len(),
                resolve_extensions: FfiapiGoStringGoSlice::from_vec_unamanged(&res.resolve_extensions),
                tsconfig: GoString::from_bytes_unmanaged(res.tsconfig.as_bytes()),

                entry_points: FfiapiGoStringGoSlice::from_vec_unamanged(&res.entry_points),
            }));
            Arc::get_mut(&mut res).unwrap().ffiapi_ptr = ffiapi_ptr;
        };

        res
    }
}

pub struct BuildResult {
    pub output_files: SliceContainer<OutputFile>,
    pub errors: SliceContainer<Message>,
    pub warnings: SliceContainer<Message>,
}

#[derive(Clone)]
pub struct TransformOptionsBuilder {
    pub source_map: SourceMap,
    pub target: Target,
    pub engines: Vec<Engine>,
    pub strict: StrictOptions,

    pub minify_whitespace: bool,
    pub minify_identifiers: bool,
    pub minify_syntax: bool,

    pub jsx_factory: String,
    pub jsx_fragment: String,

    pub defines: HashMap<String, String>,
    pub pure_functions: Vec<String>,

    pub source_file: String,
    pub loader: Loader,
}

pub struct TransformOptions {
    // We keep data that fields of ffiapi_ptr point to.
    engines: Vec<FfiapiEngine>,
    jsx_factory: String,
    jsx_fragment: String,
    defines: Vec<FfiapiDefine>,
    pure_functions: Vec<GoString>,
    source_file: String,
    pub(crate) ffiapi_ptr: *const FfiapiTransformOptions,
}

unsafe impl Send for TransformOptions {}

unsafe impl Sync for TransformOptions {}

impl Drop for TransformOptions {
    fn drop(&mut self) {
        unsafe {
            let _: Box<FfiapiTransformOptions> = Box::from_raw(self.ffiapi_ptr as _);
        };
    }
}

impl TransformOptionsBuilder {
    pub fn new() -> TransformOptionsBuilder {
        TransformOptionsBuilder {
            source_map: SourceMap::None,
            target: Target::ESNext,
            engines: Vec::new(),
            strict: StrictOptions {
                class_fields: false,
                nullish_coalescing: false,
            },
            minify_whitespace: false,
            minify_identifiers: false,
            minify_syntax: false,
            jsx_factory: String::new(),
            jsx_fragment: String::new(),
            defines: HashMap::new(),
            pure_functions: Vec::new(),
            source_file: String::new(),
            loader: Loader::JS,
        }
    }

    pub fn build(self) -> Arc<TransformOptions> {
        let mut res = Arc::new(TransformOptions {
            // We move into Arc first before creating pointers to data in it, as the move to the
            // heap by Arc should change the data's location.
            engines: transform(self.engines, FfiapiEngine::from_engine),
            jsx_factory: self.jsx_factory,
            jsx_fragment: self.jsx_fragment,
            defines: transform(self.defines, FfiapiDefine::from_map_entry),
            pure_functions: transform(self.pure_functions, GoString::from_string),
            source_file: self.source_file,
            ffiapi_ptr: std::ptr::null(),
        });

        unsafe {
            let ffiapi_ptr = Box::into_raw(Box::new(FfiapiTransformOptions {
                source_map: self.source_map as u8,
                target: self.target as u8,
                engines: get_allocation_pointer(&res.engines),
                engines_len: res.engines.len(),
                strict_nullish_coalescing: self.strict.nullish_coalescing,
                strict_class_fields: self.strict.class_fields,
                minify_whitespace: self.minify_whitespace,
                minify_identifiers: self.minify_identifiers,
                minify_syntax: self.minify_syntax,
                jsx_factory: GoString::from_bytes_unmanaged(res.jsx_factory.as_bytes()),
                jsx_fragment: GoString::from_bytes_unmanaged(res.jsx_fragment.as_bytes()),
                defines: get_allocation_pointer(&res.defines),
                defines_len: res.defines.len(),
                pure_functions: FfiapiGoStringGoSlice::from_vec_unamanged(&res.pure_functions),
                source_file: GoString::from_bytes_unmanaged(res.source_file.as_bytes()),
                loader: self.loader as u8,
            }));
            Arc::get_mut(&mut res).unwrap().ffiapi_ptr = ffiapi_ptr;
        };

        res
    }
}

pub struct TransformResult {
    pub js: StrContainer,
    pub js_source_map: StrContainer,
    pub errors: SliceContainer<Message>,
    pub warnings: SliceContainer<Message>,
}
