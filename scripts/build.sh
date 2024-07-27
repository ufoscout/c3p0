#!/usr/bin/env bash
set -e
set -x
export RUST_BACKTRACE=full

cargo build 
cargo build --all-features