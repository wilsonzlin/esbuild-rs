use std::ffi::{CStr, CString};
use std::os::raw::c_char;

mod test;

extern "C" {
    fn MinifyJs(code: GoString) -> *const c_char;
}

#[repr(C)]
struct GoString {
    a: *const c_char,
    b: i64,
}

pub fn esbuild(code: &str) -> &str {
    let c_code = CString::new(code).expect("CString::new failed");
    let ptr = c_code.as_ptr();
    let go_string = GoString {
        a: ptr,
        b: c_code.as_bytes().len() as i64,
    };
    let result = unsafe { MinifyJs(go_string) };
    let c_str = unsafe { CStr::from_ptr(result) };
    c_str.to_str().expect("decode C string")
}
