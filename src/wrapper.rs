use std::{fmt, ops, slice, str};
use std::fmt::{Display, Formatter};
use std::os::raw::{c_char, c_void};

use libc::{ptrdiff_t, size_t};

use crate::bridge::{FfiapiDefine, FfiapiEngine, FfiapiLoader, GoString};

// We wrap C arrays allocated from Go and sent to us in SliceContainer, such as `*ffiapi_message`.
// This will own the memory, make it usable as a slice, and drop using the matching deallocator.
pub struct SliceContainer<T> {
    pub(crate) ptr: *mut T,
    pub(crate) len: usize,
}

impl<T> ops::Deref for SliceContainer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.ptr, self.len)
        }
    }
}

impl<T> Drop for SliceContainer<T> {
    fn drop(&mut self) {
        unsafe {
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

impl ops::Deref for StrContainer {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe {
            str::from_utf8_unchecked(slice::from_raw_parts(self.data as *mut u8, self.len))
        }
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
        write!(f, "{} [{}:{}:{}]", &*self.text, &*self.file, self.line, self.column)
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

pub struct Defines {
    pub(crate) vec: Vec<FfiapiDefine>,
}

impl Defines {
    pub fn new() -> Defines {
        Defines {
            vec: Vec::new(),
        }
    }

    pub fn add(&mut self, name: String, value: String) -> () {
        self.vec.push(FfiapiDefine {
            from: GoString::from_string(name),
            to: GoString::from_string(value),
        });
    }
}

pub struct Engines {
    pub(crate) vec: Vec<FfiapiEngine>,
}

impl Engines {
    pub fn new() -> Engines {
        Engines {
            vec: Vec::new(),
        }
    }

    pub fn add(&mut self, name: EngineName, version: String) -> () {
        self.vec.push(FfiapiEngine {
            name: name as u8,
            version: GoString::from_string(version),
        });
    }
}

pub struct EntryPoints {
    pub(crate) vec: Vec<GoString>,
}

impl EntryPoints {
    pub fn new() -> EntryPoints {
        EntryPoints {
            vec: Vec::new(),
        }
    }

    pub fn add(&mut self, name: String) -> () {
        self.vec.push(GoString::from_string(name));
    }
}

pub struct Externals {
    pub(crate) vec: Vec<GoString>,
}

impl Externals {
    pub fn new() -> Externals {
        Externals {
            vec: Vec::new(),
        }
    }

    pub fn add(&mut self, name: String) -> () {
        self.vec.push(GoString::from_string(name));
    }
}

pub struct Loaders {
    pub(crate) vec: Vec<FfiapiLoader>,
}

impl Loaders {
    pub fn new() -> Loaders {
        Loaders {
            vec: Vec::new(),
        }
    }

    pub fn add(&mut self, name: String, loader: Loader) -> () {
        self.vec.push(FfiapiLoader {
            name: GoString::from_string(name),
            loader: loader as u8,
        });
    }
}

pub struct PureFunctions {
    pub(crate) vec: Vec<GoString>,
}

impl PureFunctions {
    pub fn new() -> PureFunctions {
        PureFunctions {
            vec: Vec::new(),
        }
    }

    pub fn add(&mut self, name: String) -> () {
        self.vec.push(GoString::from_string(name));
    }
}

pub struct ResolveExtensions {
    pub(crate) vec: Vec<GoString>,
}

impl ResolveExtensions {
    pub fn new() -> ResolveExtensions {
        ResolveExtensions {
            vec: Vec::new(),
        }
    }

    pub fn add(&mut self, ext: String) -> () {
        self.vec.push(GoString::from_string(ext));
    }
}

pub struct StrictOptions {
    pub nullish_coalescing: bool,
    pub class_fields: bool,
}

pub struct BuildOptions {
    pub source_map: SourceMap,
    pub target: Target,
    pub engines: Engines,
    pub strict: StrictOptions,

    pub minify_whitespace: bool,
    pub minify_identifiers: bool,
    pub minify_syntax: bool,

    pub jsx_factory: String,
    pub jsx_fragment: String,

    pub defines: Defines,
    pub pure_functions: PureFunctions,

    pub global_name: String,
    pub bundle: bool,
    pub splitting: bool,
    pub outfile: String,
    pub metafile: String,
    pub outdir: String,
    pub platform: Platform,
    pub format: Format,
    pub externals: Externals,
    pub loaders: Loaders,
    pub resolve_extensions: ResolveExtensions,
    pub tsconfig: String,

    pub entry_points: EntryPoints,
}

pub struct BuildResult {
    pub output_files: SliceContainer<OutputFile>,
    pub errors: SliceContainer<Message>,
    pub warnings: SliceContainer<Message>,
}

pub struct TransformOptions {
    pub source_map: SourceMap,
    pub target: Target,
    pub engines: Engines,
    pub strict: StrictOptions,

    pub minify_whitespace: bool,
    pub minify_identifiers: bool,
    pub minify_syntax: bool,

    pub jsx_factory: String,
    pub jsx_fragment: String,

    pub defines: Defines,
    pub pure_functions: PureFunctions,

    pub source_file: String,
    pub loader: Loader,
}

pub struct TransformResult {
    pub js: Vec<u8>,
    pub errors: SliceContainer<Message>,
    pub warnings: SliceContainer<Message>,
}
