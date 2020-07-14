#!/usr/bin/env bash

set -e

if [[ "$OSTYPE" == "cydwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
  out_name="esbuild.dll"
else
  out_name="libesbuild.so"
fi

go build -buildmode=c-shared -o build/$out_name esbuild.go
