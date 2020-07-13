package main

// #include <stdlib.h>
//
// typedef void (*minify_js_callback) (
//   void* cb_data,
//   void** min_codes,
//   unsigned long long* min_code_lens,
//   unsigned long long codes_len
// );
//
// static void call_callback(
//   minify_js_callback f,
//   void* cb_data,
//   void** min_codes,
//   unsigned long long* min_code_lens,
//   unsigned long long codes_len
// ) {
//   f(cb_data, min_codes, min_code_lens, codes_len);
// }
//
// static void** allocate_pointer_array(unsigned long long len) {
//   return malloc(sizeof(void*) * len);
// }
//
// static void set_pointer_array_elem(void** ary, unsigned long long i, void* elem) {
//   ary[i] = elem;
// }
//
// static unsigned long long* allocate_ulonglong_array(unsigned long long len) {
//   return malloc(sizeof(unsigned long long) * len);
// }
//
// static void set_ulonglong_array_elem(unsigned long long* ary, unsigned long long i, unsigned long long elem) {
//   ary[i] = elem;
// }
import "C"
import (
	"github.com/evanw/esbuild/pkg/api"
	"sync"
	"unsafe"
)

func KickoffMinifyJs(
	codes []string,
	cb C.minify_js_callback,
	cbData unsafe.Pointer,
) {
	var wg sync.WaitGroup
	cCodesLen := C.ulonglong(len(codes))
	cMinCodes := C.allocate_pointer_array(cCodesLen)
	cMinCodeLens := C.allocate_ulonglong_array(cCodesLen)
	for i, code := range codes {
		wg.Add(1)
		go func(i int, code string) {
			res := api.Transform(code, api.TransformOptions{
				MinifyWhitespace:  true,
				MinifyIdentifiers: true,
				MinifySyntax:      true,
			})
			min := res.JS
			// TODO This might be slow, as it's entering and exiting out of Go repeatedly.
			C.set_pointer_array_elem(cMinCodes, C.ulonglong(i), C.CBytes(min))
			C.set_ulonglong_array_elem(cMinCodeLens, C.ulonglong(i), C.ulonglong(len(min)))
			wg.Done()
		}(i, code)
	}
	wg.Wait()

	C.call_callback(cb, cbData, cMinCodes, cMinCodeLens, cCodesLen)
}

//export MinifyJs
func MinifyJs(
	codes []string,
	cb C.minify_js_callback,
	cbData unsafe.Pointer,
) {
	go KickoffMinifyJs(codes, cb, cbData)
}

func main() {}
