# esbuild-rs

Rust wrapper for esbuild using FFI and Cgo. [esbuild](https://github.com/evanw/esbuild) is an extremely fast JavaScript minifier written in Go.

## Using

This library requires Go 1.13 or higher. The Go source is included and compiled at build time. The build will not interfere with or create files in `GOROOT` or `GOPATH`, or download any Go dependencies.

Check the [docs](https://docs.rs/esbuild-rs/) for the API.

## Async

A [fork of esbuild](https://github.com/wilsonzlin/esbuild-lib) is used to allow taking advantage of the Go scheduler for optimal concurrency. Friendly functions that use Futures are available, which are suitable for most cases; for advanced usage, direct functions that take a callback and return immediately are also available, requiring additional concurrency management on top.

## Windows

Since Cgo uses GCC, a GCC compiler is required to compile the Go library, even if the MSVC Rust toolchain is used. [TDM-GCC](https://jmeubank.github.io/tdm-gcc/) is recommended.

If the `msvc` toolchain is used, this library will compile a DLL, embed it into the resulting Rust binary, and load it at runtime from memory using [MemoryModule](https://github.com/wilsonzlin/memorymodule-rs). This happens transparently at build time and run time, and requires no extra effort.
