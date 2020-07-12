package main

import "C"
import (
	"github.com/evanw/esbuild/pkg/api"
	"unsafe"
)

//export MinifyJs
func MinifyJs(code string, out_len *C.ulonglong) unsafe.Pointer {
	result := api.Transform(code, api.TransformOptions{
		MinifyWhitespace:  true,
		MinifyIdentifiers: true,
		MinifySyntax:      true,
	})

	*out_len = C.ulonglong(len(result.JS))

	return C.CBytes(result.JS)
}

func main() {}
