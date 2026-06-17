use rgba::Gba;
use rgba::KeyState;

fn save_ppm(fb: &[u32], path: &str) {
    let width = 240;
    let height = 160;
    let mut out = Vec::with_capacity(width * height * 3 + 100);
    out.extend_from_slice(format!("P6\n{} {}\n255\n", width, height).as_bytes());
    for y in 0..height {
        for x in 0..width {
            let pixel = fb[y * width + x];
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            out.extend_from_slice(&[r, g, b]);
        }
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
    for _ in 0..20 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
    eprintln!(
        "After 240 frames (before input): {} non-zero pixels",
        nonzero
    );

    let mut color_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &p in &fb {
        *color_counts.entry(p).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = color_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!("Top 5 colors:");
    for (color, count) in sorted.iter().take(5) {
        eprintln!("  {:08X}: {} pixels", color, count);
    }

    save_ppm(&fb, "/tmp/game_240frames.ppm");
    eprintln!("Saved /tmp/game_240frames.ppm");
    std::process::Command::new("convert")
        .args(["/tmp/game_240frames.ppm", "/tmp/game_240frames.png"])
        .status()
        .expect("convert failed");
    eprintln!("Converted to /tmp/game_240frames.png");
}
