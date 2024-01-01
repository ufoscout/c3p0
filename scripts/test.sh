#!/usr/bin/env sh
set -e
export RUST_BACKTRACE=full

cargo test 
cargo test --all-features

docker system prune -f && docker volume prune -f
