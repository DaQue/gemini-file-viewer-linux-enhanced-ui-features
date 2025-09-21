#!/usr/bin/env bash
set -euo pipefail

# Install GFV icons and desktop entry into ~/.local for the current user.

APP_NAME="gfv"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN_PATH="${REPO_ROOT}/target/release/gfv"

TMP_ICON_DIR=""
cleanup() {
  if [[ -n "${TMP_ICON_DIR}" && -d "${TMP_ICON_DIR}" ]]; then
    rm -rf "${TMP_ICON_DIR}"
  fi
}
trap cleanup EXIT

# Prefer freshly generated icons from assets/icons; fall back to icon_pack.zip.
if [[ -d "${REPO_ROOT}/assets/icons" ]]; then
  ICON_DIR="${REPO_ROOT}/assets/icons"
elif [[ -f "${REPO_ROOT}/icon_pack.zip" ]]; then
  TMP_ICON_DIR="$(mktemp -d)"
  unzip -q "${REPO_ROOT}/icon_pack.zip" -d "${TMP_ICON_DIR}"
  ICON_DIR="${TMP_ICON_DIR}"
else
  echo "No icon directory found; expected assets/icons or icon_pack.zip" >&2
  exit 1
fi

XDG_ICONS="${HOME}/.local/share/icons/hicolor"
XDG_APPS="${HOME}/.local/share/applications"

echo "Installing icons to ${XDG_ICONS} (prefer assets; fallback to pack)..."
for size in 16 24 32 48 64 96 128 256 384 512; do
  src="${REPO_ROOT}/assets/icons/icon_${size}.png"
  if [[ ! -f "${src}" ]]; then
    alt="${ICON_DIR}/icon_${size}.png"
    [[ -f "${alt}" ]] && src="${alt}"
  fi
  dest_dir="${XDG_ICONS}/${size}x${size}/apps"
  mkdir -p "${dest_dir}"
  if [[ -f "${src}" ]]; then
    install -m 0644 "${src}" "${dest_dir}/${APP_NAME}.png"
    echo "  ${size}x${size} -> ${dest_dir}/${APP_NAME}.png"
  else
    echo "  WARN: missing ${src}, skipping" >&2
  fi
done

echo "Installing desktop entry to ${XDG_APPS}..."
mkdir -p "${XDG_APPS}"
DESKTOP_FILE="${XDG_APPS}/${APP_NAME}.desktop"
sed \
  -e "s|^Exec=.*$|Exec=${BIN_PATH} %F|" \
  -e "s|^Icon=.*$|Icon=${APP_NAME}|" \
  -e "s|^Name=.*$|Name=GFV|" \
  -e "s|^StartupWMClass=.*$|StartupWMClass=${APP_NAME}|" \
  "${REPO_ROOT}/packaging/gfv.desktop" > "${DESKTOP_FILE}"
chmod 0644 "${DESKTOP_FILE}"
update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true

# Refresh icon cache if possible (won't exist on all distros)
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f "${XDG_ICONS}" || true
fi

echo "Done. You may need to log out/in or restart your shell/dock."
