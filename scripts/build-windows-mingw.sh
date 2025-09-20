#!/usr/bin/env bash
set -euo pipefail

# Cross‑compile GFV for Windows (x86_64-pc-windows-gnu) from Linux using MinGW.
# Requirements on Debian/Ubuntu:
#   sudo apt-get update && sudo apt-get install -y mingw-w64
#   rustup target add x86_64-pc-windows-gnu

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

TARGET="x86_64-pc-windows-gnu"
BIN_DIR="target/${TARGET}/release"
OUT_DIR="releases"

echo "Ensuring Rust target ${TARGET} is installed..."
rustup target add "${TARGET}" >/dev/null 2>&1 || true

echo "Building release for ${TARGET}..."
cargo build --release --target "${TARGET}"

EXE="${BIN_DIR}/gfv.exe"
if [[ ! -f "${EXE}" ]]; then
  echo "Build did not produce ${EXE}. Ensure MinGW toolchain is installed (mingw-w64)." >&2
  exit 1
fi

mkdir -p "${OUT_DIR}"
VER=$(sed -n 's/^version\s*=\s*"\(.*\)"/\1/p' Cargo.toml | head -n1)
ZIP="${OUT_DIR}/gfv-${VER}-windows-x86_64.zip"

echo "Packaging ${EXE} -> ${ZIP}"
cd "${BIN_DIR}"
zip -9 "${REPO_ROOT}/${ZIP}" gfv.exe >/dev/null

cd "${REPO_ROOT}"
sha256sum "${ZIP}" | awk '{print $1"  "$2}' > "${OUT_DIR}/SHA256SUMS-${VER}-windows.txt"
echo "Done: ${ZIP}"

