package main

// #include <string.h>
//
// typedef void (*minify_js_callback) (
//   void*,
//   unsigned long long
// );
//
// static void call_callback(
//   minify_js_callback f,
//   void* cb_data,
//   unsigned long long output_len
// ) {
//   f(cb_data, output_len);
// }
import "C"
import (
	"github.com/evanw/esbuild/pkg/api"
	"unsafe"
)

func KickoffMinifyJs(
	code string,
	out unsafe.Pointer,
	cb C.minify_js_callback,
	cbData unsafe.Pointer,
) {
	result := api.Transform(code, api.TransformOptions{
		MinifyWhitespace:  true,
		MinifyIdentifiers: true,
		MinifySyntax:      true,
	})

	resCode := result.JS
	resLen := len(resCode)

	C.memcpy(out, unsafe.Pointer(&resCode[0]), C.ulong(resLen))
	C.call_callback(cb, cbData, C.ulonglong(resLen))
}

//export MinifyJs
func MinifyJs(
	code string,
	out unsafe.Pointer,
	cb C.minify_js_callback,
	cbData unsafe.Pointer,
) {
	go KickoffMinifyJs(code, out, cb, cbData)
}

func main() {}
