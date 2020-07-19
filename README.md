# esbuild-rs

Rust wrapper for esbuild using FFI and Cgo. [esbuild](https://github.com/evanw/esbuild) is an extremely fast JavaScript minifier written in Go.

## Using

This library requires Go 1.13 or higher. The Go source is included and compiled at build time. The build will not interfere with or create files in `GOROOT` or `GOPATH`, or download any dependencies.

Check the [docs](https://docs.rs/esbuild-rs/) for the API.

## Async

A fork of [esbuild](https://github.com/wilsonzlin/esbuild-lib) is used to allow taking advantage of the Go scheduler for optimal concurrency.

The APIs in this library simply take a callback, which is called asynchronously from a Goroutine once the process finishes. This means that some Rust concurrency handling is needed on top. Some useful tools are [WaitGroup](https://docs.rs/crossbeam/0.7.3/crossbeam/sync/struct.WaitGroup.html) for waiting for all callbacks to be called before continuing, [channels](https://docs.rs/crossbeam/0.7.3/crossbeam/channel/index.html) for passing results from a callback thread to the main thread, and [deque](https://docs.rs/crossbeam/0.7.3/crossbeam/deque/index.html) for queueing completed results and processing them in parallel. All of these are available from the [crossbeam](https://crates.io/crates/crossbeam) crate.

## Windows

Since Cgo uses GCC, it's recommended to use the `gnu` toolchain and [TDM-GCC](https://jmeubank.github.io/tdm-gcc/).

If the `msvc` toolchain is used, this library will instead compile a DLL, embed it into the resulting Rust binary, and load it at runtime in memory using [MemoryModule](https://github.com/wilsonzlin/memorymodule-rs). This happens transparently at build time and run time, and requires no extra effort.
