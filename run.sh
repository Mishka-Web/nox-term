#!/usr/bin/env sh
set -eu
cargo build --release
exec ./target/release/nox "$@"
