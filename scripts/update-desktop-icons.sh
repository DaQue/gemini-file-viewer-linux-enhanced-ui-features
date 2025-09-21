#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HICOLOR_DIR="${HOME}/.local/share/icons/hicolor"
DESKTOP_APPS="${HOME}/.local/share/applications"

echo "Refreshing repository icons from icon_pack.zip..."
"${REPO_ROOT}/scripts/refresh-icons.sh"

echo "Removing cached hicolor icons..."
if [[ -d "${HICOLOR_DIR}" ]]; then
  removed=0
  while IFS= read -r path; do
    if rm -f "${path}"; then
      echo "  removed ${path}"
      removed=1
    else
      echo "  WARN: could not remove ${path}" >&2
    fi
  done < <(find "${HICOLOR_DIR}" -maxdepth 3 -type f -name 'gfv.png')
  if [[ ${removed} -eq 0 ]]; then
    echo "  none found"
  fi
else
  echo "  ${HICOLOR_DIR} does not exist; skipping"
fi

echo "Reinstalling icons and desktop entry..."
"${REPO_ROOT}/packaging/install_linux_icons.sh"

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  echo "Refreshing GTK icon cache..."
  gtk-update-icon-cache -f "${HICOLOR_DIR}" || true
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  echo "Updating desktop database..."
  update-desktop-database "${DESKTOP_APPS}" || true
fi

echo "Done. If the launcher still shows the old icon, restart your shell (e.g. GNOME: Alt+F2, then 'r')."
