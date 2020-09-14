#!/usr/bin/env sh

set -e
touch src/main.rs
cargo clippy
