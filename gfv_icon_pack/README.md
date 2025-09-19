# GFV Icon Pack (Windows + Linux)

Contents:
- `icons/png/icon_<size>.png` — 16..512 px PNGs
- `icons/windows/app_icon.ico` — multi-resolution ICO (16–256)
- `linux/gfv.desktop` — launcher entry
- `linux/install_user_icons.sh` — user-level installer
- `windows/build.rs` — Rust icon embed (winres)
- Platform READMEs

Quick Start
- **Windows**: add `winres`, copy `windows/build.rs`, build release.
- **Linux**: run `linux/install_user_icons.sh` (edit `Exec=` first).
