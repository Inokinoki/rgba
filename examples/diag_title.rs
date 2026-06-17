use rgba::Gba;

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

    // Title screen at frame 240
    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    save_ppm(&fb, "/tmp/title_240.ppm");
    let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
    let mut color_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &p in &fb {
        *color_counts.entry(p).or_insert(0) += 1;
    }
    eprintln!(
        "Frame 240: {} non-zero, {} unique colors",
        nonzero,
        color_counts.len()
    );

    // Frame 600 - let it run more
    for _ in 0..360 {
        gba.run_frame_parallel(&mut fb);
    }
    save_ppm(&fb, "/tmp/title_600.ppm");
    let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
    let mut color_counts2: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &p in &fb {
        *color_counts2.entry(p).or_insert(0) += 1;
    }
    eprintln!(
        "Frame 600: {} non-zero, {} unique colors",
        nonzero,
        color_counts2.len()
    );

    let _ = std::process::Command::new("convert")
        .args(["/tmp/title_240.ppm", "/tmp/title_240.png"])
        .status();
    let _ = std::process::Command::new("convert")
        .args(["/tmp/title_600.ppm", "/tmp/title_600.png"])
        .status();
    eprintln!("Saved PNGs");
}
