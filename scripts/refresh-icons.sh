#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ICON_ZIP="${REPO_ROOT}/icon_pack.zip"
OUT_DIR="${REPO_ROOT}/assets/icons"
TMP_DIR="$(mktemp -d)"

if ! python3 -c "import PIL" >/dev/null 2>&1; then
  echo "This script requires the Python Pillow package. Install with: pip install pillow" >&2
  exit 1
fi

cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

if [[ ! -f "${ICON_ZIP}" ]]; then
  echo "Expected ${ICON_ZIP}. Place icon_pack.zip in the repository root." >&2
  exit 1
fi

unzip -q "${ICON_ZIP}" -d "${TMP_DIR}"
PNG_SOURCE="${TMP_DIR}"
mkdir -p "${OUT_DIR}"

python3 - "$PNG_SOURCE" "$OUT_DIR" <<'PY'
import sys
from pathlib import Path
from PIL import Image

png_source = Path(sys.argv[1])
out_dir = Path(sys.argv[2])
target_sizes = [16, 24, 32, 48, 64, 96, 128, 256, 384, 512]

available = {}
for path in png_source.glob("icon_*.png"):
    stem = path.stem
    try:
        size = int(stem.split("_")[1])
    except (IndexError, ValueError):
        continue
    available[size] = path

if not available:
    raise SystemExit("icon_pack.zip did not contain icon_*.png entries")

largest_size = max(available)
with Image.open(available[largest_size]) as base_img:
    base_img = base_img.convert("RGBA")
    for size in target_sizes:
        dest = out_dir / f"icon_{size}.png"
        if size in available:
            with Image.open(available[size]) as src:
                src.convert("RGBA").save(dest)
        else:
            resized = base_img.resize((size, size), Image.LANCZOS)
            resized.save(dest)
        print(f"{size} -> {dest}")
PY

ICO_SOURCE="${TMP_DIR}/icon.ico"
if [[ -f "${ICO_SOURCE}" ]]; then
  cp "${ICO_SOURCE}" "${OUT_DIR}/icon.ico"
  echo "ico -> ${OUT_DIR}/icon.ico"
else
  echo "WARN: icon_pack.zip missing icon.ico" >&2
fi
