gfv 2.0.4

Highlights
- Retire the deprecated `gfv_icon_pack` bundle; the app now ships only the refreshed icon set in `assets/icons/`.
- New automation: `scripts/refresh-icons.sh` regenerates the PNG/ICO set from `icon_pack.zip`, and `scripts/update-desktop-icons.sh` clears caches before reinstalling Linux desktop assets.
- Packaging scripts (`build-release`, `build-musl`, `build-windows`) automatically refresh icons to keep deliverables in sync.

Packaging
- Linux: `scripts/build-release.sh` (or `cargo build --release`) then `tar -C target/release -czf releases/gfv-2.0.4-linux-x86_64.tar.gz gfv`.
- Checksums: `sha256sum releases/gfv-2.0.4-linux-x86_64.tar.gz > releases/SHA256SUMS-2.0.4.txt`.
- Windows: run `scripts/build-windows.ps1` or `scripts/build-windows-mingw.sh`; archives land in `releases/` with matching hashes.

Upgrade Notes
- If you previously installed the legacy icons, rerun `scripts/update-desktop-icons.sh` to purge cached icons from `~/.local/share/icons/hicolor`.
- Ensure Python Pillow is installed (`pip install pillow`) before using `scripts/refresh-icons.sh` or scripts that depend on it.
