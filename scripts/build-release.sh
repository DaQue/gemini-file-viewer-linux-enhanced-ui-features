#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
./scripts/refresh-icons.sh
cargo build --release
ls -lh target/release/gfv
