package main

import "C"
import "github.com/evanw/esbuild/pkg/api"

//export MinifyJs
func MinifyJs(code string) *C.char {
	result := api.Transform(code, api.TransformOptions{
		MinifyWhitespace:  true,
		MinifyIdentifiers: true,
		MinifySyntax:      true,
	})

	return C.CString(string(result.JS))
}

func main() {}
