use std::{convert, fmt, slice, str};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::os::raw::{c_char, c_void};
use std::sync::Arc;

use libc::{ptrdiff_t, size_t};

use crate::bridge::{FfiapiBuildOptions, FfiapiMapStringStringEntry, FfiapiEngine, FfiapiGoStringGoSlice, FfiapiLoader, FfiapiTransformOptions, get_allocation_pointer, GoString, FfiapiEntryPoint};

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

impl<T> SliceContainer<T> {
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
pub enum Charset {
    Default,
    ASCII,
    UTF8,
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
pub enum JSXMode {
    Transform,
    Preserve,
}

#[derive(Copy, Clone)]
pub enum LegalComments {
    Default,
    None,
    Inline,
    EndOfFile,
    Linked,
    External,
}

#[derive(Copy, Clone)]
pub enum Loader {
    None,
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
    CSS,
    Default,
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
pub enum SourcesContent {
    Include,
    Exclude,
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
pub enum TreeShaking {
    Default,
    IgnoreAnnotations,
}

#[derive(Clone)]
pub struct Engine {
    pub name: EngineName,
    pub version: String,
}

#[derive(Clone)]
pub struct EntryPoint {
    pub input_path: String,
    pub output_path: String,
}

// BuildOptions and TransformOptions are nice APIs that mimics official Go API and use standard Rust
// types. They're similar to Ffiapi*Options, but we create a separate struct for ease of use, as
// Ffiapi*Options uses raw pointers which are difficult to mutate, either directly or in
// abstracted methods/helper functions.

#[derive(Clone)]
pub struct BuildOptionsBuilder {
    pub source_map: SourceMap,
    pub source_root: String,
    pub sources_content: SourcesContent,

    pub target: Target,
    pub engines: Vec<Engine>,

    pub minify_whitespace: bool,
    pub minify_identifiers: bool,
    pub minify_syntax: bool,
    pub charset: Charset,
    pub tree_shaking: TreeShaking,
    pub legal_comments: LegalComments,

    pub jsx_mode: JSXMode,
    pub jsx_factory: String,
    pub jsx_fragment: String,

    pub define: HashMap<String, String>,
    pub pure: Vec<String>,
    pub keep_names: bool,

    pub global_name: String,
    pub bundle: bool,
    pub preserve_symlinks: bool,
    pub splitting: bool,
    pub outfile: String,
    pub metafile: bool,
    pub outdir: String,
    pub outbase: String,
    pub abs_working_dir: String,
    pub platform: Platform,
    pub format: Format,
    pub external: Vec<String>,
    pub main_fields: Vec<String>,
    pub conditions: Vec<String>,
    pub loader: HashMap<String, Loader>,
    pub resolve_extensions: Vec<String>,
    pub tsconfig: String,
    pub out_extensions: HashMap<String, String>,
    pub public_path: String,
    pub inject: Vec<String>,
    pub banner: HashMap<String, String>,
    pub footer: HashMap<String, String>,
    pub node_paths: Vec<String>,

    pub entry_names: String,
    pub chunk_names: String,
    pub asset_names: String,

    pub entry_points: Vec<String>,
    pub entry_points_advanced: Vec<EntryPoint>,

