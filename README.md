gfv (Linux)

Stable
- gfv is stable and actively maintained. Please report issues and suggestions.

Project status
- This repository is the canonical home for GFV. Older GitHub repositories such as
  [`DaQue/FileViewer`](https://github.com/DaQue/FileViewer),
  [`DaQue/gemini-file-viewer-linux`](https://github.com/DaQue/gemini-file-viewer-linux), and
  [`DaQue/file-viewer2`](https://github.com/DaQue/file-viewer2) are now archived and kept only for
  historical reference.
- Latest release: [v2.0.5](https://github.com/DaQue/gemini-file-viewer-linux-enhanced-ui-features/releases/tag/v2.0.5)

A lightweight desktop viewer for text/code and images, built with egui/eframe. This variant is set up for Linux builds with small, portable binaries.

Highlights
- Persistent settings: Remembers Dark Mode, Line Numbers, and Recent Files across runs.
- Recent Files: Wide, non-wrapping menu with a Clear option.
- Image formats: PNG, JPEG, GIF, BMP, WEBP (scaled smoothly).
- Prev/Next navigation: Navigate sibling files in the same folder when viewing images or text files.
- Size-optimized release: opt-level="z", LTO, panic=abort, strip=true.

Usage
- Open a file via Open File or Recent Files.
- When viewing an image or a text file, Prev and Next buttons appear in the toolbar to move to the previous/next sibling file in the same directory.
- Use the status bar to copy the path or open the containing folder.

Prerequisites
- Toolchain: Rust stable (rustup)
- System libraries (Ubuntu/Debian):
  sudo apt update && sudo apt install -y \
    build-essential pkg-config \
    libx11-dev libxrandr-dev libxi-dev libxcursor-dev \
    libgl1-mesa-dev libegl1-mesa-dev \
    libxkbcommon-dev libwayland-dev \
    xdg-desktop-portal xdg-desktop-portal-gtk

Notes on dialogs and Wayland
- rfd uses desktop portals on modern Linux. Ensure xdg-desktop-portal and a backend (gtk/gnome/kde) are running.
- If you hit Wayland issues, force X11: WINIT_UNIX_BACKEND=x11 ./gfv

Build
- Release (optimized):
  cargo build --release
  ./target/release/gfv

- Debug (faster compile):
  cargo build
  ./target/debug/gfv

Install
- Linux (user scope):
  1. Ensure `icon_pack.zip` is present in the repo root.
  2. ./scripts/build-release.sh
  3. ./scripts/update-desktop-icons.sh (installs icons + desktop entry under `~/.local`).
  4. Launch via `gfv` in your app list or `target/release/gfv`.
- Windows:
  1. Place `icon_pack.zip` beside the repository.
  2. Native toolchain: `powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1`.
     Cross (from Linux): `bash scripts/build-windows-mingw.sh` (requires mingw-w64 + target).
  3. Grab the zip in `releases/` and extract/run `gfv.exe`.

Portable-ish (musl)
- Install musl target and tools:
  rustup target add x86_64-unknown-linux-musl
  sudo apt install -y musl-tools
- Build:
  cargo build --release --target x86_64-unknown-linux-musl
- Note: You still need GPU/GL drivers at runtime; GL can’t be fully static.

Packaging
- Simple tarball:
  tar -C target/release -czf gfv-linux-x86_64.tar.gz gfv
 - Desktop integration (Linux):
   - Copy icons to user icon theme (HiDPI sizes optional but recommended):
     for s in 16 32 48 64 128 256; do mkdir -p "$HOME/.local/share/icons/hicolor/${s}x${s}/apps"; cp assets/icons/icon_${s}.png "$HOME/.local/share/icons/hicolor/${s}x${s}/apps/gfv.png"; done
     # If available, also provide 512x512: copy assets/icons/icon_512.png to ~/.local/share/icons/hicolor/512x512/apps/gfv.png
   - Install desktop entry:
     mkdir -p "$HOME/.local/share/applications" && cp os/linux/gfv.desktop "$HOME/.local/share/applications/gfv.desktop"
   - Refresh caches (if present): update-desktop-database "$HOME/.local/share/applications"; gtk-update-icon-cache -f "$HOME/.local/share/icons/hicolor"
   - Alternative template: see packaging/gfv.desktop (edit Exec= to your binary path)

Windows
- EXE now embeds an icon when building on Windows (via build.rs). Pinned taskbar and drag/drop behave as expected.

Scripts
- scripts/install-deps-ubuntu.sh: Installs common build deps on Ubuntu/Debian.
- scripts/build-release.sh: Builds size-optimized release.
- scripts/build-musl.sh: Builds a musl release (adds target if missing).
- scripts/refresh-icons.sh: Syncs `assets/icons/` from `icon_pack.zip` (called automatically by other build scripts; run manually after updating the pack).
- scripts/update-desktop-icons.sh: Refreshes icons, clears cached copies under `~/.local/share/icons/hicolor`, and reinstalls the desktop entry.
