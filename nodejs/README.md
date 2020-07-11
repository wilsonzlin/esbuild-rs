# esbuild-native

Experimental native esbuild library for Node.js.

## Building

You'll need Rust, Go, and Node.js.

Install the dependencies in this folder using `npm i`, and then run `npm run build`.

To test it out, open `node` and `require('.')`. There is only one function right now; see [src/index.ts](src/index.ts).

## Benchmarking

Install the dependencies in [bench](./bench), and then run `node bench/run.js`.
