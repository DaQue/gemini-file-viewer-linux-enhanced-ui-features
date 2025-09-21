#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
rustup target add x86_64-unknown-linux-musl || true
./scripts/refresh-icons.sh
cargo build --release --target x86_64-unknown-linux-musl
ls -lh target/x86_64-unknown-linux-musl/release/gfv
