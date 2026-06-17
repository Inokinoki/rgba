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
            let r = (pixel >> 16) as u8;
            let g = (pixel >> 8) as u8;
            let b = pixel as u8;
            out.extend_from_slice(&[r, g, b]);
        }
    }
    std::fs::write(path, &out).unwrap();
}

fn analyze(fb: &[u32], label: &str) {
    let mut color_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &p in fb {
        *color_counts.entry(p).or_insert(0) += 1;
    }
    let unique = color_counts.len();
    let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
    let green: usize = fb.iter().filter(|&&p| p == 0x0000FF00).count();
    let mut sorted: Vec<_> = color_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    println!(
        "{}: {} unique colors, {} nonzero, {} green ({:.0}%)",
        label,
        unique,
        nonzero,
        green,
        green as f64 / 384.0
    );
    for (&color, &count) in sorted.iter().take(5) {
        let r = (color >> 16) as u8;
        let g = ((color >> 8) as u8);
        let b = (color as u8);
        println!(
            "  #{:02X}{:02X}{:02X}: {} pixels ({:.1}%)",
            r,
            g,
            b,
            count,
            count as f64 / 384.0
        );
    }
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Phase 1: Title screen (600 frames to let it fully load)
    for _ in 0..600 {
        gba.run_frame_parallel(&mut fb);
    }
    analyze(&fb, "Title screen (frame 600)");
    save_ppm(&fb, "/tmp/game_title.ppm");
    std::process::Command::new("python3")
        .args([
            "-c",
            "from PIL import Image; Image.open('/tmp/game_title.ppm').save('/tmp/game_title.png')",
        ])
        .output()
        .ok();

    // Phase 2: Press START to proceed
    gba.input.press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
    }
    analyze(&fb, "After START (frame 840)");
    save_ppm(&fb, "/tmp/game_after_start.ppm");
    std::process::Command::new("python3")
        .args(["-c", "from PIL import Image; Image.open('/tmp/game_after_start.ppm').save('/tmp/game_after_start.png')"])
        .output().ok();

    // Phase 3: Press A multiple times
    for _ in 0..10 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }
    analyze(&fb, "After A x10 (frame ~1060)");
    save_ppm(&fb, "/tmp/game_ingame.ppm");
    std::process::Command::new("python3")
        .args(["-c", "from PIL import Image; Image.open('/tmp/game_ingame.ppm').save('/tmp/game_ingame.png')"])
        .output().ok();

    println!("\nSaved: /tmp/game_title.png, /tmp/game_after_start.png, /tmp/game_ingame.png");
}
