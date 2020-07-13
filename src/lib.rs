use std::os::raw::{c_char, c_ulonglong, c_void};
#[cfg(feature = "use-dylib")]
use dlopen::wrapper::{Container, WrapperApi};
#[cfg(feature = "use-dylib")]
use dlopen_derive::WrapperApi;
#[cfg(feature = "use-dylib")]
use lazy_static::lazy_static;

mod test;

#[cfg(not(feature = "use-dylib"))]
extern "C" {
    fn MinifyJs(code: GoString, out_len: *mut c_ulonglong) -> *const c_void;
}

#[cfg(feature = "use-dylib")]
#[derive(WrapperApi)]
struct Api {
    MinifyJs: unsafe extern "C" fn(code: GoString, out_len: *mut c_ulonglong) -> *const c_void,
}

#[cfg(feature = "use-dylib")]
lazy_static! {
    static ref DYLIB_CONT: Container<Api> = unsafe {
        Container::load("esbuild.dll")
    }.expect("open dynamic library");
}

#[repr(C)]
struct GoString {
    a: *const c_char,
    b: i64,
}

pub fn esbuild<'i, 'o>(code: &'i [u8]) -> &'o [u8] {
    let go_string = GoString {
        a: code as *const [u8] as *const c_char,
        b: code.len() as i64,
    };
    let mut out_len = 0;

    unsafe {
        #[cfg(not(feature = "use-dylib"))]
        let result = MinifyJs(go_string, &mut out_len) as *mut u8;

        #[cfg(feature = "use-dylib")]
        let result = DYLIB_CONT.MinifyJs(go_string, &mut out_len) as *mut u8;

        core::slice::from_raw_parts(result, out_len as usize)
    }
}
