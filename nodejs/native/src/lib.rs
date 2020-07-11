use neon::prelude::*;
use esbuild_rs::esbuild_unchecked;

fn minify(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let src = cx.argument::<JsBuffer>(0)?;
    let res = cx.borrow(&src, |src| unsafe {
        esbuild_unchecked(src.as_slice::<u8>())
    });
    let mut rv = JsBuffer::new(&mut cx, res.len() as u32)?;
    cx.borrow_mut(&mut rv, |rv| rv.as_mut_slice::<u8>().copy_from_slice(res));
    Ok(rv)
}

register_module!(mut cx, {
    cx.export_function("minify", minify)
});
