# Linux Icon & .desktop Setup

## User-level install
1. Extract this zip.
2. Edit `linux/gfv.desktop`: set `Exec=/absolute/path/to/gfv` or a PATH name.
3. Run:
```bash
chmod +x linux/install_user_icons.sh
./linux/install_user_icons.sh
```
4. Log out/in if the taskbar icon doesn't update.

This installs PNGs into `~/.local/share/icons/hicolor/...` and `gfv.desktop` into `~/.local/share/applications/`.

## System-wide (optional; sudo)
Install PNGs to `/usr/share/icons/hicolor/<size>x<size>/apps/gfv.png` and the desktop file to `/usr/share/applications/gfv.desktop`, then refresh caches.
