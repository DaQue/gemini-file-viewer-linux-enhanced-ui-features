use std::fs;
use std::path::Path;
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};

fn build_app_icon_rgba(size: u32) -> (Vec<u8>, u32, u32) {
    let mut rgba: Vec<u8> = vec![0; (size * size * 4) as usize];
    let to_idx = |x: u32, y: u32| -> usize { ((y * size + x) * 4) as usize };
    fn put_px(buf: &mut [u8], to_idx: &dyn Fn(u32,u32)->usize, x: u32, y: u32, c: [u8; 4]) {
        let i = to_idx(x, y);
        if i + 3 < buf.len() { buf[i] = c[0]; buf[i + 1] = c[1]; buf[i + 2] = c[2]; buf[i + 3] = c[3]; }
    }
    let fill_rect = |buf: &mut [u8], x0: u32, y0: u32, w: u32, h: u32, c: [u8; 4]| {
        for yy in y0..(y0 + h).min(size) {
            for xx in x0..(x0 + w).min(size) {
                put_px(buf, &to_idx, xx, yy, c);
            }
        }
    };
    let inside_triangle = |px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32| -> bool {
        let sign = |x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32| (x1 - x3) * (y2 - y3) - (x2 - x3) * (y1 - y3);
        let b1 = sign(px, py, ax, ay, bx, by) < 0.0;
        let b2 = sign(px, py, bx, by, cx, cy) < 0.0;
        let b3 = sign(px, py, cx, cy, ax, ay) < 0.0;
        (b1 == b2) && (b2 == b3)
    };
    // Colors
    let dark = [0x1e, 0x29, 0x3b, 0xff];
    let sky = [0x0e, 0xa5, 0xe9, 0xff];
    let sep = [0x94, 0xa3, 0xb8, 0xff];
    let code1 = [0x60, 0xa5, 0xfa, 0xff];
    let code2 = [0x22, 0xc5, 0x5e, 0xff];
    let code3 = [0xf4, 0x72, 0xb6, 0xff];
    let sun = [0xf5, 0x9e, 0x0b, 0xff];
    // Lighten mountain color substantially for an earlier, pre-sunset look
    let mount = [0x8f, 0xb6, 0xff, 0xff];

    for y in 0..size {
        for x in 0..size {
            let c = if (x as i32) - (y as i32) < 0 { dark } else { sky };
            put_px(&mut rgba, &to_idx, x, y, c);
        }
    }
    for y in 0..size {
        for x in 0..size {
            let d = (x as i32) - (y as i32);
            if d == 0 || d == 1 { put_px(&mut rgba, &to_idx, x, y, sep); }
        }
    }

    // Left code bars and brackets
    fill_rect(&mut rgba, 28, 56, 110, 16, code1);
    fill_rect(&mut rgba, 28, 88, 78, 16, code2);
    fill_rect(&mut rgba, 28, 120, 96, 16, code3);
    fill_rect(&mut rgba, 20, 40, 8, 64, code1);
    fill_rect(&mut rgba, 20, 40, 28, 8, code1);
    fill_rect(&mut rgba, 20, 96, 28, 8, code1);
    fill_rect(&mut rgba, 140, 40, 8, 64, code1);
    fill_rect(&mut rgba, 120, 40, 28, 8, code1);
    fill_rect(&mut rgba, 120, 96, 28, 8, code1);

    // Mountain + sun
    let (ax, ay, bx, by, cx, cy) = (150.0, 200.0, 220.0, 200.0, 190.0, 140.0);
    for y in 120..220 {
        for x in 140..236 {
            if inside_triangle(x as f32, y as f32, ax, ay, bx, by, cx, cy) { put_px(&mut rgba, &to_idx, x, y, mount); }
        }
    }
    let (sx, sy, r2) = (210.0f32, 70.0f32, 18.0f32 * 18.0f32);
    for y in 40..100 {
        for x in 180..240 {
            let dx = x as f32 - sx; let dy = y as f32 - sy;
            if dx*dx + dy*dy <= r2 { put_px(&mut rgba, &to_idx, x, y, sun); }
        }
    }

    (rgba, size, size)
}

fn main() {
    let out_dir = Path::new("assets").join("icons");
    fs::create_dir_all(&out_dir).expect("create assets/icons");
    let sizes = [16u32, 24, 32, 48, 64, 96, 128, 256, 384, 512];
    for &s in &sizes {
        let (rgba, w, h) = build_app_icon_rgba(s);
        let img = image::RgbaImage::from_vec(w, h, rgba).expect("rgba to image");
        let out = out_dir.join(format!("icon_{}.png", s));
        img.save(&out).expect("save icon png");
        println!("wrote {}", out.display());
    }

    // Build multi-size ICO (common sizes; Windows supports up to 256)
    let mut dir = IconDir::new(ResourceType::Icon);
    for &s in &[16u32, 32, 48, 64, 128, 256] {
        let (rgba, _w, _h) = build_app_icon_rgba(s);
        let image = IconImage::from_rgba_data(s, s, rgba);
        let entry = IconDirEntry::encode(&image).expect("encode ico entry");
        dir.add_entry(entry);
    }
    let ico_path = out_dir.join("icon.ico");
    let mut f = fs::File::create(&ico_path).expect("create ico");
    dir.write(&mut f).expect("write ico");
    println!("wrote {}", ico_path.display());
}
