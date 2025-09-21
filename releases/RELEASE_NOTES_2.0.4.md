gfv 2.0.4

Changes
- Remove the deprecated `gfv_icon_pack` assets and fallback; installer now expects the refreshed PNG set or `icon_pack.zip`.
- Keep Linux desktop integration scripts aligned with the new icon flow.

Packaging
- Build: `cargo build --release`
- Icons: run `scripts/update-desktop-icons.sh` after updating `assets/icons/` to refresh the local icon cache.

Notes
- If you previously installed the legacy icons, rerun `scripts/update-desktop-icons.sh` to clear cached copies.
