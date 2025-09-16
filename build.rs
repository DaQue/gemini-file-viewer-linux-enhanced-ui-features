#[cfg(windows)]
fn main() {
    use std::fs;
    use std::path::PathBuf;
    // Build a simple ICO from our generated icon PNG approach is skipped; we render an ICO on the fly.
    // We'll reuse the same procedural RGBA from src/main.rs but keep this tiny: solid color fallback.
    let size = 64u32;
    let mut rgba: Vec<u8> = vec![0; (size * size * 4) as usize];
    for y in 0..size { for x in 0..size {
        let i = ((y * size + x) * 4) as usize;
        // cyan-ish fallback, diagonal split darker
        let dark = (x as i32) - (y as i32) < 0;
        if dark { rgba[i..i+4].copy_from_slice(&[0x1e,0x29,0x3b,0xff]); } else { rgba[i..i+4].copy_from_slice(&[0x0e,0xa5,0xe9,0xff]); }
    }}
    let image = ico::IconImage::from_rgba_data(size, size, rgba);
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    icon_dir.add_entry(ico::IconDirEntry::encode(&image).unwrap());
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ico_path = out.join("app.ico");
    let mut f = std::fs::File::create(&ico_path).unwrap();
    icon_dir.write(&mut f).unwrap();

    let mut res = winres::WindowsResource::new();
    res.set_icon(ico_path.to_string_lossy());
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {}
