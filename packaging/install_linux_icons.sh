#!/usr/bin/env bash
set -euo pipefail

# Install GFV icons and desktop entry into ~/.local for the current user.

APP_NAME="gfv"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN_PATH="${REPO_ROOT}/target/release/gfv"
# Prefer freshly generated icons from assets/icons; fallback to gfv_icon_pack if missing
if [[ -d "${REPO_ROOT}/assets/icons" ]]; then
  ICON_DIR="${REPO_ROOT}/assets/icons"
elif [[ -d "${REPO_ROOT}/gfv_icon_pack/icons/png" ]]; then
  ICON_DIR="${REPO_ROOT}/gfv_icon_pack/icons/png"
else
  echo "No icon directory found; expected assets/icons or gfv_icon_pack/icons/png" >&2
  exit 1
fi

XDG_ICONS="${HOME}/.local/share/icons/hicolor"
XDG_APPS="${HOME}/.local/share/applications"

echo "Installing icons to ${XDG_ICONS} (prefer assets; fallback to pack)..."
for size in 16 24 32 48 64 96 128 256 384 512; do
  src="${REPO_ROOT}/assets/icons/icon_${size}.png"
  [[ -f "${src}" ]] || src="${REPO_ROOT}/gfv_icon_pack/icons/png/icon_${size}.png"
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