    pub write: bool,
    pub allow_overwrite: bool,
    pub incremental: bool,
}

pub struct BuildOptions {
    // We keep data that fields of ffiapi_ptr point to.
    source_root: String,
    engines: Vec<FfiapiEngine>,
    jsx_factory: String,
    jsx_fragment: String,
    define: Vec<FfiapiMapStringStringEntry>,
    pure: Vec<GoString>,
    global_name: String,
    outfile: String,
    outdir: String,
    outbase: String,
    abs_working_dir: String,
    external: Vec<GoString>,
    main_fields: Vec<GoString>,
    conditions: Vec<GoString>,
    loader: Vec<FfiapiLoader>,
    resolve_extensions: Vec<GoString>,
    tsconfig: String,
    out_extensions: Vec<FfiapiMapStringStringEntry>,
    public_path: String,
    inject: Vec<GoString>,
    banner: Vec<FfiapiMapStringStringEntry>,
    footer: Vec<FfiapiMapStringStringEntry>,
    node_paths: Vec<GoString>,
    entry_names: String,
    chunk_names: String,
    asset_names: String,
    entry_points: Vec<GoString>,
    entry_points_advanced: Vec<FfiapiEntryPoint>,
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
            source_root: "".to_string(),
            sources_content: SourcesContent::Include,
            target: Target::ESNext,
            engines: vec![],
            minify_whitespace: false,
            minify_identifiers: false,
            minify_syntax: false,
            charset: Charset::Default,
            tree_shaking: TreeShaking::Default,
            legal_comments: LegalComments::Default,
            jsx_mode: JSXMode::Transform,
            jsx_factory: "".to_string(),
            jsx_fragment: "".to_string(),
            define: Default::default(),
            pure: vec![],
            keep_names: false,
            global_name: "".to_string(),
            bundle: false,
            preserve_symlinks: false,
            splitting: false,
            outfile: "".to_string(),
            metafile: false,
            outdir: "".to_string(),
            outbase: "".to_string(),
            abs_working_dir: "".to_string(),
            platform: Platform::Browser,
            format: Format::Default,
            external: vec![],
            main_fields: vec![],
            conditions: vec![],
            loader: Default::default(),
            resolve_extensions: vec![],
            tsconfig: "".to_string(),
            out_extensions: Default::default(),
            public_path: "".to_string(),
            inject: vec![],
            banner: Default::default(),
            footer: Default::default(),
            node_paths: vec![],
            entry_names: "".to_string(),
            chunk_names: "".to_string(),
            asset_names: "".to_string(),
            entry_points: vec![],
            entry_points_advanced: vec![],
            write: false,
            allow_overwrite: false,
            incremental: false
        }
    }

