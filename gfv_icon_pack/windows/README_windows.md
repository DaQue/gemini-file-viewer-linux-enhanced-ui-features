# Windows Icon Setup (Rust + winres)

1. Add the dependency:

```toml
# Cargo.toml
[package]
build = "build.rs"

[build-dependencies]
winres = "0.1"
```

2. Copy `windows/build.rs` to your crate root as `build.rs`.
3. Ensure `icons/windows/app_icon.ico` exists relative to your crate.
4. Build:
```powershell
cargo clean
cargo build --release
```

The resulting `.exe` will carry the icon for the taskbar/Start/Explorer.
