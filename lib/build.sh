#!/usr/bin/env bash

set -e

mkdir -p tmp
export GOPATH="$(realpath tmp)"

if [[ "$OSTYPE" == "cydwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
  out_name="esbuild.dll"
else
  out_name="libesbuild.so"
fi

go get ./
go build -buildmode=c-shared -o build/$out_name esbuild.go
