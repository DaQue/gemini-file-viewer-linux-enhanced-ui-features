#!/usr/bin/env bash
set -euo pipefail
APP_NAME="gfv"
ICON_SIZES=(16 24 32 48 64 96 128 256 384 512)

for sz in "${ICON_SIZES[@]}"; do
  mkdir -p "${HOME}/.local/share/icons/hicolor/${sz}x${sz}/apps"
  install -m 0644 "icons/png/icon_${sz}.png" "${HOME}/.local/share/icons/hicolor/${sz}x${sz}/apps/${APP_NAME}.png"
done

mkdir -p "${HOME}/.local/share/applications"
install -m 0644 "linux/gfv.desktop" "${HOME}/.local/share/applications/${APP_NAME}.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${HOME}/.local/share/applications" || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f "${HOME}/.local/share/icons/hicolor" || true
fi

echo "Installed icons and desktop entry for ${APP_NAME}."
echo "Edit linux/gfv.desktop Exec= path if needed."
