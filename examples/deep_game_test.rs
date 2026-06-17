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
            out.extend_from_slice(&[(pixel >> 16) as u8, (pixel >> 8) as u8, pixel as u8]);
        }
    }
    std::fs::write(path, &out).unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..600 {
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

    for _ in 0..50 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    for _ in 0..300 {
        gba.run_frame_parallel(&mut fb);
    }

    let mut color_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &p in &fb {
        *color_counts.entry(p).or_insert(0) += 1;
    }
    println!("Deep gameplay: {} unique colors", color_counts.len());

    let mut sorted: Vec<_> = color_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (&color, &count) in sorted.iter().take(8) {
        let r = (color >> 16) as u8;
        let g = (color >> 8) as u8;
        let b = color as u8;
        println!(
            "  #{:02X}{:02X}{:02X}: {} ({:.1}%)",
            r,
            g,
            b,
            count,
            count as f64 / 384.0
        );
    }

    save_ppm(&fb, "/tmp/game_deep.ppm");
    std::process::Command::new("python3")
        .args([
            "-c",
            "from PIL import Image; Image.open('/tmp/game_deep.ppm').save('/tmp/game_deep.png')",
        ])
        .output()
        .ok();
    println!("Saved /tmp/game_deep.png");
}
