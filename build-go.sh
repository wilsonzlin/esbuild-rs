#!/usr/bin/env bash

set -e

go build -buildmode=c-archive -o libesbuild.a esbuild.go
