#!/usr/bin/env bash

set -e

mkdir -p tmp
export GOPATH="$(realpath tmp)"

if [[ "$OSTYPE" == "cydwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
  out_name="esbuild.lib"
else
  out_name="libesbuild.a"
fi

go get ./
go build -buildmode=c-archive -o build/$out_name native/esbuild.go
