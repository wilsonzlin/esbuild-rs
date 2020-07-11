package main

import "C"
import "github.com/evanw/esbuild/pkg/api"

//export MinifyJs
func MinifyJs(code string, out_len *C.ulonglong) *C.byte {
	result := api.Transform(code, api.TransformOptions{
		MinifyWhitespace:  true,
		MinifyIdentifiers: true,
		MinifySyntax:      true,
	})

	*out_len = len(result.JS)

	return C.CBytes(result.JS)
}

func main() {}
