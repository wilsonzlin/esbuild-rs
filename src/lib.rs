use std::os::raw::{c_char, c_ulonglong, c_void};

mod test;

extern "C" {
    fn MinifyJs(code: GoString, out_len: *mut c_ulonglong) -> *const c_void;
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
        let result = MinifyJs(go_string, &mut out_len);
        core::slice::from_raw_parts(result as *mut u8, out_len as usize)
    }
}
