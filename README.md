Gemini File Viewer (Linux)

Pre-beta
- This app is currently in pre-beta. Features and UI may change and bugs are expected. Please report issues and suggestions.

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
- If you hit Wayland issues, force X11: WINIT_UNIX_BACKEND=x11 ./gemini-file-viewer-linux

Build
- Release (optimized):
  cargo build --release
  ./target/release/gemini-file-viewer-linux

- Debug (faster compile):
  cargo build
  ./target/debug/gemini-file-viewer-linux

Portable-ish (musl)
- Install musl target and tools:
  rustup target add x86_64-unknown-linux-musl
  sudo apt install -y musl-tools
- Build:
  cargo build --release --target x86_64-unknown-linux-musl
- Note: You still need GPU/GL drivers at runtime; GL can’t be fully static.

Packaging
- Simple tarball:
  tar -C target/release -czf gemini-file-viewer-linux-x86_64.tar.gz gemini-file-viewer-linux
- Optional desktop integration: ship a .desktop file and icon under ~/.local/share/applications and ~/.local/share/icons.

Scripts
- scripts/install-deps-ubuntu.sh: Installs common build deps on Ubuntu/Debian.
- scripts/build-release.sh: Builds size-optimized release.
- scripts/build-musl.sh: Builds a musl release (adds target if missing).
