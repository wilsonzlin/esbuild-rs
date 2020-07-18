# esbuild-rs

Rust wrapper for esbuild using FFI and Cgo. [esbuild](https://github.com/evanw/esbuild) is an extremely fast JavaScript minifier written in Go.

Takes advantage of the Go scheduler for optimal concurrency by using Rust callbacks. Uses a fork of [esbuild](https://github.com/wilsonzlin/esbuild-lib) to achieve this.

## Using

This library requires Go 1.13 or higher. The Go source is included and compiled at build time. The build will not interfere with or create files in `GOROOT` or `GOPATH`, and does not require downloading any dependencies.

Check the [docs](https://docs.rs/esbuild-rs/) for the API.

## Windows

Since Cgo uses GCC, it's recommended to use the `gnu` toolchain and [TDM-GCC](https://jmeubank.github.io/tdm-gcc/).

If the `msvc` toolchain is used, this library will instead compile a DLL, embed it into the resulting Rust binary, and load it at runtime in memory using [MemoryModule](https://github.com/wilsonzlin/memorymodule-rs). This happens transparently at build time and run time, and requires no extra effort.
