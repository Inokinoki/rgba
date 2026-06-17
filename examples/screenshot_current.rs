use rgba::Gba;
use rgba::KeyState;

fn save_bmp(fb: &[u32], path: &str) {
    let width = 240u32;
    let height = 160u32;
    let row_size = (width * 3 + 3) & !3;
    let pixel_size = row_size * height;
    let file_size = 54 + pixel_size;
    let mut out = Vec::with_capacity(file_size as usize);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&file_size.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.extend_from_slice(&[1, 0, 24, 0]);
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&pixel_size.to_le_bytes());
    out.extend_from_slice(&[0; 16]);
    out.extend_from_slice(&[0; 16]);
    for y in (0..height).rev() {
        let row_start = (y * width) as usize;
        let mut row = Vec::with_capacity(row_size as usize);
        for x in 0..width {
            let pixel = fb[row_start + x as usize];
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            row.extend_from_slice(&[b, g, r]);
        }
        while row.len() < row_size as usize {
            row.push(0);
        }
        out.extend_from_slice(&row);
    }
    std::fs::write(path, &out).unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
    }
    for _ in 0..80 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    save_bmp(&fb, "/tmp/current_render.bmp");
    println!("Screenshot saved to /tmp/current_render.bmp");

    std::process::Command::new("convert")
        .args(["/tmp/current_render.bmp", "/tmp/current_render.png"])
        .status()
        .expect("convert failed");
    println!("Converted to /tmp/current_render.png");
}
