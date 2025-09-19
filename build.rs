#[cfg(windows)]
fn main() {
    // Embed the prebuilt multi-size ICO for Windows executables.
    // Keeps branding consistent with Linux desktop icons.
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icons/icon.ico");
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {}
