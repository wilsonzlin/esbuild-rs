use std::{ops, slice, str};
use std::os::raw::{c_void, c_char};
use libc::{ptrdiff_t, size_t};
use crate::bridge::{FfiapiEngine, GoString, FfiapiDefine};

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

pub struct StrictOptions {
    pub nullish_coalescing: bool,
    pub class_fields: bool,
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
    pub pure_functions: Vec<String>,

    pub source_file: String,
    pub loader: Loader,
}

pub struct TransformResult {
    pub js: Vec<u8>,
    pub errors: SliceContainer<Message>,
    pub warnings: SliceContainer<Message>,
}
