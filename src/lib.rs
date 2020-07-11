use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;

mod test;

#[repr(C)]
struct GoString {
    a: *const c_char,
    b: i64,
}

#[derive(WrapperApi)]
struct Api {
    MinifyJs: unsafe extern "C" fn(code: GoString) -> *const c_char,
}

pub fn esbuild(code: &str) -> &str {
    let mut cont: Container<Api> = unsafe {
        Container::load("esbuild.dll")
    }.expect("open dynamic library");
    let c_code = CString::new(code).expect("CString::new failed");
    let ptr = c_code.as_ptr();
    let go_string = GoString {
        a: ptr,
        b: c_code.as_bytes().len() as i64,
    };
    let result = unsafe { cont.MinifyJs(go_string) };
    let c_str = unsafe { CStr::from_ptr(result) };
    c_str.to_str().expect("decode C string")
}
