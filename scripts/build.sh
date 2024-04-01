#!/usr/bin/env sh
set -e
set -x
export RUST_BACKTRACE=full

cargo build 
cargo build --all-features