    pub fn build(self) -> Arc<BuildOptions> {
        let mut res = Arc::new(BuildOptions {
            // We move into Arc first before creating pointers to data in it, as the move to the
            // heap by Arc should change the data's location.
            source_root: self.source_root,
            engines: transform(self.engines, FfiapiEngine::from_engine),
            jsx_factory: self.jsx_factory,
            jsx_fragment: self.jsx_fragment,
            define: transform(self.define, FfiapiMapStringStringEntry::from_map_entry),
            pure: transform(self.pure, GoString::from_string),
            global_name: self.global_name,
            outfile: self.outfile,
            outdir: self.outdir,
            outbase: self.outbase,
            abs_working_dir: self.abs_working_dir,
            external: transform(self.external, GoString::from_string),
            main_fields: transform(self.main_fields, GoString::from_string),
            conditions: transform(self.conditions, GoString::from_string),
            loader: transform(self.loader, FfiapiLoader::from_map_entry),
            resolve_extensions: transform(self.resolve_extensions, GoString::from_string),
            tsconfig: self.tsconfig,
            out_extensions: transform(self.out_extensions, FfiapiMapStringStringEntry::from_map_entry),
            public_path: self.public_path,
            inject: transform(self.inject, GoString::from_string),
            banner: transform(self.banner, FfiapiMapStringStringEntry::from_map_entry),
            footer: transform(self.footer, FfiapiMapStringStringEntry::from_map_entry),
            node_paths: transform(self.node_paths, GoString::from_string),
            entry_names: self.entry_names,
            chunk_names: self.chunk_names,
            asset_names: self.asset_names,
            entry_points: transform(self.entry_points, GoString::from_string),
            entry_points_advanced: transform(self.entry_points_advanced, FfiapiEntryPoint::from_entry_point),
            ffiapi_ptr: std::ptr::null(),
        });

        unsafe {
            let ffiapi_ptr = Box::into_raw(Box::new(FfiapiBuildOptions {
                source_map: self.source_map as u8,
                source_root: GoString::from_bytes_unmanaged(res.source_root.as_bytes()),
                sources_content: self.sources_content as u8,

                target: self.target as u8,
                engines: get_allocation_pointer(&res.engines),
                engines_len: res.engines.len(),

                minify_whitespace: self.minify_whitespace,
                minify_identifiers: self.minify_identifiers,
                minify_syntax: self.minify_syntax,
                charset: self.charset as u8,
                tree_shaking: self.tree_shaking as u8,
                legal_comments: self.legal_comments as u8,

                jsx_mode: self.jsx_mode as u8,
                jsx_factory: GoString::from_bytes_unmanaged(res.jsx_factory.as_bytes()),
                jsx_fragment: GoString::from_bytes_unmanaged(res.jsx_fragment.as_bytes()),

                define: get_allocation_pointer(&res.define),
                define_len: res.define.len(),
                pure: FfiapiGoStringGoSlice::from_vec_unamanged(&res.pure),
                keep_names: self.keep_names,

                global_name: GoString::from_bytes_unmanaged(res.global_name.as_bytes()),
                bundle: self.bundle,
                preserve_symlinks: self.preserve_symlinks,
                splitting: self.splitting,
                outfile: GoString::from_bytes_unmanaged(res.outfile.as_bytes()),
                metafile: self.metafile,
                outdir: GoString::from_bytes_unmanaged(res.outdir.as_bytes()),
                outbase: GoString::from_bytes_unmanaged(res.outbase.as_bytes()),
                abs_working_dir: GoString::from_bytes_unmanaged(res.abs_working_dir.as_bytes()),
                platform: self.platform as u8,
                format: self.format as u8,
                external: FfiapiGoStringGoSlice::from_vec_unamanged(&res.external),
                main_fields: FfiapiGoStringGoSlice::from_vec_unamanged(&res.main_fields),
                conditions: FfiapiGoStringGoSlice::from_vec_unamanged(&res.conditions),
                loader: get_allocation_pointer(&res.loader),
                loader_len: res.loader.len(),
                resolve_extensions: FfiapiGoStringGoSlice::from_vec_unamanged(&res.resolve_extensions),
                tsconfig: GoString::from_bytes_unmanaged(res.tsconfig.as_bytes()),
                out_extensions: get_allocation_pointer(&res.out_extensions),
                out_extensions_len: res.out_extensions.len(),
                public_path: GoString::from_bytes_unmanaged(res.public_path.as_bytes()),
                inject: FfiapiGoStringGoSlice::from_vec_unamanged(&res.inject),
                banner: get_allocation_pointer(&res.banner),
                banner_len: res.banner.len(),
                footer: get_allocation_pointer(&res.footer),
                footer_len: res.footer.len(),
                node_paths: FfiapiGoStringGoSlice::from_vec_unamanged(&res.node_paths),

                entry_names: GoString::from_bytes_unmanaged(res.entry_names.as_bytes()),
                chunk_names: GoString::from_bytes_unmanaged(res.chunk_names.as_bytes()),
                asset_names: GoString::from_bytes_unmanaged(res.asset_names.as_bytes()),

                entry_points: FfiapiGoStringGoSlice::from_vec_unamanged(&res.entry_points),
                entry_points_advanced: get_allocation_pointer(&res.entry_points_advanced),
                entry_points_advanced_len: res.entry_points_advanced.len(),

                write: self.write,
                allow_overwrite: self.allow_overwrite,
                incremental: self.incremental,
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
    pub source_root: String,
    pub sources_content: SourcesContent,

    pub target: Target,
    pub format: Format,
    pub global_name: String,
    pub engines: Vec<Engine>,

    pub minify_whitespace: bool,
    pub minify_identifiers: bool,
    pub minify_syntax: bool,
    pub charset: Charset,
    pub tree_shaking: TreeShaking,
    pub legal_comments: LegalComments,

    pub jsx_mode: JSXMode,
    pub jsx_factory: String,
    pub jsx_fragment: String,

    pub tsconfig_raw: String,
    pub footer: String,
    pub banner: String,

    pub define: HashMap<String, String>,
    pub pure: Vec<String>,
    pub keep_names: bool,

    pub source_file: String,
    pub loader: Loader,
}

pub struct TransformOptions {
    // We keep data that fields of ffiapi_ptr point to.
    source_root: String,
    global_name: String,
    engines: Vec<FfiapiEngine>,
    jsx_factory: String,
    jsx_fragment: String,
    tsconfig_raw: String,
    footer: String,
    banner: String,
    define: Vec<FfiapiMapStringStringEntry>,
    pure: Vec<GoString>,
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
            source_root: "".to_string(),
            sources_content: SourcesContent::Include,
            target: Target::ESNext,
            format: Format::Default,
            global_name: "".to_string(),
            engines: vec![],
            minify_whitespace: false,
            minify_identifiers: false,
            minify_syntax: false,
            charset: Charset::Default,
            tree_shaking: TreeShaking::Default,
            legal_comments: LegalComments::Default,
            jsx_mode: JSXMode::Transform,
            jsx_factory: "".to_string(),
            jsx_fragment: "".to_string(),
            tsconfig_raw: "".to_string(),
            footer: "".to_string(),
            banner: "".to_string(),
            define: Default::default(),
            pure: vec![],
            keep_names: false,
            source_file: "".to_string(),
            loader: Loader::None
        }
    }

    pub fn build(self) -> Arc<TransformOptions> {
        let mut res = Arc::new(TransformOptions {
            // We move into Arc first before creating pointers to data in it, as the move to the
            // heap by Arc should change the data's location.
            source_root: self.source_root,
            global_name: self.global_name,
            engines: transform(self.engines, FfiapiEngine::from_engine),
            jsx_factory: self.jsx_factory,
            jsx_fragment: self.jsx_fragment,
            tsconfig_raw: self.tsconfig_raw,
            footer: self.footer,
            banner: self.banner,
            define: transform(self.define, FfiapiMapStringStringEntry::from_map_entry),
            pure: transform(self.pure, GoString::from_string),
            source_file: self.source_file,
            ffiapi_ptr: std::ptr::null(),
        });

        unsafe {
            let ffiapi_ptr = Box::into_raw(Box::new(FfiapiTransformOptions {
                source_map: self.source_map as u8,
                source_root: GoString::from_bytes_unmanaged(res.source_root.as_bytes()),
                sources_content: self.sources_content as u8,

                target: self.target as u8,
                format: self.format as u8,
                global_name: GoString::from_bytes_unmanaged(res.global_name.as_bytes()),
                engines: get_allocation_pointer(&res.engines),
                engines_len: res.engines.len(),

                minify_whitespace: self.minify_whitespace,
                minify_identifiers: self.minify_identifiers,
                minify_syntax: self.minify_syntax,
                charset: self.charset as u8,
                tree_shaking: self.tree_shaking as u8,
                legal_comments: self.legal_comments as u8,

                jsx_mode: self.jsx_mode as u8,
                jsx_factory: GoString::from_bytes_unmanaged(res.jsx_factory.as_bytes()),
                jsx_fragment: GoString::from_bytes_unmanaged(res.jsx_fragment.as_bytes()),
                tsconfig_raw: GoString::from_bytes_unmanaged(res.tsconfig_raw.as_bytes()),
                footer: GoString::from_bytes_unmanaged(res.footer.as_bytes()),
                banner: GoString::from_bytes_unmanaged(res.banner.as_bytes()),

                define: get_allocation_pointer(&res.define),
                define_len: res.define.len(),
                pure: FfiapiGoStringGoSlice::from_vec_unamanged(&res.pure),
                keep_names: self.keep_names,

                source_file: GoString::from_bytes_unmanaged(res.source_file.as_bytes()),
                loader: self.loader as u8,
            }));
            Arc::get_mut(&mut res).unwrap().ffiapi_ptr = ffiapi_ptr;
        };

        res
    }
}

pub struct TransformResult {
    pub code: StrContainer,
    pub map: StrContainer,
    pub errors: SliceContainer<Message>,
    pub warnings: SliceContainer<Message>,
}
