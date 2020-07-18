use std::{ops, slice};
use std::os::raw::c_void;
use crate::bridge::{FfiapiEngine, GoString, FfiapiDefine, FfiapiMessage};

// We wrap C arrays allocated from Go and sent to us in CVec, such as `*ffiapi_message`.
// This will own the memory, make it usable as a slice, and drop using the matching deallocator.
pub struct CVec<T> {
    pub(crate) ptr: *mut T,
    pub(crate) len: usize,
}

impl<T> ops::Deref for CVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.ptr, self.len)
        }
    }
}

impl<T> Drop for CVec<T> {
    fn drop(&mut self) {
        unsafe {
            // We pass `malloc` to Go as the allocator.
            libc::free(self.ptr as *mut c_void);
        };
    }
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
    pub errors: CVec<FfiapiMessage>,
    pub warnings: CVec<FfiapiMessage>,
}
