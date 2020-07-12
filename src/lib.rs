use std::ffi::CString;
use std::os::raw::{c_char, c_ulonglong, c_void};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;

mod test;

#[cfg(not(feature = "use-dylib"))]
extern "C" {
    fn MinifyJs(code: GoString, out_len: *mut c_ulonglong) -> *const c_void;
}

#[cfg(feature = "use-dylib")]
#[derive(WrapperApi)]
struct Api {
    MinifyJs: unsafe extern "C" fn(code: GoString) -> *const c_char,
}

#[cfg(feature = "use-dylib")]
lazy_static! {
    let mut ref DYLIB_CONT: Container<Api> = unsafe {
        Container::load("esbuild.dll")
    }.expect("open dynamic library");
}

#[repr(C)]
struct GoString {
    a: *const c_char,
    b: i64,
}

pub unsafe fn esbuild_unchecked<'i, 'o>(code: &'i [u8]) -> &'o [u8] {
    let go_string = GoString {
        a: code as *const [u8] as *const c_char,
        b: code.len() as i64,
    };
    let mut out_len = 0;

    #[cfg(not(feature = "use-dylib"))]
    let result = MinifyJs(go_string, &mut out_len) as *mut u8;

    #[cfg(feature = "use-dylib")]
    let result = CONT.MinifyJs(go_string, &mut out_len) as *mut u8;

    core::slice::from_raw_parts(result, out_len as usize)
}

pub fn esbuild(code: &str) -> &str {
    let c_code = CString::new(code).expect("CString::new failed");
    let out_bytes = unsafe { esbuild_unchecked(&c_code.as_bytes_with_nul()) };
    std::str::from_utf8(out_bytes).expect("decoding UTF-8")
}
