package main

// typedef void (*minify_js_callback) (
//   void*,
//   void*,
//   unsigned long long
// );
//
// static void call_callback(
//   minify_js_callback f,
//   void* cb_data,
//   void* output,
//   unsigned long long output_len
// ) {
//   f(cb_data, output, output_len);
// }
import "C"
import (
	"github.com/evanw/esbuild/pkg/api"
	"unsafe"
)

func KickoffMinifyJs(
	code string,
	cb C.minify_js_callback,
	cbData unsafe.Pointer,
) {
	result := api.Transform(code, api.TransformOptions{
		MinifyWhitespace:  true,
		MinifyIdentifiers: true,
		MinifySyntax:      true,
	})

	resCode := result.JS
	ptr := C.CBytes(resCode)
	C.call_callback(cb, cbData, ptr, C.ulonglong(len(resCode)))
}

//export MinifyJs
func MinifyJs(
	code string,
	cb C.minify_js_callback,
	cbData unsafe.Pointer,
) {
	go KickoffMinifyJs(code, cb, cbData)
}

func main() {}
